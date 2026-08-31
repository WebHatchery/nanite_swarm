//! Mass Driver and Landing Pad behavior owned by one planet.

use crate::engine::{BuildingType, GridPos, ResourceType};

use super::super::game_state::PlanetState;
use super::{cargo_name, transit_seconds, Shipment};

impl PlanetState {
    /// Every Mass Driver standing on this world, whether it currently has
    /// power, is on standby, or is stalled by dust.
    pub fn mass_drivers_built(&self) -> usize {
        self.grid.find_buildings(BuildingType::MassDriver).len()
    }

    /// Powered Mass Drivers standing on this world.
    pub fn mass_drivers_online(&self) -> usize {
        self.powered_positions(BuildingType::MassDriver).len()
    }

    /// Landing Pads standing on this world, including pads that have lost
    /// power and therefore cannot catch a pod yet.
    pub fn landing_pads_built(&self) -> usize {
        self.grid.find_buildings(BuildingType::LandingPad).len()
    }

    /// Powered Landing Pads standing on this world. A world with none of these
    /// cannot be shipped to, however many drivers are pointed at it.
    pub fn landing_pads_online(&self) -> usize {
        self.powered_positions(BuildingType::LandingPad).len()
    }

    pub fn landing_pad_positions(&self) -> Vec<GridPos> {
        self.powered_positions(BuildingType::LandingPad)
    }

    /// Unload a pod onto the emptiest pad with room for it, and say whether
    /// anything caught it.
    ///
    /// A pad holds one cargo at a time, because what lands on it goes on the
    /// pad exactly as a drill's ore goes on its own — one pile, one resource,
    /// one crew to carry it off. A pod that finds nowhere to land stays up.
    pub(super) fn accept_pod(
        &mut self,
        cargo: ResourceType,
        amount: f32,
        target_pad: Option<GridPos>,
    ) -> bool {
        let capacity = self.config.mass_driver.pad_capacity;
        let mut best: Option<(GridPos, f32)> = None;
        for pos in self.powered_positions(BuildingType::LandingPad) {
            if target_pad.is_some_and(|target| target != pos) {
                continue;
            }
            let key = (pos.x, pos.y);
            let held = self.output_buffers.get(&key).copied().unwrap_or(0.0);
            // A pad already piled with something else is not a place to put
            // this, however much room is left on it.
            if held > 0.0 && self.pad_cargo.get(&key) != Some(&cargo) {
                continue;
            }
            if held + amount > capacity {
                continue;
            }
            if best.is_none_or(|(_, most)| held < most) {
                best = Some((pos, held));
            }
        }

        let Some((pos, _)) = best else {
            return false;
        };
        let key = (pos.x, pos.y);
        *self.output_buffers.entry(key).or_insert(0.0) += amount;
        self.pad_cargo.insert(key, cargo);
        true
    }

    /// What a pad is holding, for the inspector.
    pub fn pad_summary(&self, pos: GridPos) -> String {
        let key = (pos.x, pos.y);
        let held = self.output_buffers.get(&key).copied().unwrap_or(0.0);
        if held <= 0.0 {
            return "Empty - waiting on a pod".to_string();
        }
        let cargo = self
            .pad_cargo
            .get(&key)
            .copied()
            .unwrap_or(ResourceType::Minerals);
        format!(
            "{:.0}/{:.0} {}",
            held,
            self.config.mass_driver.pad_capacity,
            cargo_name(cargo)
        )
    }

    /// How full the fullest pod on the world is, as a share of a whole one.
    /// One number for a bank of drivers, because the map shows one bar.
    pub fn pod_fraction(&self) -> f32 {
        let capacity = self.config.mass_driver.pod_capacity.max(0.001);
        self.pod_loads
            .values()
            .fold(0.0f32, |best, loaded| best.max(*loaded))
            / capacity
    }

    /// The nominal cargo rate for this world's standing order, in units per
    /// minute. Each powered driver fills its own pod; the schedule is a pause
    /// after a full pod leaves, so it lowers the shared route rate without
    /// changing the load speed. This is a planning number, not a promise that
    /// the source has cargo or that the destination has a free pad.
    pub fn export_rate_per_minute(&self) -> f32 {
        let Some(order) = self.export else {
            return 0.0;
        };
        let drivers = self.mass_drivers_online() as f32;
        let load_rate = self.config.mass_driver.load_rate;
        let pod_capacity = self.config.mass_driver.pod_capacity;
        if drivers <= 0.0 || load_rate <= 0.0 || pod_capacity <= 0.0 {
            return 0.0;
        }
        let fill_seconds = pod_capacity / load_rate;
        let cycle_seconds = fill_seconds + order.schedule_seconds.max(0.0);
        if cycle_seconds <= 0.0 || !cycle_seconds.is_finite() {
            return 0.0;
        }
        (drivers * pod_capacity / cycle_seconds * 60.0).max(0.0)
    }

