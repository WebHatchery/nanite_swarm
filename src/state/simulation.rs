//! Per-tick simulation: drills, servers, dust, biomass, tutorial, power collapse

use crate::engine::{BuildingType, DroneState, StatId, TerrainType};
use macroquad_toolkit::math::lerp;

use super::game_state::PlanetState;

pub(super) const DUST_RATE: f32 = 0.12; // dust per second
/// One point on the throughput graph is one second of world time.
const THROUGHPUT_SAMPLE_SECONDS: f32 = 1.0;
const SWEEPER_RATE: f32 = 0.6; // dust cleared per second
const SWEEPER_RADIUS: i32 = 3;
const FILTER_RADIUS: i32 = 3;
const FILTER_RATE_MULTIPLIER: f32 = 0.6;
const POLLUTION_RADIUS: i32 = 3;
const POLLUTION_RATE_MULTIPLIER: f32 = 1.3;
/// Acid at full strength corrodes the network this many times faster than dust
/// settles on it.
const ACID_RAIN_MULTIPLIER: f32 = 4.0;
/// Tiles a Shield Generator or Heater Node protects, measured like the sweeper.
pub(super) const HAZARD_COUNTER_RADIUS: i32 = 4;
/// Share of a hazard a counter building holds off inside its radius.
pub(super) const HAZARD_COUNTER_STRENGTH: f32 = 0.9;
// Config-driven values are loaded from assets/game_config.json.

/// The simulation advances in whole steps of this length and nothing else.
/// Frame time only decides *how many* steps run, never how long one is, so the
/// world behaves the same at 30fps, at 144fps, and in a headless test.
pub const TICK_SECONDS: f32 = 1.0 / 30.0;

/// Ceiling on catch-up steps per call. A long stall (a load, a dragged window)
/// drops the excess rather than spending minutes replaying it. Fast-forward
/// needs headroom here too: four times speed at thirty frames a second is four
/// steps a frame.
const MAX_CATCHUP_TICKS: u32 = 12;

/// The speeds the player can pick between, slowest first.
pub const TIME_SCALES: [f32; 4] = [0.5, 1.0, 2.0, 4.0];

impl PlanetState {
    /// Advance the world by real elapsed time, running whole [`TICK_SECONDS`]
    /// steps and banking the remainder. Returns how many steps ran, so callers
    /// that keep their own timers can advance by the same amount.
    pub fn advance(&mut self, real_delta: f32, allow_visuals: bool) -> u32 {
        if !real_delta.is_finite() || real_delta <= 0.0 {
            return 0;
        }
        // Toasts are UI, not world: they fade in real time and keep fading
        // while the world is stopped.
        self.notifications.update(real_delta);
        if self.paused {
            return 0;
        }

        // Speed scales how much world time a second of real time buys, not how
        // long a step is: the step is the one thing that never changes.
        self.sim_accumulator += real_delta * self.time_scale.max(0.0);
        let mut ticks = 0;
        while self.sim_accumulator >= TICK_SECONDS && ticks < MAX_CATCHUP_TICKS {
            self.sim_accumulator -= TICK_SECONDS;
            self.step(TICK_SECONDS, allow_visuals);
            ticks += 1;
        }

        if ticks == MAX_CATCHUP_TICKS {
            // Dropped time: never let a backlog snowball into a freeze.
            self.sim_accumulator = 0.0;
        }

        ticks
    }

