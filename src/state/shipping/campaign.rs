//! Campaign-level standing orders and in-flight freight.

use crate::engine::GridPos;

use super::super::campaign::Campaign;
use super::super::game_state::PlanetState;
use super::{accepted_cargo, cargo_name, default_cargo, ExportOrder, Shipment};

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
        self.data().data().planet(index).pending_pod_cap.max(1)
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

    /// Point this world's drivers at the next colonized world, then at
    /// nothing, then round again. A world cannot throw at itself.
    pub fn cycle_export_target(&mut self) {
        self.cycle_export_target_for(self.current_index());
    }

    pub fn cycle_export_target_for(&mut self, world: usize) {
        let current = world;
        let mut candidates: Vec<usize> = (0..crate::state::campaign::PLANET_COUNT)
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

    /// Cycle the source-stock reserve for a standing order. The reserve and
    /// the legacy `surplus_only` flag move together so old saves and new UI
    /// edits keep the same dispatch rule.
    pub fn cycle_export_reserve_for(&mut self, world: usize) {
        let Some(order) = self.export_order_for(world) else {
            return;
        };
        let current = if order.surplus_only {
            order.reserve_source.max(0.0)
        } else {
            0.0
        };
        let next = super::EXPORT_RESERVE_CHOICES
            .iter()
            .position(|value| (*value - current).abs() < 0.01)
            .map(|index| {
                super::EXPORT_RESERVE_CHOICES[(index + 1) % super::EXPORT_RESERVE_CHOICES.len()]
            })
            .unwrap_or(super::EXPORT_RESERVE_CHOICES[1]);
        if let Some(planet) = self.planet_mut(world) {
            planet.export = Some(ExportOrder {
                surplus_only: next > 0.0,
                reserve_source: next,
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
        for planet in self.planets.iter_mut().flatten() {
            self.shipments.append(&mut planet.launched_pods);
        }
        self.refresh_shipment_pad_targets();

        if delta_time > 0.0 {
            for shipment in &mut self.shipments {
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
            let cap = self
                .data()
                .data()
                .planet(shipment.to)
                .pending_pod_cap
                .max(1);
            if pending_for_world >= cap {
                if let Some(held) = self.shipments.get_mut(index) {
                    held.overflow = true;
                }
                continue;
            }
            let origin_name = self.data().data().planet(shipment.from).name.clone();
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
                origin_name
            ));
            caught.push(index);
        }

        for index in caught.into_iter().rev() {
            self.shipments.remove(index);
        }
    }

    /// Apply a standing order's new pad choice to pods that are already on
    /// that route. A pod keeps its destination and cargo, but a pad that was
    /// demolished or powered down must not become a permanent orbiting grave.
    fn refresh_shipment_pad_targets(&mut self) {
        let route_pads: Vec<Option<(usize, Option<GridPos>)>> = self
            .planets
            .iter()
            .map(|planet| planet.as_ref().and_then(|planet| planet.export))
            .map(|order| order.map(|order| (order.target, order.target_pad)))
            .collect();
        for shipment in &mut self.shipments {
            // A pod already in flight keeps its destination, but a pad choice
            // belongs to the standing order that launched it. If that order
            // is retargeted or stopped, release the stale coordinate so an
            // available pad at the original destination can catch the pod.
            shipment.target_pad = route_pads
                .get(shipment.from)
                .and_then(|order| order.as_ref())
                .and_then(|(target, target_pad)| (*target == shipment.to).then_some(*target_pad))
                .flatten();
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