    /// What one driver is doing, for the inspector: its route and its pod.
    pub fn export_summary(&self, pos: GridPos) -> String {
        let Some(order) = self.export else {
            return "No route - set one on the Solar System map".to_string();
        };
        let loaded = self.pod_loads.get(&(pos.x, pos.y)).copied().unwrap_or(0.0);
        format!(
            "{} -> {} {:.0}/{:.0}",
            cargo_name(order.cargo),
            self.data().planet(order.target).name,
            loaded,
            self.config.mass_driver.pod_capacity
        )
    }

    /// Load cargo credited by offline catch-up into the standing order.
    ///
    /// Offline production is intentionally aggregated rather than replaying
    /// every drone trip. Only the resources produced by that catch-up window
    /// are eligible here, so an order does not silently drain an old
    /// stockpile just because the player was away. The source reserve still
    /// applies, and full pods enter the same campaign shipment queue as live
    /// launches.
    pub(in crate::state) fn apply_offline_export_progress(&mut self) -> u32 {
        let Some(order) = self.export else {
            return 0;
        };
        if order.target == self.planet_index
            || self.last_offline_simulated <= 0.0
            || !self.last_offline_simulated.is_finite()
        {
            return 0;
        }

        let elapsed = self.last_offline_simulated;
        let effective_elapsed = if self.export_cooldown > 0.0 {
            if elapsed <= self.export_cooldown {
                self.export_cooldown -= elapsed;
                return 0;
            }
            let effective = elapsed - self.export_cooldown;
            self.export_cooldown = 0.0;
            effective
        } else {
            elapsed
        };

        let drivers = self.powered_positions(BuildingType::MassDriver);
        let load_rate = self.config.mass_driver.load_rate;
        let capacity = self.config.mass_driver.pod_capacity;
        if drivers.is_empty()
            || !load_rate.is_finite()
            || !capacity.is_finite()
            || load_rate <= 0.0
            || capacity <= 0.0
        {
            return 0;
        }

        let cargo_gained = self.offline_cargo_gained(order.cargo);
        if cargo_gained <= 0.0 || !cargo_gained.is_finite() {
            return 0;
        }

        let schedule = order.schedule_seconds.max(0.0);
        let possible: Vec<(GridPos, f32)> = drivers
            .iter()
            .map(|pos| {
                let loaded = self.pod_loads.get(&(pos.x, pos.y)).copied().unwrap_or(0.0);
                (
                    *pos,
                    offline_driver_load(effective_elapsed, loaded, capacity, load_rate, schedule),
                )
            })
            .collect();
        let possible_total: f32 = possible.iter().map(|(_, amount)| *amount).sum();
        let reserve = if order.surplus_only {
            order.reserve_source.max(0.0)
        } else {
            0.0
        };
        let available = (self.resources.get(order.cargo) - reserve).max(0.0);
        let mut remaining = cargo_gained.min(available).min(possible_total);
        if remaining <= 0.0 || !remaining.is_finite() {
            return 0;
        }

        let transit = transit_seconds(
            self.data(),
            self.planet_index,
            order.target,
            &self.config.mass_driver,
        );
        let mut launched = 0;
        for (pos, limit) in possible {
            let amount = remaining.min(limit);
            remaining -= amount;
            let key = (pos.x, pos.y);
            let mut loaded = self.pod_loads.get(&key).copied().unwrap_or(0.0) + amount;
            while loaded + 0.001 >= capacity {
                loaded = (loaded - capacity).max(0.0);
                self.launched_pods.push(Shipment {
                    from: self.planet_index,
                    to: order.target,
                    cargo: order.cargo,
                    amount: capacity,
                    remaining: transit,
                    transit,
                    target_pad: order.target_pad,
                    overflow: false,
                    priority: order.priority,
                });
                launched += 1;
            }
            self.pod_loads.insert(key, loaded);
            if remaining <= 0.001 {
                break;
            }
        }

        let loaded = cargo_gained.min(available).min(possible_total) - remaining;
        self.resources.add(order.cargo, -loaded.max(0.0));
        if launched > 0 {
            self.export_cooldown = schedule;
        }
        launched
    }