    /// One simulation step. Every caller passes a fixed, known step length.
    pub fn step(&mut self, delta_time: f32, allow_visuals: bool) {
        let sim_delta = if self.battery_seconds <= 0.0 {
            delta_time * 0.1
        } else {
            delta_time
        };

        self.time_played += sim_delta as f64;

        self.update_dust(sim_delta);

        self.update_biomass_harvesters(sim_delta);
        self.update_tutorial();

        if self.power_collapse_cooldown > 0.0 {
            self.power_collapse_cooldown = (self.power_collapse_cooldown - delta_time).max(0.0);
        }
        if self.power_collapse_shutdown > 0.0 {
            self.power_collapse_shutdown = (self.power_collapse_shutdown - delta_time).max(0.0);
        }
        if self.research_lock_timer > 0.0 {
            self.research_lock_timer = (self.research_lock_timer - delta_time).max(0.0);
        }
        if self.collapse_notice_timer > 0.0 {
            self.collapse_notice_timer = (self.collapse_notice_timer - delta_time).max(0.0);
        }
        if self.arrival_notice_timer > 0.0 {
            self.arrival_notice_timer = (self.arrival_notice_timer - delta_time).max(0.0);
        }
        if self.save_notice_timer > 0.0 {
            self.save_notice_timer = (self.save_notice_timer - delta_time).max(0.0);
        }

        // Update drones
        if self.power_collapse_shutdown <= 0.0 {
            let core = self.grid.find_core();
            let events = self.drones.update(sim_delta);
            let mut delivered_total = 0.0;
            for event in events {
                match event {
                    crate::engine::DroneEvent::Delivered {
                        amount,
                        at,
                        resource,
                        ..
                    } => {
                        if Some(at) == core {
                            match resource {
                                crate::engine::ResourceType::Alloy => {
                                    self.resources.alloy += amount
                                }
                                _ => delivered_total += amount,
                            }
                        } else {
                            // Ore dropped at a processing building waits there
                            // until that building gets round to it.
                            *self.input_buffers.entry((at.x, at.y)).or_insert(0.0) += amount;
                        }
                    }
                    crate::engine::DroneEvent::ReachedDrill { drone_id } => {
                        if let Some(drone) = self.drones.get_drone_mut(drone_id) {
                            drone.state = DroneState::Idle;
                        }
                    }
                    _ => {}
                }
            }
            if delivered_total > 0.0 {
                self.resources.minerals += delivered_total;
                self.delivered_since_sample += delivered_total;
                if allow_visuals {
                    self.spawn_resource_burst();
                }
            }
        }

        self.sample_throughput(sim_delta);

        // Process drills and server banks
        if self.power_collapse_shutdown <= 0.0 {
            self.update_logistics(sim_delta, allow_visuals);
            self.update_servers(sim_delta);
            self.update_recipes(sim_delta);
            self.update_seed_ship(sim_delta);
        }

        // Particles for drone motion
        if allow_visuals {
            self.spawn_drone_trails(sim_delta);
            self.update_particles(sim_delta);
        }

        // Power-based energy generation
        self.grid.update_power_grid();
        let net_power = self.net_power();
        self.power_balance = net_power;
        self.resources.energy += self.power_balance * sim_delta;

        // Passive data trickle from Core to avoid research deadlock
        if let Some(core_pos) = self.grid.find_core() {
            if let Some(core_tile) = self.grid.get(core_pos) {
                if let Some(core) = core_tile.building.as_ref() {
                    if core.powered && !core.is_dust_stalled() {
                        let rate = self
                            .stats
                            .apply(StatId::DataGeneration, self.config.resources.core_data_rate);
                        self.resources.data += rate * sim_delta * core.dust_efficiency();
                    }
                }
            }
        }

        if net_power < 0.0 {
            self.power_negative_seconds += delta_time;
            if self.power_negative_seconds >= self.config.collapse.negative_power_seconds
                && self.power_collapse_cooldown <= 0.0
            {
                self.trigger_power_collapse();
            }
        } else {
            self.power_negative_seconds = 0.0;
        }

        // Battery drain for offline mechanics
        self.battery_seconds = (self.battery_seconds - delta_time).max(0.0);

        // Cap resources
        self.resources.energy = self
            .resources
            .energy
            .clamp(0.0, self.config.resources.max_energy);
        self.resources.minerals = self.resources.minerals.min(self.mineral_capacity());
        self.resources.data = self.resources.data.min(1000.0);
        self.resources.biomass = self.resources.biomass.min(1000.0);
        self.resources.alloy = self.resources.alloy.min(1000.0);

        self.update_achievements();

        if self.offline_notice_timer > 0.0 {
            self.offline_notice_timer = (self.offline_notice_timer - delta_time).max(0.0);
        }

        for anim in &mut self.placement_anims {
            anim.timer = (anim.timer - delta_time).max(0.0);
        }
        self.placement_anims.retain(|anim| anim.timer > 0.0);
    }

