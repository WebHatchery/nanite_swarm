//! Mass Drivers: what a world throws at another world, and what is in flight.
//!
//! The driver is a logistics consumer like any other. Drones carry ore to its
//! hopper over the conduit network, it pulls that into a pod, and when the pod
//! is full it throws it. Nothing teleports: a world that has not routed
//! anything to its driver exports nothing, however large its stockpile.
//!
//! The order (what to load, where to throw it) belongs to the world, not to
//! one driver, and it survives being left behind — that is the whole point. A
//! world the swarm walked away from keeps feeding the world it walked to.

use serde::{Deserialize, Serialize};

use crate::data::MassDriverConfig;
use crate::engine::{BuildingType, GridPos, ResourceType};

use super::campaign::Campaign;
use super::game_state::PlanetState;

/// A standing order for a world: what its Mass Drivers load, and where they
/// throw it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportOrder {
    pub target: usize,
    pub cargo: ResourceType,
}

/// A pod that has left the ground and not yet landed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipment {
    pub from: usize,
    pub to: usize,
    pub cargo: ResourceType,
    pub amount: f32,
    /// World seconds still to fly, out of the [`Shipment::transit`] it started
    /// with, so the map can draw how far along it is.
    pub remaining: f32,
    pub transit: f32,
}

impl Shipment {
    /// How much of the flight is behind it, from zero at launch to one on
    /// landing.
    pub fn progress(&self) -> f32 {
        if self.transit <= 0.0 {
            return 1.0;
        }
        (1.0 - self.remaining / self.transit).clamp(0.0, 1.0)
    }
}

/// How long a throw between two worlds takes, off their orbits.
pub fn transit_seconds(from: usize, to: usize, config: &MassDriverConfig) -> f32 {
    let data = crate::data::game_data();
    let gap = (data.planet(from).orbit_radius - data.planet(to).orbit_radius).abs();
    (gap * config.seconds_per_orbit_unit).max(config.min_transit_seconds)
}

impl PlanetState {
    /// Powered Mass Drivers standing on this world.
    pub fn mass_drivers_online(&self) -> usize {
        self.powered_positions(BuildingType::MassDriver).len()
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

    /// What one driver is doing, for the inspector: its route and its pod.
    pub fn export_summary(&self, pos: GridPos) -> String {
        let Some(order) = self.export else {
            return "No route - set one on the map (M)".to_string();
        };
        let loaded = self.pod_loads.get(&(pos.x, pos.y)).copied().unwrap_or(0.0);
        format!(
            "{} -> {} {:.0}/{:.0}",
            cargo_name(order.cargo),
            crate::data::game_data().planet(order.target).name,
            loaded,
            self.config.mass_driver.pod_capacity
        )
    }

    /// Pull what has been carried to each driver into its pod, and throw the
    /// pods that filled up this step.
    pub(super) fn update_exports(&mut self, delta_time: f32) {
        let Some(order) = self.export else {
            return;
        };
        if order.target == self.planet_index {
            return;
        }
        let config = self.config.mass_driver.clone();
        let capacity = config.pod_capacity.max(0.001);
        let transit = transit_seconds(self.planet_index, order.target, &config);

        for pos in self.powered_positions(BuildingType::MassDriver) {
            let key = (pos.x, pos.y);
            let hopper = self.input_buffers.get(&key).copied().unwrap_or(0.0);
            let pulled = (config.load_rate * delta_time).min(hopper);
            if pulled <= 0.0 {
                continue;
            }
            if let Some(buffer) = self.input_buffers.get_mut(&key) {
                *buffer = (*buffer - pulled).max(0.0);
            }
            let loaded = self.pod_loads.entry(key).or_insert(0.0);
            *loaded += pulled;
            while *loaded >= capacity {
                *loaded -= capacity;
                self.launched_pods.push(Shipment {
                    from: self.planet_index,
                    to: order.target,
                    cargo: order.cargo,
                    amount: capacity,
                    remaining: transit,
                    transit,
                });
            }
        }
    }
}

impl Campaign {
    /// The standing order of the world in front of the player.
    pub fn export_order(&self) -> Option<ExportOrder> {
        self.current().export
    }

