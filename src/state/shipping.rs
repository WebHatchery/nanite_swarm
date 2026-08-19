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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExportOrder {
    pub target: usize,
    pub cargo: ResourceType,
    #[serde(default)]
    pub target_pad: Option<GridPos>,
    #[serde(default)]
    pub schedule_seconds: f32,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub reserve_source: f32,
    #[serde(default)]
    pub surplus_only: bool,
}

impl Default for ExportOrder {
    fn default() -> Self {
        Self {
            target: 0,
            cargo: ResourceType::Minerals,
            target_pad: None,
            schedule_seconds: 0.0,
            priority: 0,
            reserve_source: 0.0,
            surplus_only: false,
        }
    }
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
    #[serde(default)]
    pub target_pad: Option<GridPos>,
    #[serde(default)]
    pub overflow: bool,
    /// Copied from the standing order so simultaneous arrivals can be
    /// resolved in a stable, player-visible order.
    #[serde(default)]
    pub priority: u8,
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

    /// Arrived, and circling: the destination has no pad with room on it. The
    /// cargo is not lost — it lands the moment somewhere can take it.
    pub fn is_holding(&self) -> bool {
        self.remaining <= 0.0
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
        let transit = transit_seconds(self.planet_index, order.target, &config);

        for pos in self.powered_positions(BuildingType::MassDriver) {
            let key = (pos.x, pos.y);
            let hopper = self.input_buffers.get(&key).copied().unwrap_or(0.0);
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
                    target_pad: order.target_pad,
                    overflow: false,
                    priority: order.priority,
                });
                self.export_cooldown = order.schedule_seconds.max(0.0);
            }
        }
    }
}

impl Campaign {
    /// The standing order of the world in front of the player.
    pub fn export_order(&self) -> Option<ExportOrder> {
        self.current().export
    }

    pub fn export_order_for(&self, index: usize) -> Option<ExportOrder> {
        self.planet(index).and_then(|planet| planet.export)
    }

    pub fn pending_pod_count(&self, index: usize) -> usize {
        self.shipments
            .iter()
            .filter(|shipment| shipment.to == index && shipment.is_holding())
            .count()
    }

    pub fn pending_pod_cap(&self, index: usize) -> usize {
        crate::data::game_data()
            .planet(index)
            .pending_pod_cap
            .max(1)
    }

    pub fn overflow_pod_count(&self, index: usize) -> usize {
        self.shipments
            .iter()
            .filter(|shipment| shipment.to == index && shipment.overflow)
            .count()
    }

    /// Everything in flight anywhere in the system.
    pub fn shipments(&self) -> &[Shipment] {
        &self.shipments
    }

    /// Point this world's drivers at the next colonized world, then at nothing,
    /// then round again. A world cannot throw at itself.
    pub fn cycle_export_target(&mut self) {
        self.cycle_export_target_for(self.current_index());
    }

    pub fn cycle_export_target_for(&mut self, world: usize) {
        let current = world;
        let mut candidates: Vec<usize> = (0..super::campaign::PLANET_COUNT)
            .filter(|index| *index != current && self.is_colonized(*index))
            .collect();
        if candidates.is_empty() {
            if let Some(planet) = self.planet_mut(world) {
                planet.export = None;
            }
            return;
        }
        candidates.sort_unstable();

        let Some(home) = self.planet(world) else {
            return;
        };
        let cargo = home
            .export
            .map(|order| order.cargo)
            .unwrap_or_else(|| default_cargo(&home.config.mass_driver));
        let next = match home.export.map(|order| order.target) {
            None => Some(candidates[0]),
            Some(target) => match candidates.iter().position(|index| *index == target) {
                // Off the end of the list is "no route", so the player can
                // stop shipping without demolishing anything.
                Some(slot) => candidates.get(slot + 1).copied(),
                None => Some(candidates[0]),
            },
        };
        if let Some(planet) = self.planet_mut(world) {
            planet.export = next.map(|target| ExportOrder {
                target,
                cargo,
                ..ExportOrder::default()
            });
        }
    }

    /// Load the next cargo the drivers accept. Does nothing until a world has
    /// somewhere to throw to, because a cargo with no destination is not an
    /// order.
    pub fn cycle_export_cargo(&mut self) {
        self.cycle_export_cargo_for(self.current_index());
    }

    pub fn cycle_export_cargo_for(&mut self, world: usize) {
        let Some(order) = self.export_order_for(world) else {
            return;
        };
        let Some(home) = self.planet(world) else {
            return;
        };
        let accepted = accepted_cargo(&home.config.mass_driver);
        if accepted.is_empty() {
            return;
        }
        let slot = accepted
            .iter()
            .position(|cargo| *cargo == order.cargo)
            .map(|slot| (slot + 1) % accepted.len())
            .unwrap_or(0);
        if let Some(planet) = self.planet_mut(world) {
            planet.export = Some(ExportOrder {
                cargo: accepted[slot],
                ..order
            });
        }
    }