    /// Run every processing building's recipe.
    ///
    /// A recipe only runs as far as its inputs allow, so a smelter starved of
    /// minerals produces proportionally less rather than stopping dead - the
    /// same shape as the drill buffer, and it keeps the numbers continuous for
    /// the fixed timestep.
    fn update_recipes(&mut self, delta_time: f32) {
        for (pos, recipe) in self.recipe_buildings() {
            let Some(building) = self.grid.get(pos).and_then(|tile| tile.building.as_ref()) else {
                continue;
            };
            if !building.powered || building.is_dust_stalled() {
                continue;
            }

            let delivered = self
                .input_buffers
                .get(&(pos.x, pos.y))
                .copied()
                .unwrap_or(0.0);

            let mut scale = building.dust_efficiency() * delta_time;
            if recipe.minerals_in > 0.0 {
                // Ore has to have been carried here. A smelter with an empty
                // hopper is idle however full the global pool is.
                scale = scale.min(delivered / recipe.minerals_in);
            }
            if recipe.biomass_in > 0.0 {
                scale = scale.min(self.resources.biomass / recipe.biomass_in);
            }
            if scale <= 0.0 {
                continue;
            }

            if recipe.minerals_in > 0.0 {
                let taken = recipe.minerals_in * scale;
                if let Some(buffer) = self.input_buffers.get_mut(&(pos.x, pos.y)) {
                    *buffer = (*buffer - taken).max(0.0);
                }
            }
            self.resources.biomass -= recipe.biomass_in * scale;
            // Output waits on the pad for a drone, the same as a drill's ore.
            *self.output_buffers.entry((pos.x, pos.y)).or_insert(0.0) += recipe.alloy_out * scale;
        }
    }

    /// Every placed building that has a recipe, with it.
    fn recipe_buildings(&self) -> Vec<(crate::engine::GridPos, crate::data::RecipeDef)> {
        crate::data::game_data()
            .buildings
            .iter()
            .filter(|def| !def.recipe.is_empty())
            .filter_map(|def| BuildingType::from_id(&def.id).map(|kind| (kind, def.recipe)))
            .flat_map(|(kind, recipe)| {
                self.grid
                    .find_buildings(kind)
                    .into_iter()
                    .map(move |pos| (pos, recipe))
            })
            .collect()
    }

    /// Update server bank data generation
    fn update_servers(&mut self, delta_time: f32) {
        let rate = self.stats.apply(
            StatId::DataGeneration,
            self.config.resources.server_data_rate,
        );

        for server_pos in self.grid.find_buildings(BuildingType::ServerBank) {
            let Some(building) = self.grid.get(server_pos).and_then(|t| t.building.as_ref()) else {
                continue;
            };
            if !building.powered || building.is_dust_stalled() {
                continue;
            }
            self.resources.data += rate * building.dust_efficiency() * delta_time;
        }
    }

    /// How far a building's upkeep effect reaches, if it has one. The view
    /// asks rather than duplicating the numbers.
    pub fn coverage_radius(&self, building_type: BuildingType) -> Option<i32> {
        match building_type {
            BuildingType::Sweeper => Some(SWEEPER_RADIUS),
            BuildingType::ShieldGenerator | BuildingType::HeaterNode => Some(HAZARD_COUNTER_RADIUS),
            _ => None,
        }
    }

    /// Positions of powered, working buildings of a type.
    pub(super) fn powered_positions(
        &self,
        building_type: BuildingType,
    ) -> Vec<crate::engine::GridPos> {
        self.grid
            .find_buildings(building_type)
            .into_iter()
            .filter(|pos| {
                self.grid
                    .get(*pos)
                    .and_then(|tile| tile.building.as_ref())
                    .is_some_and(|building| building.powered && !building.is_dust_stalled())
            })
            .collect()
    }