    fn offline_cargo_gained(&self, cargo: ResourceType) -> f32 {
        match cargo {
            ResourceType::Minerals => self.last_offline_report.minerals_gained,
            ResourceType::Alloy => self.last_offline_report.alloy_gained,
            ResourceType::Components => self.last_offline_report.components_gained,
            _ => 0.0,
        }
    }

    /// Pull what has been carried to each driver into its pod, and throw the
    /// pods that filled up this step.
    pub(in crate::state) fn update_exports(&mut self, delta_time: f32) {
        if self.export_cooldown > 0.0 {
            self.export_cooldown = (self.export_cooldown - delta_time).max(0.0);
            return;
        }
        let Some(order) = self.export else {
            return;
        };
        if order.target == self.planet_index {
            return;
        }
        let config = self.config.mass_driver.clone();
        let capacity = config.pod_capacity.max(0.001);
        let transit = transit_seconds(self.data(), self.planet_index, order.target, &config);

        for pos in self.powered_positions(BuildingType::MassDriver) {
            let key = (pos.x, pos.y);
            // The total buffer is retained for old saves and readable
            // summaries, but a standing order must draw from its typed
            // hopper. Otherwise changing Minerals to Alloy halfway through
            // a load would silently relabel cargo already on the pad.
            let hopper = match self.input_hoppers.get(&key) {
                Some(hoppers) => hoppers.get(&order.cargo).copied().unwrap_or(0.0),
                None => self.input_buffers.get(&key).copied().unwrap_or(0.0),
            };
            if order.surplus_only && self.resources.get(order.cargo) <= order.reserve_source {
                continue;
            }
            let pulled = (config.load_rate * delta_time).min(hopper);
            if pulled <= 0.0 {
                continue;
            }
            if let Some(buffer) = self.input_buffers.get_mut(&key) {
                *buffer = (*buffer - pulled).max(0.0);
            }
            if let Some(buffer) = self
                .input_hoppers
                .get_mut(&key)
                .and_then(|hoppers| hoppers.get_mut(&order.cargo))
            {
                *buffer = (*buffer - pulled).max(0.0);
            }
            let loaded = self.pod_loads.entry(key).or_insert(0.0);
            *loaded += pulled;
            // Fixed-step accumulation can leave a nominally full pod a few
            // millionths short of its capacity. Treat that as full, then
            // clamp the remainder so the drift cannot become negative cargo.
            while *loaded + 0.001 >= capacity {
                *loaded = (*loaded - capacity).max(0.0);
                self.launched_pods.push(Shipment {
                    from: self.planet_index,
                    to: order.target,
                    cargo: order.cargo,
                    amount: capacity,
                    remaining: transit,
                    transit,
                    target_pad: order.target_pad,
                    overflow: false,
                    priority: order.priority,
                });
                self.export_cooldown = order.schedule_seconds.max(0.0);
            }
        }
    }
}

fn offline_driver_load(
    elapsed: f32,
    loaded: f32,
    capacity: f32,
    load_rate: f32,
    schedule: f32,
) -> f32 {
    let loaded = loaded.clamp(0.0, capacity);
    let fill_seconds = capacity / load_rate;
    let first_fill = (capacity - loaded) / load_rate;
    if elapsed <= first_fill {
        return (elapsed * load_rate).max(0.0);
    }

    let mut possible = capacity - loaded;
    let remaining = elapsed - first_fill;
    let cycle = fill_seconds + schedule;
    if cycle <= 0.0 || !cycle.is_finite() {
        return possible.max(0.0);
    }

    // The first full pod has already been accounted for above. Every later
    // cycle is a pause followed by a fill, so subtract the pause once before
    // counting complete cycles. This also handles a zero-second schedule
    // without losing the partial fill after the first pod.
    let after_pause = (remaining - schedule).max(0.0);
    let full_cycles = (after_pause / cycle).floor();
    possible += full_cycles * capacity;
    let remainder = after_pause - full_cycles * cycle;
    possible += remainder.min(fill_seconds) * load_rate;
    possible.max(0.0)
}
