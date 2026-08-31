//! Logistics: drill dispatch, and keeping drones on an intact network route.
//!
//! Every delivery walks the conduit network (see `engine::routing`). A drone
//! whose route is cut stops where it stands and waves an error flag; it goes
//! back to work by itself once the network is whole again.

use crate::engine::{
    route_over_network_weighted_with, BuildingType, DroneEvent, DroneState, GridPos, ResourceType,
    StatId,
};

use super::game_state::PlanetState;

mod producers;
mod routes;

type DeliveryChoice = ((bool, i32, usize, i32, i32), GridPos, Vec<GridPos>);

impl PlanetState {
    /// Run one logistics tick: re-check live routes, then dispatch drills.
    pub(super) fn update_logistics(&mut self, delta_time: f32, allow_visuals: bool) {
        let Some(core) = self.grid.find_core() else {
            return;
        };

        for event in self.resolve_routes(core) {
            if let DroneEvent::PathBlocked { drone_id } = event {
                if allow_visuals {
                    if let Some(pos) = self.drone_position(drone_id) {
                        self.spawn_route_break_burst(pos);
                    }
                }
            }
        }

        self.update_drills(delta_time);
        self.dispatch_producers(core);
        self.update_traffic();
        for remaining in self.route_reservations.values_mut() {
            *remaining = (*remaining - delta_time).max(0.0);
        }
        self.route_reservations
            .retain(|_, remaining| *remaining > 0.0);
    }

    /// Number of drones currently stalled on a broken route.
    pub fn stalled_drone_count(&self) -> usize {
        self.drones.count_by_state(DroneState::Error)
    }

    /// How many drones a drill keeps in service, after research.
    pub fn drone_crew_size(&self) -> usize {
        self.stats
            .apply(
                StatId::DronesPerDrill,
                self.config.resources.drones_per_drill,
            )
            .round()
            .max(1.0) as usize
    }

    /// Where a load of ore from `from` should go, and how to get there.
    ///
    /// A processing building with room in its hopper takes priority over the
    /// Core. Lean hoppers are served before distance breaks the tie, including
    /// loads already in flight, so parallel processors share supply instead of
    /// one nearby machine quietly monopolising it.
    pub(crate) fn delivery_for(
        &self,
        from: GridPos,
        core: GridPos,
        resource: ResourceType,
    ) -> Option<(GridPos, Vec<GridPos>)> {
        let hopper_ceiling = self.processor_pad_capacity();
        let dust_response = self.resolved_dust_response();
        let mut best: Option<DeliveryChoice> = None;

        for pos in self.consumers_of(resource) {
            let delivered = self
                .input_hoppers
                .get(&(pos.x, pos.y))
                .and_then(|hopper| hopper.get(&resource))
                .copied()
                .unwrap_or(0.0);
            let inbound: f32 = self
                .drones
                .drones()
                .iter()
                .filter(|drone| {
                    drone.target == pos
                        && drone.resource_type == resource
                        && drone.carrying > 0.0
                        && drone.state == DroneState::MovingToCore
                })
                .map(|drone| drone.carrying)
                .sum();
            let committed = delivered + inbound;
            if committed + self.drones.drone_capacity > hopper_ceiling {
                continue;
            }
            let Some(route) = route_over_network_weighted_with(
                &self.grid,
                from,
                pos,
                self.route_cost(),
                &dust_response,
            ) else {
                continue;
            };
            let consumption = self.consumer_rate(pos, resource).max(0.001);
            let demand_band = (committed / consumption).floor() as i32;
            let priority = self
                .grid
                .get(pos)
                .and_then(|tile| tile.building.as_ref())
                .is_some_and(|building| building.input_priority);
            // `false` sorts before `true`: a priority line gets first claim,
            // while demand and distance still balance peers in the same tier.
            let key = (!priority, demand_band, route.len(), pos.y, pos.x);
            if best.as_ref().is_none_or(|(best_key, _, _)| key < *best_key) {
                best = Some((key, pos, route));
            }
        }

        best.map(|(_, pos, route)| (pos, route)).or_else(|| {
            route_over_network_weighted_with(
                &self.grid,
                from,
                core,
                self.route_cost(),
                &dust_response,
            )
            .map(|route| (core, route))
        })
    }

    fn consumer_rate(&self, pos: GridPos, resource: ResourceType) -> f32 {
        let Some(kind) = self
            .grid
            .get(pos)
            .and_then(|tile| tile.building.as_ref())
            .map(|building| building.building_type)
        else {
            return 1.0;
        };
        self.data()
            .building(kind.id())
            .recipe
            .inputs
            .get(resource.id())
            .copied()
            .unwrap_or(1.0)
            .max(0.001)
    }

    /// Powered buildings whose recipe eats ore.
    /// Powered buildings whose hopper wants this resource carried to it.
    ///
    /// A drone looks for one of these before falling back to the Core, which
    /// is what makes a Smelter parked beside the drills get fed first.
    fn consumers_of(&self, resource: ResourceType) -> Vec<GridPos> {
        let mut consumers: Vec<GridPos> = self
            .data()
            .buildings
            .iter()
            .filter(|def| {
                def.recipe.carried_ids().contains(&resource.id())
                    && def.recipe.inputs.get(resource.id()).copied().unwrap_or(0.0) > 0.0
            })
            .filter_map(|def| BuildingType::from_id(&def.id))
            .flat_map(|kind| self.powered_positions(kind))
            .collect();

        // A Mass Driver wants whatever this world's standing order says it is
        // throwing, and nothing at all while there is nowhere to throw it. It
        // is fed over the network like any other consumer: an export is a
        // logistics run that happens to end in orbit.
        if self.export.is_some_and(|order| {
            order.cargo == resource
                && order.target != self.planet_index
                && self.export_can_claim_surplus(order)
        }) {
            consumers.extend(self.powered_positions(BuildingType::MassDriver));
        }

        consumers
    }

    /// A surplus order only lets a Mass Driver claim new cargo while the Core
    /// stockpile is above its protected floor. Keeping the check here as well
    /// as in `update_exports` matters for refined goods: Alloy and Components
    /// can otherwise be routed straight from a processor to the driver and
    /// never pass through the stockpile that the order is meant to protect.
    fn export_can_claim_surplus(&self, order: crate::state::shipping::ExportOrder) -> bool {
        !order.surplus_only || self.resources.get(order.cargo) > order.reserve_source
    }
}

#[cfg(test)]
mod tests;