    fn update_dust(&mut self, delta_time: f32) {
        let dust_rate = self.stats.apply(StatId::DustAccumulation, DUST_RATE);
        // Acid eats the network specifically: a corroded conduit stalls, and a
        // stalled conduit stops carrying traffic, which is how a Venus run
        // fails rather than merely slowing.
        let acid_rate = DUST_RATE * self.acid_strength() * ACID_RAIN_MULTIPLIER;
        let shields = self.powered_positions(BuildingType::ShieldGenerator);
        let sweeper_positions = self.grid.find_buildings(BuildingType::Sweeper);
        let powered_sweepers: Vec<_> = sweeper_positions
            .into_iter()
            .filter(|pos| {
                self.grid
                    .get(*pos)
                    .and_then(|t| t.building.as_ref())
                    .map(|b| b.powered && !b.is_dust_stalled())
                    .unwrap_or(false)
            })
            .collect();
        let filter_positions: Vec<_> = self
            .grid
            .iter_tiles()
            .filter_map(|(pos, tile)| if tile.filter { Some(pos) } else { None })
            .collect();
        let cleared_forest_positions: Vec<_> = self
            .grid
            .iter_tiles()
            .filter_map(|(pos, tile)| if tile.forest_cleared { Some(pos) } else { None })
            .collect();

        for (pos, tile) in self.grid.iter_tiles_mut() {
            let Some(building) = tile.building.as_mut() else {
                continue;
            };
            let mut rate = dust_rate;
            if acid_rate > 0.0 && building.transmits_power() {
                let sheltered = shields
                    .iter()
                    .any(|shield| pos.distance(*shield) as i32 <= HAZARD_COUNTER_RADIUS);
                rate += if sheltered {
                    acid_rate * (1.0 - HAZARD_COUNTER_STRENGTH)
                } else {
                    acid_rate
                };
            }

            if filter_positions
                .iter()
                .any(|filter_pos| pos.distance(*filter_pos) as i32 <= FILTER_RADIUS)
            {
                rate *= FILTER_RATE_MULTIPLIER;
            }
            if cleared_forest_positions
                .iter()
                .any(|cleared_pos| pos.distance(*cleared_pos) as i32 <= POLLUTION_RADIUS)
            {
                rate *= POLLUTION_RATE_MULTIPLIER;
            }

            // Apply sweeper cleaning if nearby powered sweeper exists
            let mut clean_rate = 0.0;
            if powered_sweepers
                .iter()
                .any(|sweeper_pos| pos.distance(*sweeper_pos) as i32 <= SWEEPER_RADIUS)
            {
                clean_rate = SWEEPER_RATE;
            }

            building.dust =
                (building.dust + rate * delta_time - clean_rate * delta_time).clamp(0.0, 100.0);
        }
    }

    fn update_biomass_harvesters(&mut self, delta_time: f32) {
        let output = self.config.resources.biomass_power_output;
        let rate = self.config.resources.biomass_consumption_rate;
        let mut power_bonus = 0.0;

        for (_, tile) in self.grid.iter_tiles_mut() {
            let Some(building) = tile.building.as_mut() else {
                continue;
            };
            if building.building_type != BuildingType::BiomassHarvester {
                continue;
            }

            if tile.terrain != TerrainType::Forest || tile.biomass_amount <= 0.0 {
                continue;
            }
            if !building.powered || building.is_dust_stalled() {
                continue;
            }

            let available = tile.biomass_amount;
            if available <= 0.0 || rate <= 0.0 {
                continue;
            }

            let consume = (rate * delta_time).min(available);
            tile.biomass_amount = (tile.biomass_amount - consume).max(0.0);
            self.resources.biomass += consume;

            let fraction = if rate * delta_time > 0.0 {
                consume / (rate * delta_time)
            } else {
                0.0
            };
            power_bonus += output * fraction * building.dust_power_generation_multiplier();

            if tile.biomass_amount <= 0.0 {
                tile.terrain = TerrainType::Empty;
                tile.forest_cleared = true;
                tile.filter = false;
            }
        }

        self.biomass_power_bonus = power_bonus;
    }

    /// Bank one second of deliveries into the graph.
    ///
    /// Sampled on world time rather than per tick, so the shape of the line
    /// means the same thing at every game speed.
    fn sample_throughput(&mut self, delta_time: f32) {
        self.throughput_timer += delta_time;
        while self.throughput_timer >= THROUGHPUT_SAMPLE_SECONDS {
            self.throughput_timer -= THROUGHPUT_SAMPLE_SECONDS;
            let rate = self.delivered_since_sample / THROUGHPUT_SAMPLE_SECONDS;
            self.throughput.push(rate);
            self.delivered_since_sample = 0.0;
        }
    }