    /// Everything in flight anywhere in the system.
    pub fn shipments(&self) -> &[Shipment] {
        &self.shipments
    }

    /// Point this world's drivers at the next colonized world, then at nothing,
    /// then round again. A world cannot throw at itself.
    pub fn cycle_export_target(&mut self) {
        let current = self.current_index();
        let mut candidates: Vec<usize> = (0..super::campaign::PLANET_COUNT)
            .filter(|index| *index != current && self.is_colonized(*index))
            .collect();
        if candidates.is_empty() {
            self.current_mut().export = None;
            return;
        }
        candidates.sort_unstable();

        let cargo = self
            .current()
            .export
            .map(|order| order.cargo)
            .unwrap_or_else(|| default_cargo(&self.current().config.mass_driver));
        let next = match self.current().export.map(|order| order.target) {
            None => Some(candidates[0]),
            Some(target) => match candidates.iter().position(|index| *index == target) {
                // Off the end of the list is "no route", so the player can
                // stop shipping without demolishing anything.
                Some(slot) => candidates.get(slot + 1).copied(),
                None => Some(candidates[0]),
            },
        };
        self.current_mut().export = next.map(|target| ExportOrder { target, cargo });
    }

    /// Load the next cargo the drivers accept. Does nothing until a world has
    /// somewhere to throw to, because a cargo with no destination is not an
    /// order.
    pub fn cycle_export_cargo(&mut self) {
        let Some(order) = self.current().export else {
            return;
        };
        let accepted = accepted_cargo(&self.current().config.mass_driver);
        if accepted.is_empty() {
            return;
        }
        let slot = accepted
            .iter()
            .position(|cargo| *cargo == order.cargo)
            .map(|slot| (slot + 1) % accepted.len())
            .unwrap_or(0);
        self.current_mut().export = Some(ExportOrder {
            cargo: accepted[slot],
            ..order
        });
    }

    /// Collect what the worlds threw this step, fly everything already up, and
    /// unload whatever landed.
    ///
    /// Transit is ticked on the same world time the foreground planet
    /// simulated, so a pod does not stall while the player reads the map and
    /// does not sprint while they fast-forward past what it cost to load it.
    pub fn update_shipments(&mut self, delta_time: f32) {
        let shipments = &mut self.shipments;
        for planet in self.planets.iter_mut().flatten() {
            shipments.append(&mut planet.launched_pods);
        }

        if delta_time > 0.0 {
            for shipment in shipments.iter_mut() {
                shipment.remaining = (shipment.remaining - delta_time).max(0.0);
            }
        }

        let mut landed = Vec::new();
        self.shipments.retain(|shipment| {
            if shipment.remaining > 0.0 {
                return true;
            }
            landed.push(shipment.clone());
            false
        });

        for shipment in landed {
            let Some(planet) = self.planet_mut(shipment.to) else {
                // The only world that can vanish is one that never existed.
                continue;
            };
            planet.resources.add(shipment.cargo, shipment.amount);
            planet.notifications.success(format!(
                "{:.0} {} landed from {}",
                shipment.amount,
                cargo_name(shipment.cargo),
                crate::data::game_data().planet(shipment.from).name
            ));
        }
    }
}

/// The first cargo a driver accepts, for a route that has just been set.
fn default_cargo(config: &MassDriverConfig) -> ResourceType {
    accepted_cargo(config)
        .first()
        .copied()
        .unwrap_or(ResourceType::Minerals)
}

/// What the drivers will throw, in the order the map cycles them. Anything the
/// engine does not recognise, or that no drone could carry there, is dropped.
fn accepted_cargo(config: &MassDriverConfig) -> Vec<ResourceType> {
    config
        .cargo
        .iter()
        .filter_map(|id| ResourceType::from_id(id))
        .filter(|cargo| cargo.is_physical())
        .collect()
}

/// A resource as the player reads it on a shipping label.
pub fn cargo_name(cargo: ResourceType) -> &'static str {
    match cargo {
        ResourceType::Minerals => "Minerals",
        ResourceType::Energy => "Energy",
        ResourceType::Data => "Data",
        ResourceType::Biomass => "Biomass",
        ResourceType::Alloy => "Alloy",
    }
}

#[cfg(test)]
mod tests;
