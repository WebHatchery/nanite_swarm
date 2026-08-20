//! Per-tick simulation: drills, servers, dust, biomass, tutorial, power collapse

use crate::engine::{BuildingType, DroneState, StatId, TerrainType};
use macroquad_toolkit::math::lerp;

use super::game_state::PlanetState;

/// One point on the throughput graph is one second of world time.
const THROUGHPUT_SAMPLE_SECONDS: f32 = 1.0;
// Everything the upkeep loop runs on - dust, sweepers, filters, pollution,
// acid and the hazard counters - is `upkeep` in assets/game_config.json.

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
pub const TIME_SCALES: [f32; 5] = [0.5, 1.0, 2.0, 4.0, 8.0];

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

        self.update_auto_clocking(sim_delta);
        self.update_dust(sim_delta);
        self.update_heat(sim_delta);

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
            let mut minerals_delivered = 0.0;
            for event in events {
                match event {
                    crate::engine::DroneEvent::Delivered {
                        amount,
                        at,
                        resource,
                        ..
                    } => {
                        if Some(at) == core {
                            self.emit_audio(super::audio::AudioEvent::Delivery);
                            self.resources.add(resource, amount);
                            if resource == crate::engine::ResourceType::Minerals {
                                minerals_delivered += amount;
                            }
                        } else {
                            // Ore dropped at a processing building waits there
                            // until that building gets round to it.
                            *self.input_buffers.entry((at.x, at.y)).or_insert(0.0) += amount;
                            *self
                                .input_hoppers
                                .entry((at.x, at.y))
                                .or_default()
                                .entry(resource)
                                .or_insert(0.0) += amount;
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
            if minerals_delivered > 0.0 {
                self.delivered_since_sample += minerals_delivered;
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
            self.update_exports(sim_delta);
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
        self.resources.components = self.resources.components.min(1000.0);

        self.update_achievements();
        self.update_core_stage();

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
        let dust_response = self.resolved_dust_response();
        for (pos, recipe) in self.recipe_buildings() {
            let Some(building) = self.grid.get(pos).and_then(|tile| tile.building.as_ref()) else {
                continue;
            };
            if !building.powered
                || building.standby
                || building.is_dust_stalled_with(&dust_response)
            {
                continue;
            }

            let hoppers = self.input_hoppers.get(&(pos.x, pos.y));

            // How much of a second's work every input can actually cover. The
            // carried one comes out of this building's hopper; a building with
            // an empty hopper is idle however full the global pool is.
            let mut scale = building.dust_efficiency_with(&dust_response)
                * building.work_multiplier()
                * self.factory_focus_multiplier(building.building_type)
                * delta_time;
            let physical_output_rate: f32 = recipe
                .outputs
                .iter()
                .filter_map(|(id, rate)| {
                    crate::engine::ResourceType::from_id(id)
                        .is_some_and(crate::engine::ResourceType::is_physical)
                        .then_some(*rate)
                })
                .sum();
            if physical_output_rate > 0.0 {
                let waiting = self
                    .output_buffers
                    .get(&(pos.x, pos.y))
                    .copied()
                    .unwrap_or(0.0);
                let room = (self.processor_pad_capacity() - waiting).max(0.0);
                scale = scale.min(room / physical_output_rate);
            }
            for (id, rate) in &recipe.inputs {
                if *rate <= 0.0 {
                    continue;
                }
                let Some(resource) = crate::engine::ResourceType::from_id(id) else {
                    continue;
                };
                let available = if recipe.carried_ids().contains(&resource.id()) {
                    hoppers
                        .and_then(|bucket| bucket.get(&resource))
                        .copied()
                        .unwrap_or_else(|| {
                            if recipe.carried.as_deref() == Some(resource.id()) {
                                self.input_buffers
                                    .get(&(pos.x, pos.y))
                                    .copied()
                                    .unwrap_or(0.0)
                            } else {
                                0.0
                            }
                        })
                } else {
                    self.resources.get(resource)
                };
                scale = scale.min(available / rate);
            }
            if scale <= 0.0 {
                continue;
            }

            for (id, rate) in &recipe.inputs {
                let Some(resource) = crate::engine::ResourceType::from_id(id) else {
                    continue;
                };
                let taken = rate * scale;
                if recipe.carried_ids().contains(&resource.id()) {
                    let hoppers = self.input_hoppers.entry((pos.x, pos.y)).or_default();
                    if let Some(buffer) = hoppers.get_mut(&resource) {
                        *buffer = (*buffer - taken).max(0.0);
                    }
                    if let Some(buffer) = self.input_buffers.get_mut(&(pos.x, pos.y)) {
                        *buffer = (*buffer - taken).max(0.0);
                    }
                } else {
                    self.resources.add(resource, -taken);
                }
                match resource {
                    crate::engine::ResourceType::Minerals => {
                        self.factory_flow_since_sample.minerals_consumed += taken
                    }
                    crate::engine::ResourceType::Alloy => {
                        self.factory_flow_since_sample.alloy_consumed += taken
                    }
                    crate::engine::ResourceType::Components => {
                        self.factory_flow_since_sample.components_consumed += taken
                    }
                    _ => {}
                }
            }

            for (id, rate) in &recipe.outputs {
                let Some(resource) = crate::engine::ResourceType::from_id(id) else {
                    continue;
                };
                // Data is the one output research speaks to by name, so it
                // goes through the same stat the server banks use.
                let made = if resource == crate::engine::ResourceType::Data {
                    self.stats.apply(StatId::DataGeneration, rate * scale)
                } else {
                    rate * scale
                };
                if resource.is_physical() {
                    // Waits on the pad for a drone, the same as a drill's ore.
                    *self.output_buffers.entry((pos.x, pos.y)).or_insert(0.0) += made;
                } else {
                    // Nothing carries Data. It is simply known.
                    self.resources.add(resource, made);
                }
                match resource {
                    crate::engine::ResourceType::Alloy => {
                        self.factory_flow_since_sample.alloy_produced += made
                    }
                    crate::engine::ResourceType::Components => {
                        self.factory_flow_since_sample.components_produced += made
                    }
                    crate::engine::ResourceType::Data => {
                        self.factory_flow_since_sample.data_produced += made
                    }
                    _ => {}
                }
            }
        }
    }

    /// Every placed building that has a recipe, with it.
    fn recipe_buildings(&self) -> Vec<(crate::engine::GridPos, &'static crate::data::RecipeDef)> {
        crate::data::game_data()
            .buildings
            .iter()
            .filter(|def| !def.recipe.is_empty())
            .filter_map(|def| BuildingType::from_id(&def.id).map(|kind| (kind, &def.recipe)))
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
        let dust_response = self.resolved_dust_response();
        let rate = self.stats.apply(
            StatId::DataGeneration,
            self.config.resources.server_data_rate,
        );

        for server_pos in self.grid.find_buildings(BuildingType::ServerBank) {
            let Some(building) = self.grid.get(server_pos).and_then(|t| t.building.as_ref()) else {
                continue;
            };
            if !building.powered
                || building.standby
                || building.is_dust_stalled_with(&dust_response)
            {
                continue;
            }
            let heat_efficiency =
                if building.is_overheated(self.config.buildings.server_bank_heat_capacity) {
                    self.config.buildings.overheat_penalty
                } else {
                    1.0
                };
            self.resources.data += rate
                * building.dust_efficiency_with(&dust_response)
                * building.work_multiplier()
                * self.factory_focus_multiplier(BuildingType::ServerBank)
                * heat_efficiency
                * delta_time;
        }
    }

    fn update_heat(&mut self, delta_time: f32) {
        let dust_response = self.resolved_dust_response();
        let cooling = self.config.buildings.water_cooling_rate.max(0.0);
        let capacity = self.config.buildings.server_bank_heat_capacity.max(1.0);
        let server_focus_multiplier = self.factory_focus_multiplier(BuildingType::ServerBank);
        let water_tiles: Vec<_> = self
            .grid
            .iter_tiles()
            .filter_map(|(pos, tile)| (tile.terrain == TerrainType::Water).then_some(pos))
            .collect();
        for (pos, tile) in self.grid.iter_tiles_mut() {
            let Some(building) = tile.building.as_mut() else {
                continue;
            };
            let def = crate::data::game_data().building(building.building_type.id());
            let generated = if building.building_type == BuildingType::ServerBank
                && building.powered
                && !building.is_dust_stalled_with(&dust_response)
            {
                self.config.buildings.server_bank_heat_per_second
                    * building.work_multiplier()
                    * server_focus_multiplier
            } else {
                0.0
            };
            let water_cooling = if def.water_cooling
                && water_tiles
                    .iter()
                    .any(|water| pos.distance(*water) as i32 <= 1)
            {
                cooling
            } else {
                0.0
            };
            building.heat =
                (building.heat + (generated - water_cooling) * delta_time).clamp(0.0, capacity);
        }
    }

    /// How far a building's upkeep effect reaches, if it has one. The view
    /// asks rather than duplicating the numbers.
    pub fn coverage_radius(&self, building_type: BuildingType) -> Option<i32> {
        let upkeep = &self.config.upkeep;
        match building_type {
            BuildingType::Sweeper => Some(upkeep.sweeper_radius),
            BuildingType::ShieldGenerator | BuildingType::HeaterNode => {
                Some(upkeep.hazard_counter_radius)
            }
            _ => None,
        }
    }

    /// Positions of powered, working buildings of a type.
    pub(super) fn powered_positions(
        &self,
        building_type: BuildingType,
    ) -> Vec<crate::engine::GridPos> {
        let dust_response = self.resolved_dust_response();
        self.grid
            .find_buildings(building_type)
            .into_iter()
            .filter(|pos| {
                self.grid
                    .get(*pos)
                    .and_then(|tile| tile.building.as_ref())
                    .is_some_and(|building| {
                        building.powered
                            && !building.standby
                            && !building.is_dust_stalled_with(&dust_response)
                    })
            })
            .collect()
    }

    fn update_dust(&mut self, delta_time: f32) {
        let upkeep = self.config.upkeep.clone();
        let dust_rate = self.stats.apply(StatId::DustAccumulation, upkeep.dust_rate);
        // Acid eats the network specifically: a corroded conduit stalls, and a
        // stalled conduit stops carrying traffic, which is how a Venus run
        // fails rather than merely slowing.
        let acid_rate = upkeep.dust_rate * upkeep.acid_multiplier;
        let acid_strength = self.acid_strength();
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
        let acid_fields = crate::data::game_data()
            .planet(self.planet_index)
            .hazard_fields
            .clone();
        let grid_width = self.grid.width;
        let grid_height = self.grid.height;

        for (pos, tile) in self.grid.iter_tiles_mut() {
            let Some(building) = tile.building.as_mut() else {
                continue;
            };
            let mut rate = dust_rate;
            let mut acid = 0.0;
            if acid_rate > 0.0 && building.transmits_power() {
                let sheltered = shields
                    .iter()
                    .any(|shield| pos.distance(*shield) as i32 <= upkeep.hazard_counter_radius);
                acid = acid_rate
                    * super::progress::hazard_field_strength(
                        pos,
                        "acid",
                        acid_strength,
                        &acid_fields,
                        grid_width,
                        grid_height,
                    );
                if sheltered {
                    acid *= 1.0 - upkeep.hazard_counter_strength;
                }
            }

            if filter_positions
                .iter()
                .any(|filter_pos| pos.distance(*filter_pos) as i32 <= upkeep.filter_radius)
            {
                rate *= upkeep.filter_multiplier;
            }
            if cleared_forest_positions
                .iter()
                .any(|cleared_pos| pos.distance(*cleared_pos) as i32 <= upkeep.pollution_radius)
            {
                rate *= upkeep.pollution_multiplier;
            }
            rate *= building.dust_accumulation_multiplier();

            // Apply sweeper cleaning if nearby powered sweeper exists
            let mut clean_rate = 0.0;
            if powered_sweepers
                .iter()
                .any(|sweeper_pos| pos.distance(*sweeper_pos) as i32 <= upkeep.sweeper_radius)
            {
                clean_rate = upkeep.sweeper_rate;
            }

            building.dust =
                (building.dust + rate * delta_time - clean_rate * delta_time).clamp(0.0, 100.0);
            building.acid_wear = (building.acid_wear + acid * delta_time).clamp(0.0, 100.0);
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
            let data_produced = self.config.resources.core_data_rate;
            let data_consumed = self
                .research
                .current_research
                .as_ref()
                .map(|_| self.config.resources.research_rate)
                .unwrap_or(0.0);
            let observed = self.factory_flow_since_sample;
            let per_second = 1.0 / THROUGHPUT_SAMPLE_SECONDS;
            self.graph_samples.push(crate::state::GraphSample {
                power_produced: self.power_generation().max(0.0),
                power_consumed: self.power_consumption().max(0.0),
                minerals_consumed: observed.minerals_consumed * per_second,
                alloy_produced: observed.alloy_produced * per_second,
                alloy_consumed: observed.alloy_consumed * per_second,
                components_produced: observed.components_produced * per_second,
                components_consumed: observed.components_consumed * per_second,
                data_produced: (data_produced + observed.data_produced).max(0.0),
                data_consumed,
            });
            self.factory_flow_since_sample = crate::state::GraphSample::default();
            if self.graph_samples.len() > 120 {
                self.graph_samples.remove(0);
            }
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
        self.record_collapse_source();
        if let Some(source) = self.latest_collapse_source().map(str::to_owned) {
            self.notifications
                .danger(format!("{} - collapse engaged", source));
        }
        self.emit_audio(super::audio::AudioEvent::Collapse);
        let collapse = self.config.collapse.clone();
        // A bigger swarm takes longer to bring back up and loses more of what
        // it was holding. Twenty flat seconds stung hardest exactly when the
        // player could least afford it and stopped registering later.
        let scale = self.collapse_scale();
        let shutdown = self
            .stats
            .apply(
                StatId::CollapseShutdown,
                lerp(
                    collapse.min_shutdown_seconds,
                    collapse.max_shutdown_seconds,
                    scale,
                ),
            )
            .clamp(collapse.min_shutdown_seconds, collapse.max_shutdown_seconds);
        let loss = self
            .stats
            .apply(
                StatId::CollapseDataLoss,
                lerp(collapse.min_data_loss, collapse.max_data_loss, scale),
            )
            .clamp(0.0, 1.0);

        self.power_negative_seconds = 0.0;
        self.power_collapse_cooldown = collapse.cooldown_seconds;
        self.power_collapse_shutdown = shutdown;
        // Kept so the drones can sag over exactly as long as this one lasts.
        self.power_collapse_length = shutdown;
        self.research_lock_timer = shutdown * collapse.research_lock_ratio.max(0.0);
        self.collapse_notice_timer = collapse.notice_seconds;
        self.network_revision = self.network_revision.wrapping_add(1);

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

    fn record_collapse_source(&mut self) {
        let local = self.grid.iter_tiles().find_map(|(pos, tile)| {
            let building = tile.building.as_ref()?;
            (building.powered && (building.is_dust_stalled() || building.acid_wear >= 100.0))
                .then_some((pos, building.building_type))
        });
        let (source, building, position) = if let Some((position, building)) = local {
            (
                format!("Local failure: {}", building.name()),
                Some(building),
                Some(position),
            )
        } else {
            ("Broad grid collapse: power deficit".to_string(), None, None)
        };
        self.collapse_history.push(crate::state::CollapseRecord {
            source,
            building,
            position,
            world_time: self.time_played,
        });
        if self.collapse_history.len() > 32 {
            let keep_from = self.collapse_history.len() - 32;
            self.collapse_history.drain(..keep_from);
        }
    }

    pub fn latest_collapse_source(&self) -> Option<&str> {
        self.collapse_history
            .last()
            .map(|record| record.source.as_str())
    }
}

#[cfg(test)]
mod tests;