    /// How far along the swarm is, for anything that should cost more the
    /// more there is of it. Zero for a base of nothing, one at full scale.
    pub fn collapse_scale(&self) -> f32 {
        let full = self.config.collapse.full_scale_structures.max(1.0);
        (self.grid.total_buildings() as f32 / full).clamp(0.0, 1.0)
    }

    /// Bring the grid down. Public because the screenshot harness stages one;
    /// the simulation reaches it through sustained negative power.
    pub fn trigger_power_collapse(&mut self) {
        let collapse = self.config.collapse.clone();
        // A bigger swarm takes longer to bring back up and loses more of what
        // it was holding. Twenty flat seconds stung hardest exactly when the
        // player could least afford it and stopped registering later.
        let scale = self.collapse_scale();
        let shutdown = lerp(
            collapse.min_shutdown_seconds,
            collapse.max_shutdown_seconds,
            scale,
        );
        let loss = lerp(collapse.min_data_loss, collapse.max_data_loss, scale).clamp(0.0, 1.0);

        self.power_negative_seconds = 0.0;
        self.power_collapse_cooldown = collapse.cooldown_seconds;
        self.power_collapse_shutdown = shutdown;
        // Kept so the drones can sag over exactly as long as this one lasts.
        self.power_collapse_length = shutdown;
        self.research_lock_timer = shutdown * collapse.research_lock_ratio.max(0.0);
        self.collapse_notice_timer = collapse.notice_seconds;

        // Drones drop cargo and shut down
        for drone in self.drones.drones_mut() {
            drone.carrying = 0.0;
            drone.state = DroneState::Error;
            drone.path.clear();
            drone.path_index = 0;
            drone.progress = 0.0;
            drone.target = drone.position;
        }

        // Corrupt data and research progress
        self.resources.data *= 1.0 - loss;
        self.research.research_progress *= 1.0 - loss;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::engine::GridPos;

    fn state() -> PlanetState {
        PlanetState::new(2, 42, GameConfig::default())
    }

    /// A world with `count` conduits standing beside the Core, for the
    /// collapse to have something to be the size of.
    fn state_with_structures(count: i32) -> PlanetState {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        state.grid.reveal_around(core, 32);
        for step in 1..=count {
            let pos = GridPos::new(core.x + step % 8, core.y + 1 + step / 8);
            if let Some(tile) = state.grid.get_mut(pos) {
                tile.terrain = TerrainType::Empty;
            }
            state.grid.place_building(pos, BuildingType::Conduit);
        }
        state
    }

    #[test]
    fn a_small_swarm_is_knocked_down_for_less_time_than_a_large_one() {
        let mut small = state_with_structures(2);
        let mut large = state_with_structures(80);
        assert!(small.collapse_scale() < large.collapse_scale());

        small.trigger_power_collapse();
        large.trigger_power_collapse();

        assert!(
            small.power_collapse_shutdown < large.power_collapse_shutdown,
            "small was down {}s and large {}s",
            small.power_collapse_shutdown,
            large.power_collapse_shutdown
        );
        assert!(small.research_lock_timer < large.research_lock_timer);
    }

    #[test]
    fn the_shutdown_stays_between_the_two_ends_it_was_given() {
        let config = GameConfig::default();
        let (min, max) = (
            config.collapse.min_shutdown_seconds,
            config.collapse.max_shutdown_seconds,
        );
        for count in [0, 1, 30, 80, 400] {
            let mut state = state_with_structures(count);
            state.trigger_power_collapse();
            let down = state.power_collapse_shutdown;
            assert!(
                (min..=max).contains(&down),
                "{} structures went down for {}s",
                count,
                down
            );
            assert_eq!(state.power_collapse_length, down);
        }
    }

    #[test]
    fn a_bigger_swarm_loses_a_bigger_share_of_what_it_was_holding() {
        let mut small = state_with_structures(2);
        let mut large = state_with_structures(80);
        small.resources.data = 1_000.0;
        large.resources.data = 1_000.0;

        small.trigger_power_collapse();
        large.trigger_power_collapse();

        assert!(
            small.resources.data > large.resources.data,
            "small kept {} and large kept {}",
            small.resources.data,
            large.resources.data
        );
        // And neither is wiped out: a collapse is a setback, never a death.
        assert!(large.resources.data > 0.0);
    }

    #[test]
    fn the_throughput_graph_starts_with_nothing_to_say() {
        let state = state();
        assert!(state.throughput.buckets().is_empty());
        assert_eq!(state.throughput.last(), None);
    }

    #[test]
    fn a_second_of_deliveries_becomes_one_point_on_the_graph() {
        let mut state = state();
        state.delivered_since_sample = 12.0;
        state.sample_throughput(1.0);

        assert_eq!(state.throughput.len(), 1);
        assert_eq!(state.throughput.last(), Some(12.0));
        // And the accumulator starts over rather than double-counting.
        assert_eq!(state.delivered_since_sample, 0.0);
    }

    #[test]
    fn a_quiet_second_is_recorded_as_a_quiet_second_not_skipped() {
        let mut state = state();
        state.delivered_since_sample = 6.0;
        state.sample_throughput(1.0);
        state.sample_throughput(1.0);

        assert_eq!(state.throughput.len(), 2);
        assert_eq!(state.throughput.last(), Some(0.0));
        // The spike is still the peak, which is the point of keeping ranges.
        assert_eq!(state.throughput.max(), Some(6.0));
    }

    #[test]
    fn a_long_stretch_of_world_time_lands_every_second_it_covers() {
        let mut state = state();
        state.delivered_since_sample = 5.0;
        // Four seconds in one go: one second of deliveries and three quiet.
        state.sample_throughput(4.0);
        assert_eq!(state.throughput.len(), 4);
        assert_eq!(state.throughput.max(), Some(5.0));
    }

    #[test]
    fn dust_accumulates_on_powered_buildings_over_time() {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);

        state.update_dust(10.0);
        let dust = state.grid.get(pos).unwrap().building.as_ref().unwrap().dust;
        assert!(dust > 0.0);
    }

