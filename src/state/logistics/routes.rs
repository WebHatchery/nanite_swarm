//! Network routing, recovery, and congestion for planetary logistics.
//!
//! These helpers keep the physical conduit graph honest: drones stop when a
//! route is cut, recover when it is restored, and share saturated tiles rather
//! than pretending every path has infinite capacity.

use crate::data::DustResponseConfig;
use crate::engine::{
    route_over_network_weighted_with, tile_carries_traffic_with, traffic_cost, BuildingType, Drone,
    DroneEvent, DroneState, Grid, GridPos,
};
use std::collections::HashMap;

use super::PlanetState;

impl PlanetState {
    /// Every piece of network with no unbroken run back to the Core, plus the
    /// pieces that have stopped carrying traffic and caused it.
    pub fn severed_network(&self) -> Vec<GridPos> {
        let Some(core) = self.grid.find_core() else {
            return Vec::new();
        };
        let dust_response = self.resolved_dust_response();

        // A piece of network regardless of whether it is working right now,
        // because a piece that has stopped working is exactly the answer.
        let is_piece = |pos: GridPos| {
            self.grid
                .get(pos)
                .and_then(|tile| tile.building.as_ref())
                .is_some_and(|building| building.carries_traffic())
        };

        let mut reached = std::collections::HashSet::new();
        let mut frontier = vec![core];
        reached.insert((core.x, core.y));
        while let Some(pos) = frontier.pop() {
            for next in pos.neighbors() {
                if !next.in_bounds(self.grid.width, self.grid.height)
                    || reached.contains(&(next.x, next.y))
                    || !tile_carries_traffic_with(&self.grid, next, &dust_response)
                {
                    continue;
                }
                reached.insert((next.x, next.y));
                frontier.push(next);
            }
        }

        self.grid
            .iter_tiles()
            .map(|(pos, _)| pos)
            .filter(|pos| is_piece(*pos) && !reached.contains(&(pos.x, pos.y)))
            .collect()
    }

    /// Find a drone by id for the route-break effect at its current tile.
    pub(super) fn drone_position(&self, drone_id: u32) -> Option<GridPos> {
        self.drones
            .drones()
            .iter()
            .find(|drone| drone.id == drone_id)
            .map(|drone| drone.position)
    }

    /// Stop drones whose route has been cut, and restart the ones whose route
    /// has come back.
    pub(super) fn resolve_routes(&mut self, core: GridPos) -> Vec<DroneEvent> {
        // Borrowed field by field: the router reads the traffic map while the
        // drones are being handed new routes.
        let grid = &self.grid;
        let dust_response = self.resolved_dust_response();
        let cost = network_cost(
            &self.traffic,
            self.config.buildings.conduit_capacity,
            self.config.buildings.congestion_route_penalty,
        );
        let mut events = Vec::new();
        let network_changed = self.route_cache_revision != self.network_revision;
        if network_changed {
            self.route_cache_revision = self.network_revision;
        }

        for drone in self.drones.drones_mut() {
            match drone.state {
                DroneState::Error => {
                    if network_changed {
                        repath(grid, drone, core, &cost, &dust_response);
                    }
                }
                DroneState::MovingToCore | DroneState::MovingToDrill => {
                    if !route_is_intact(grid, drone, &dust_response)
                        && !repath(grid, drone, core, &cost, &dust_response)
                    {
                        events.push(drone.block());
                    }
                }
                DroneState::Delivering => {
                    // The cargo is already banked by the ReachedCore event, so
                    // drop it before heading home: a drone that walks the route
                    // back is what makes a long run cost throughput.
                    drone.carrying = 0.0;
                    match route_over_network_weighted_with(
                        grid,
                        drone.position,
                        drone.home,
                        &cost,
                        &dust_response,
                    ) {
                        Some(route) => drone.return_to_drill(route),
                        None => events.push(drone.block()),
                    }
                }
                DroneState::Idle => {}
            }
        }

        events
    }

