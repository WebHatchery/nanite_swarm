//! Per-tick simulation: drills, servers, dust, biomass, tutorial, power collapse

use crate::engine::{BuildingType, DroneState, StatId, TerrainType};

use super::game_state::PlanetState;

const DUST_RATE: f32 = 0.12; // dust per second
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
        if self.paused || !real_delta.is_finite() || real_delta <= 0.0 {
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
                if allow_visuals {
                    self.spawn_resource_burst();
                }
            }
        }

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
            if self.power_negative_seconds >= 60.0 && self.power_collapse_cooldown <= 0.0 {
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

    fn update_tutorial(&mut self) {
        if self.tutorial_done {
            return;
        }

        let has_drill = !self.grid.find_buildings(BuildingType::Drill).is_empty();
        let drill_connected = self.grid.iter_tiles().any(|(_, tile)| {
            tile.building
                .as_ref()
                .map(|b| b.building_type == BuildingType::Drill && b.connected_to_core)
                .unwrap_or(false)
        });
        let conduits_unlocked = self.is_building_unlocked(BuildingType::Conduit);
        let server_unlocked = self.is_building_unlocked(BuildingType::ServerBank);
        let wind_unlocked = self.is_building_unlocked(BuildingType::WindTurbine);
        let has_wind_turbine = !self
            .grid
            .find_buildings(BuildingType::WindTurbine)
            .is_empty();
        let has_server_bank = !self
            .grid
            .find_buildings(BuildingType::ServerBank)
            .is_empty();

        match self.tutorial_step {
            0 if has_drill => self.tutorial_step = 1,
            1 if conduits_unlocked => self.tutorial_step = 2,
            2 if drill_connected => self.tutorial_step = 3,
            3 if server_unlocked && has_server_bank => self.tutorial_step = 4,
            4 if wind_unlocked && has_wind_turbine => {
                self.tutorial_step = 5;
                self.tutorial_done = true;
            }
            _ => {}
        }
    }

    pub(super) fn trigger_power_collapse(&mut self) {
        self.power_negative_seconds = 0.0;
        self.power_collapse_cooldown = 120.0;
        self.power_collapse_shutdown = 20.0;
        self.research_lock_timer = 30.0;
        self.collapse_notice_timer = 10.0;

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
        self.resources.data *= 0.7;
        self.research.research_progress *= 0.75;
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

        for _ in 0..70 {
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
        assert_eq!(state.resources.data, 70.0);
        assert_eq!(state.research.research_progress, 75.0);
        assert_eq!(state.power_collapse_cooldown, 120.0);
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