    #[test]
    fn advance_runs_whole_ticks_and_banks_the_remainder() {
        let mut state = state();
        assert_eq!(state.advance(TICK_SECONDS * 2.5, false), 2);
        assert!((state.sim_accumulator - TICK_SECONDS * 0.5).abs() < 1e-4);
        assert!((state.time_played - (TICK_SECONDS * 2.0) as f64).abs() < 1e-4);

        // The banked half tick means the next one needs only half a tick more.
        assert_eq!(state.advance(TICK_SECONDS * 0.6, false), 1);
    }

    #[test]
    fn a_paused_world_does_not_move() {
        let mut state = state();
        state.toggle_pause();
        assert!(state.paused);

        assert_eq!(state.advance(10.0, false), 0);
        assert_eq!(state.time_played, 0.0);
        assert_eq!(state.sim_accumulator, 0.0);

        // And starts again exactly where it stopped.
        state.toggle_pause();
        assert!(state.advance(TICK_SECONDS, false) > 0);
        assert!(state.time_played > 0.0);
    }

    #[test]
    fn speed_buys_more_world_time_for_the_same_real_time() {
        let mut normal = state();
        let mut fast = state();
        fast.change_speed(true);
        assert_eq!(fast.time_scale, 2.0);

        // One second of wall clock each, in frames.
        for _ in 0..60 {
            normal.advance(1.0 / 60.0, false);
            fast.advance(1.0 / 60.0, false);
        }

        let ratio = fast.time_played / normal.time_played;
        assert!(
            (ratio - 2.0).abs() < 0.1,
            "double speed simulated {ratio} times as much"
        );
    }

    #[test]
    fn the_speed_ladder_stops_at_both_ends() {
        let mut state = state();
        for _ in 0..10 {
            state.change_speed(false);
        }
        assert_eq!(state.time_scale, TIME_SCALES[0]);
        for _ in 0..10 {
            state.change_speed(true);
        }
        assert_eq!(state.time_scale, TIME_SCALES[TIME_SCALES.len() - 1]);
    }

