//! Per-tick simulation: drills, servers, dust, biomass, tutorial, power collapse

#[cfg(test)]
use crate::engine::{BuildingType, TerrainType};
use crate::engine::{DroneState, StatId};

use super::game_state::PlanetState;

mod collapse;
mod recipes;
mod upkeep;

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
}

#[cfg(test)]
mod tests;