    /// Count the drones on each network tile, then set every drone's speed
    /// from the dust on its drill and the traffic on the tile it is crossing.
    ///
    /// A conduit tile passes `conduit_capacity` drones at full speed; past
    /// that they share it, so a shared trunk slows everything routed through
    /// it. This is the pressure that makes a second route worth building.
    pub(super) fn update_traffic(&mut self) {
        self.traffic.clear();
        for drone in self.drones.drones() {
            if !matches!(
                drone.state,
                DroneState::MovingToCore | DroneState::MovingToDrill
            ) {
                continue;
            }
            let Some(tile) = drone.path.get(drone.path_index) else {
                continue;
            };
            *self.traffic.entry((tile.x, tile.y)).or_insert(0) += 1;
        }

        let capacity = self.config.buildings.conduit_capacity.max(1.0);
        let freeze = self.freeze_strength();
        let counter_radius = self.config.upkeep.hazard_counter_radius;
        let counter_strength = self.config.upkeep.hazard_counter_strength;
        let heaters = if freeze > 0.0 {
            self.powered_positions(BuildingType::HeaterNode)
        } else {
            Vec::new()
        };
        let base_speed = self.drones.drone_speed;
        let dust_response = self.resolved_dust_response();
        let grid = &self.grid;
        let traffic = &self.traffic;
        let hazard_fields = self.data().planet(self.planet_index).hazard_fields.clone();
        let feature_definitions = self.data().planet(self.planet_index).features.clone();
        let features = &self.features;
        let grid_width = self.grid.width;
        let grid_height = self.grid.height;

        for drone in self.drones.drones_mut() {
            let mut speed = base_speed;
            if let Some(building) = grid.get(drone.home).and_then(|t| t.building.as_ref()) {
                speed *= building.dust_drone_speed_multiplier_with(&dust_response);
            }
            if let Some(tile) = drone.path.get(drone.path_index) {
                // The cold bites where the network is not heated, so a long run
                // wants nodes spaced along it rather than one at the Core.
                if freeze > 0.0 {
                    let local_freeze = crate::state::progress::hazard_field_strength(
                        *tile,
                        "cold",
                        freeze,
                        &hazard_fields,
                        grid_width,
                        grid_height,
                    ) * crate::state::game_state::feature_hazard_multiplier(
                        features,
                        &feature_definitions,
                        *tile,
                        grid_width,
                        grid_height,
                        "cold",
                    );
                    let warmed = heaters
                        .iter()
                        .any(|heater| tile.distance(*heater) as i32 <= counter_radius);
                    let bite = if warmed {
                        local_freeze * (1.0 - counter_strength)
                    } else {
                        local_freeze
                    };
                    speed *= 1.0 - bite;
                }

                let load = traffic.get(&(tile.x, tile.y)).copied().unwrap_or(0) as f32;
                if load > capacity {
                    speed *= capacity / load;
                }
            }
            drone.speed = speed;
        }
    }

    /// Tiles carrying more traffic than they can pass.
    pub fn congested_tiles(&self) -> usize {
        let capacity = self.config.buildings.conduit_capacity.max(1.0);
        self.traffic
            .values()
            .filter(|load| **load as f32 > capacity)
            .count()
    }

    /// What each network tile is worth to a router this tick.
    ///
    /// Routing that ignores traffic sends every drone down the same shortest
    /// run no matter how saturated it is, which leaves a second parallel run
    /// doing nothing. Weighting by load is what makes laying one worth the
    /// minerals.
    pub(super) fn route_cost(&self) -> impl Fn(GridPos) -> f32 + '_ {
        let base = network_cost(
            &self.traffic,
            self.config.buildings.conduit_capacity,
            self.config.buildings.congestion_route_penalty,
        );
        move |pos| {
            base(pos)
                + self
                    .route_reservations
                    .get(&(pos.x, pos.y))
                    .copied()
                    .unwrap_or(0.0)
                    * 0.25
        }
    }

    /// Is this tile over its throughput limit right now?
    pub fn is_congested(&self, pos: GridPos) -> bool {
        let capacity = self.config.buildings.conduit_capacity.max(1.0);
        self.traffic
            .get(&(pos.x, pos.y))
            .is_some_and(|load| *load as f32 > capacity)
    }
}

/// Send a drone on again from wherever it stands: onward to the Core if it is
/// carrying, otherwise home to its drill. This is both how a re-route around a
/// cut happens and how a stalled drone goes back to work once the network is
/// whole. Returns `false` when the network cannot carry it there at all.
fn repath(
    grid: &Grid,
    drone: &mut Drone,
    core: GridPos,
    cost: impl Fn(GridPos) -> f32,
    dust_response: &DustResponseConfig,
) -> bool {
    let carrying = drone.carrying;
    let destination = match drone.state {
        // Whatever it was sent to, which may be a processing building.
        DroneState::MovingToCore => drone.target,
        DroneState::MovingToDrill => drone.home,
        _ if carrying > 0.0 => core,
        _ => drone.home,
    };

    if destination == drone.home && drone.position == destination {
        drone.state = DroneState::Idle;
        return true;
    }

    let Some(route) =
        route_over_network_weighted_with(grid, drone.position, destination, cost, dust_response)
    else {
        return false;
    };

    if destination == core {
        // Whatever it was already carrying stays what it is carrying.
        let resource = drone.resource_type;
        drone.dispatch(destination, route, carrying, resource);
    } else {
        drone.return_to_drill(route);
    }
    true
}

/// Is the rest of this drone's path still on the network? The final tile is the
/// destination (a drill does not itself carry traffic), so it is exempt.
fn route_is_intact(grid: &Grid, drone: &Drone, dust_response: &DustResponseConfig) -> bool {
    let remaining = drone.path.get(drone.path_index..).unwrap_or(&[]);
    let Some((_destination, corridor)) = remaining.split_last() else {
        return true;
    };
    corridor
        .iter()
        .all(|pos| tile_carries_traffic_with(grid, *pos, dust_response))
}

/// The routing cost of every network tile, given who is already on it.
fn network_cost(
    traffic: &HashMap<(i32, i32), u32>,
    capacity: f32,
    penalty: f32,
) -> impl Fn(GridPos) -> f32 + '_ {
    let capacity = capacity.max(1.0);
    move |pos: GridPos| {
        let load = traffic.get(&(pos.x, pos.y)).copied().unwrap_or(0);
        traffic_cost(load, capacity, penalty)
    }
}
