//! Logistics: drill dispatch, and keeping drones on an intact network route.
//!
//! Every delivery walks the conduit network (see `engine::routing`). A drone
//! whose route is cut stops where it stands and waves an error flag; it goes
//! back to work by itself once the network is whole again.

use crate::engine::{
    route_over_network, route_over_network_weighted, tile_carries_traffic, traffic_cost,
    BuildingType, Drone, DroneEvent, DroneState, Grid, GridPos, ResourceType, StatId,
};

use std::collections::HashMap;

use super::game_state::PlanetState;

/// How much a producer may stockpile on its pad while its drones are away, as
/// a multiple of a drone load. Past this it simply stops producing: a building
/// that outruns its logistics is the pressure, not free storage.
const PAD_LOADS: f32 = 3.0;
/// How many drone loads a processing building will let pile up in its hopper
/// before drones start taking ore elsewhere.
const HOPPER_LOADS: f32 = 3.0;

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
    }

    /// Whether a part-full pad is worth a trip.
    ///
    /// Only on a run long enough that the walk dominates, and only once the pad
    /// holds a worthwhile share of a load — otherwise a crew would trickle out
    /// single grains of ore.
    fn worth_a_partial_load(&self, carried: f32, load: f32, route_len: usize) -> bool {
        let config = &self.config.buildings;
        route_len as f32 >= config.partial_load_min_route
            && carried >= load * config.partial_load_min_share
    }

    /// Number of drones currently stalled on a broken route.
    pub fn stalled_drone_count(&self) -> usize {
        self.drones.count_by_state(DroneState::Error)
    }

    /// Every piece of network with no unbroken run back to the Core, plus the
    /// pieces that have stopped carrying traffic and caused it.
    ///
    /// The HUD could say a drone was stalled but never where the break was, so
    /// finding it meant walking the run by eye. Worth asking only while
    /// something is actually stalled: a run being laid outward from a drill is
    /// disconnected on purpose.
    pub fn severed_network(&self) -> Vec<GridPos> {
        let Some(core) = self.grid.find_core() else {
            return Vec::new();
        };

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
                    || !tile_carries_traffic(&self.grid, next)
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

    fn drone_position(&self, drone_id: u32) -> Option<GridPos> {
        self.drones
            .drones()
            .iter()
            .find(|drone| drone.id == drone_id)
            .map(|drone| drone.position)
    }

    /// Stop drones whose route has been cut, and restart the ones whose route
    /// has come back.
    fn resolve_routes(&mut self, core: GridPos) -> Vec<DroneEvent> {
        // Borrowed field by field: the router reads the traffic map while the
        // drones are being handed new routes.
        let grid = &self.grid;
        let cost = network_cost(
            &self.traffic,
            self.config.buildings.conduit_capacity,
            self.config.buildings.congestion_route_penalty,
        );
        let mut events = Vec::new();

        for drone in self.drones.drones_mut() {
            match drone.state {
                DroneState::Error => {
                    repath(grid, drone, core, &cost);
                }
                DroneState::MovingToCore | DroneState::MovingToDrill => {
                    if !route_is_intact(grid, drone) && !repath(grid, drone, core, &cost) {
                        events.push(drone.block());
                    }
                }
                DroneState::Delivering => {
                    // The cargo is already banked by the ReachedCore event, so
                    // drop it before heading home: a drone that walks the route
                    // back is what makes a long run cost throughput.
                    drone.carrying = 0.0;
                    match route_over_network_weighted(grid, drone.position, drone.home, &cost) {
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
    fn update_traffic(&mut self) {
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
        let grid = &self.grid;
        let traffic = &self.traffic;

        for drone in self.drones.drones_mut() {
            let mut speed = base_speed;
            if let Some(building) = grid.get(drone.home).and_then(|t| t.building.as_ref()) {
                speed *= building.dust_drone_speed_multiplier();
            }
            if let Some(tile) = drone.path.get(drone.path_index) {
                // The cold bites where the network is not heated, so a long run
                // wants nodes spaced along it rather than one at the Core.
                if freeze > 0.0 {
                    let warmed = heaters
                        .iter()
                        .any(|heater| tile.distance(*heater) as i32 <= counter_radius);
                    let bite = if warmed {
                        freeze * (1.0 - counter_strength)
                    } else {
                        freeze
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
    fn route_cost(&self) -> impl Fn(GridPos) -> f32 + '_ {
        network_cost(
            &self.traffic,
            self.config.buildings.conduit_capacity,
            self.config.buildings.congestion_route_penalty,
        )
    }

    /// Is this tile over its throughput limit right now?
    pub fn is_congested(&self, pos: GridPos) -> bool {
        let capacity = self.config.buildings.conduit_capacity.max(1.0);
        self.traffic
            .get(&(pos.x, pos.y))
            .is_some_and(|load| *load as f32 > capacity)
    }

    /// Where a load of ore from `from` should go, and how to get there.
    ///
    /// A processing building with room in its hopper takes priority over the
    /// Core, nearest first, so a Smelter parked beside the drills is fed before
    /// the ore ever reaches the pool. Placement is the decision; this only
    /// reads it.
    fn delivery_for(
        &self,
        from: GridPos,
        core: GridPos,
        resource: ResourceType,
    ) -> Option<(GridPos, Vec<GridPos>)> {
        let hopper_ceiling = self.drones.drone_capacity * HOPPER_LOADS;
        let mut best: Option<(GridPos, Vec<GridPos>)> = None;

        for pos in self.consumers_of(resource) {
            let delivered = self
                .input_buffers
                .get(&(pos.x, pos.y))
                .copied()
                .unwrap_or(0.0);
            if delivered + self.drones.drone_capacity > hopper_ceiling {
                continue;
            }
            let Some(route) = route_over_network_weighted(&self.grid, from, pos, self.route_cost())
            else {
                continue;
            };
            if best
                .as_ref()
                .is_none_or(|(_, shortest)| route.len() < shortest.len())
            {
                best = Some((pos, route));
            }
        }

        best.or_else(|| {
            route_over_network_weighted(&self.grid, from, core, self.route_cost())
                .map(|route| (core, route))
        })
    }

    /// Powered buildings whose recipe eats ore.
    /// Powered buildings whose hopper wants this resource carried to it.
    ///
    /// A drone looks for one of these before falling back to the Core, which
    /// is what makes a Smelter parked beside the drills get fed first.
    fn consumers_of(&self, resource: ResourceType) -> Vec<GridPos> {
        let mut consumers: Vec<GridPos> = crate::data::game_data()
            .buildings
            .iter()
            .filter(|def| {
                def.recipe.carried.as_deref() == Some(resource.id())
                    && def.recipe.carried_rate() > 0.0
            })
            .filter_map(|def| BuildingType::from_id(&def.id))
            .flat_map(|kind| self.powered_positions(kind))
            .collect();

        // A Mass Driver wants whatever this world's standing order says it is
        // throwing, and nothing at all while there is nowhere to throw it. It
        // is fed over the network like any other consumer: an export is a
        // logistics run that happens to end in orbit.
        if self
            .export
            .is_some_and(|order| order.cargo == resource && order.target != self.planet_index)
        {
            consumers.extend(self.powered_positions(BuildingType::MassDriver));
        }

        consumers
    }

    /// Cut ore into each drill's buffer.
    ///
    /// Everything a drill produces goes on its pad; getting it anywhere is
    /// [`Self::dispatch_producers`]'s problem, the same as for a Smelter.
    fn update_drills(&mut self, delta_time: f32) {
        let rate = self.drill_output_rate();
        let ceiling = self.drones.drone_capacity * PAD_LOADS;
        let depletion = self.config.ore.depletion_per_unit.max(0.0);

        for drill_pos in self.grid.find_buildings(BuildingType::Drill) {
            let Some(tile) = self.grid.get(drill_pos) else {
                continue;
            };
            let Some(building) = tile.building.as_ref() else {
                continue;
            };
            if !building.powered || building.is_dust_stalled() {
                continue;
            }
            // What the ground under this particular drill is worth. Placement
            // is a spatial decision, not only a question of run length.
            let richness = tile.ore_richness;
            let efficiency = building.dust_efficiency();
            let cut = rate * richness * efficiency * delta_time;

            let buffer = self
                .output_buffers
                .entry((drill_pos.x, drill_pos.y))
                .or_insert(0.0);
            let before = *buffer;
            *buffer = (before + cut).min(ceiling);

            // Only the bonus is spent: a deposit is cut down towards ordinary
            // ground and stops there (gdd.md §3). What the pad could not hold
            // was never cut, so it is not charged for either.
            if richness > 1.0 && depletion > 0.0 {
                let taken = *buffer - before;
                if let Some(tile) = self.grid.get_mut(drill_pos) {
                    tile.ore_richness = (richness - taken * depletion).max(1.0);
                }
            }
        }
    }

    /// Give every producer a crew, and send a drone whenever a full load is
    /// waiting on its pad.
    ///
    /// A drill and a Smelter are the same problem here: something has piled up
    /// somewhere and has to be carried to whatever wants it. Only the resource
    /// and therefore the destination differ.
    fn dispatch_producers(&mut self, core: GridPos) {
        let load = self.drones.drone_capacity;
        let crew = self.drone_crew_size();

        for (producer, resource) in self.producers() {
            // Research can grow a crew; the extra drones turn up at the next
            // cycle rather than needing the building rebuilt.
            while self.drones.drones_at(producer).len() < crew {
                self.drones.spawn_drone(producer);
            }

            let waiting = self
                .output_buffers
                .get(&(producer.x, producer.y))
                .copied()
                .unwrap_or(0.0);
            if waiting <= 0.0 {
                continue;
            }

            // Ore goes to whatever wants it and is nearest on the network, and
            // to the Core only when nothing does. Refined output has no
            // consumer yet, so it always goes home to the Core.
            // Everything looks for something that wants it and is nearest on
            // the network, and goes home to the Core only when nothing does.
            let delivery = self.delivery_for(producer, core, resource);
            let Some((destination, route)) = delivery else {
                // Powered but no pipe anywhere: it piles up on the pad instead
                // of teleporting into the pool.
                continue;
            };

            // On a long run a drone that waits for a full load stands still
            // while ore piles up behind it, which is why a second drone used to
            // be worth nothing except on the longest runs. On a short run
            // waiting costs almost nothing, because it comes straight back.
            let carried = waiting.min(load);
            if carried < load && !self.worth_a_partial_load(carried, load, route.len()) {
                continue;
            }

            let idle_drone = self
                .drones
                .drones()
                .iter()
                .find(|d| d.home == producer && d.state == DroneState::Idle)
                .map(|d| d.id);

            let Some(drone_id) = idle_drone else {
                continue;
            };
            if let Some(buffer) = self.output_buffers.get_mut(&(producer.x, producer.y)) {
                *buffer -= carried;
            }
            if let Some(drone) = self.drones.get_drone_mut(drone_id) {
                drone.dispatch(destination, route, carried, resource);
            }
        }
    }

    /// Every powered building that piles something up for collection, and what
    /// that something is.
    fn producers(&self) -> Vec<(GridPos, ResourceType)> {
        let mut producers: Vec<(GridPos, ResourceType)> = self
            .powered_positions(BuildingType::Drill)
            .into_iter()
            .map(|pos| (pos, ResourceType::Minerals))
            .collect();

        // Anything whose recipe makes something a drone can carry is a
        // producer too, whatever that something turns out to be.
        for def in &crate::data::game_data().buildings {
            let Some(kind) = BuildingType::from_id(&def.id) else {
                continue;
            };
            for (id, rate) in &def.recipe.outputs {
                let Some(resource) = ResourceType::from_id(id) else {
                    continue;
                };
                if *rate <= 0.0 || !resource.is_physical() {
                    continue;
                }
                producers.extend(
                    self.powered_positions(kind)
                        .into_iter()
                        .map(|pos| (pos, resource)),
                );
            }
        }

        // A Landing Pad with a pod on it is a producer of whatever that pod
        // held. Nothing else about it is special: the cargo sits there until a
        // drone walks it somewhere, so a pad with no pipe to it is a pile.
        for pos in self.powered_positions(BuildingType::LandingPad) {
            let key = (pos.x, pos.y);
            if self.output_buffers.get(&key).copied().unwrap_or(0.0) <= 0.0 {
                continue;
            }
            if let Some(cargo) = self.pad_cargo.get(&key) {
                producers.push((pos, *cargo));
            }
        }

        producers
    }
}

/// Send a drone on again from wherever it stands: onward to the Core if it is
/// carrying, otherwise home to its drill. This is both how a re-route around a
/// cut happens and how a stalled drone goes back to work once the network is
/// whole. Returns `false` when the network cannot carry it there at all.
fn repath(grid: &Grid, drone: &mut Drone, core: GridPos, cost: impl Fn(GridPos) -> f32) -> bool {
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

    let Some(route) = route_over_network_weighted(grid, drone.position, destination, cost) else {
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
fn route_is_intact(grid: &Grid, drone: &Drone) -> bool {
    let remaining = drone.path.get(drone.path_index..).unwrap_or(&[]);
    let Some((_destination, corridor)) = remaining.split_last() else {
        return true;
    };
    corridor.iter().all(|pos| tile_carries_traffic(grid, *pos))
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

#[cfg(test)]
mod tests;