    #[test]
    fn the_fastest_speed_is_not_capped_away_by_the_catch_up_limit() {
        let mut state = state();
        for _ in 0..3 {
            state.change_speed(true);
        }
        assert_eq!(state.time_scale, 4.0);

        // A slow frame at top speed still buys everything it should.
        let ticks = state.advance(1.0 / 30.0, false);
        let expected = (4.0 / 30.0 / TICK_SECONDS).floor() as u32;
        assert_eq!(ticks, expected, "fast-forward lost time to the tick cap");
    }

    #[test]
    fn advance_ignores_a_zero_or_nonsense_delta() {
        let mut state = state();
        assert_eq!(state.advance(0.0, false), 0);
        assert_eq!(state.advance(-1.0, false), 0);
        assert_eq!(state.advance(f32::NAN, false), 0);
        assert_eq!(state.time_played, 0.0);
    }

    #[test]
    fn advance_drops_a_long_backlog_instead_of_replaying_it() {
        let mut state = state();
        // Ten seconds of stall: run the cap, then start clean.
        assert_eq!(state.advance(10.0, false), MAX_CATCHUP_TICKS);
        assert_eq!(state.sim_accumulator, 0.0);
    }

    #[test]
    fn frame_rate_does_not_change_how_much_world_time_passes() {
        let mut fast = state();
        let mut slow = state();

        // Two seconds of wall clock, at 120fps and at 40fps.
        let mut fast_ticks = 0;
        for _ in 0..240 {
            fast_ticks += fast.advance(1.0 / 120.0, false);
        }
        let mut slow_ticks = 0;
        for _ in 0..80 {
            slow_ticks += slow.advance(1.0 / 40.0, false);
        }

        assert!(
            (fast_ticks as i64 - slow_ticks as i64).abs() <= 1,
            "{fast_ticks} vs {slow_ticks} ticks for the same wall clock"
        );
        assert!((fast.time_played - slow.time_played).abs() <= TICK_SECONDS as f64 * 1.5);
        assert!((fast.resources.energy - slow.resources.energy).abs() <= 1.0);
    }

    #[test]
    fn power_collapse_triggers_after_sustained_negative_power() {
        let mut state = state();
        // Force a persistent deficit and drive the simulation past the 60s threshold.
        state.resources.energy = 1_000_000.0;
        state.config.resources.max_energy = 1_000_000.0;
        state.resources.minerals = 1_000_000.0;
        // A Server Bank placed adjacent to the Core is powered directly (Core
        // transmits power to neighbors) and consumes more than the Core generates.
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.unlock_building(BuildingType::ServerBank);
        state.select_building(BuildingType::ServerBank);
        assert!(state.try_place_building(pos));

        assert!(state.net_power() < 0.0);

        // Just past the trigger: the shutdown is short for a base this small,
        // so running well past it would only prove it had already lifted.
        for _ in 0..61 {
            state.step(1.0, false);
        }

        assert!(state.power_collapse_cooldown > 0.0);
        assert!(state.power_collapse_shutdown > 0.0);
    }

    #[test]
    fn trigger_power_collapse_drops_drone_cargo_and_corrupts_progress() {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);
        state.drones.drones_mut()[0].carrying = 5.0;
        state.resources.data = 100.0;
        state.research.research_progress = 100.0;

        state.trigger_power_collapse();

        assert_eq!(state.drones.drones()[0].carrying, 0.0);
        assert_eq!(state.drones.drones()[0].state, DroneState::Error);
        // How much is lost scales with the size of the base, so this pins the
        // ends rather than one number that moves with the balance.
        let collapse = &GameConfig::default().collapse;
        let kept = state.resources.data;
        assert!(
            kept < 100.0 * (1.0 - collapse.min_data_loss) + 0.001
                && kept > 100.0 * (1.0 - collapse.max_data_loss),
            "a two-building base kept {} of 100 Data",
            kept
        );
        assert_eq!(state.research.research_progress, kept);
        assert_eq!(state.power_collapse_cooldown, collapse.cooldown_seconds);
    }

    #[test]
    fn tutorial_advances_when_first_drill_is_placed() {
        let mut state = state();
        assert_eq!(state.tutorial_step, 0);
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);

        state.update_tutorial();
        assert_eq!(state.tutorial_step, 1);
    }
}