    pub fn cycle_export_pad_for(&mut self, world: usize) {
        let Some(order) = self.export_order_for(world) else {
            return;
        };
        let pads = self
            .planet(order.target)
            .map(PlanetState::landing_pad_positions)
            .unwrap_or_default();
        let next = match order.target_pad {
            None => pads.first().copied(),
            Some(current) => pads
                .iter()
                .position(|pad| *pad == current)
                .and_then(|index| pads.get(index + 1).copied()),
        };
        if let Some(planet) = self.planet_mut(world) {
            planet.export = Some(ExportOrder {
                target_pad: next,
                ..order
            });
        }
    }

    pub fn cycle_export_schedule_for(&mut self, world: usize) {
        let Some(order) = self.export_order_for(world) else {
            return;
        };
        let choices = [0.0, 2.0, 5.0, 10.0, 30.0];
        let next = choices
            .iter()
            .position(|value| (*value - order.schedule_seconds).abs() < 0.01)
            .map(|index| choices[(index + 1) % choices.len()])
            .unwrap_or(0.0);
        if let Some(planet) = self.planet_mut(world) {
            planet.export = Some(ExportOrder {
                schedule_seconds: next,
                ..order
            });
        }
    }

    pub fn cycle_export_priority_for(&mut self, world: usize) {
        let Some(order) = self.export_order_for(world) else {
            return;
        };
        if let Some(planet) = self.planet_mut(world) {
            planet.export = Some(ExportOrder {
                priority: (order.priority + 1) % 4,
                ..order
            });
        }
    }

    pub fn toggle_export_surplus_for(&mut self, world: usize) {
        let Some(order) = self.export_order_for(world) else {
            return;
        };
        if let Some(planet) = self.planet_mut(world) {
            planet.export = Some(ExportOrder {
                surplus_only: !order.surplus_only,
                reserve_source: if order.surplus_only {
                    order.reserve_source
                } else if order.reserve_source > 0.0 {
                    order.reserve_source
                } else {
                    100.0
                },
                ..order
            });
        }
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

        // A pod that has arrived only comes down if something on the ground can
        // catch it. Anything else stays in the list holding, and tries again
        // next step: cargo circling overhead is recoverable, cargo deleted for
        // want of a pad is not.
        let arrived: Vec<usize> = self
            .shipments
            .iter()
            .enumerate()
            .filter(|(_, shipment)| shipment.is_holding())
            .map(|(index, _)| index)
            .collect();

        let mut arrived = arrived;
        // Higher-priority orders land first. All remaining fields are stable
        // tie-breakers, so equal orders behave identically after a reload.
        arrived.sort_by_key(|index| {
            let shipment = &self.shipments[*index];
            (
                std::cmp::Reverse(shipment.priority),
                shipment.from,
                shipment.to,
                shipment.cargo.id(),
            )
        });
        let mut caught = Vec::new();
        for index in arrived {
            let shipment = self.shipments[index].clone();
            if self.planet(shipment.to).is_none() {
                // The only world that can vanish is one that never existed.
                caught.push(index);
                continue;
            }
            let pending_for_world = self
                .shipments
                .iter()
                .enumerate()
                .filter(|(other_index, other)| {
                    *other_index != index && other.to == shipment.to && other.is_holding()
                })
                .count();
            let cap = crate::data::game_data()
                .planet(shipment.to)
                .pending_pod_cap
                .max(1);
            if pending_for_world >= cap {
                if let Some(held) = self.shipments.get_mut(index) {
                    held.overflow = true;
                }
                continue;
            }
            let Some(planet) = self.planet_mut(shipment.to) else {
                continue;
            };
            if !planet.accept_pod(shipment.cargo, shipment.amount, shipment.target_pad) {
                continue;
            }
            planet.notifications.success(format!(
                "{:.0} {} landed from {}",
                shipment.amount,
                cargo_name(shipment.cargo),
                crate::data::game_data().planet(shipment.from).name
            ));
            caught.push(index);
        }

        for index in caught.into_iter().rev() {
            self.shipments.remove(index);
        }
    }

    /// Pods that have arrived and found nowhere to land.
    pub fn holding_shipments(&self) -> usize {
        self.shipments
            .iter()
            .filter(|shipment| shipment.is_holding())
            .count()
    }

    pub fn overflow_shipments(&self) -> usize {
        self.shipments
            .iter()
            .filter(|shipment| shipment.overflow)
            .count()
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
        ResourceType::Components => "Components",
    }
}

#[cfg(test)]
mod tests;
