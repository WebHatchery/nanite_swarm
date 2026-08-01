//! Logistics: drill dispatch, and keeping drones on an intact network route.
//!
//! Every delivery walks the conduit network (see `engine::routing`). A drone
//! whose route is cut stops where it stands and waves an error flag; it goes
//! back to work by itself once the network is whole again.

use crate::engine::{
    route_over_network, tile_carries_traffic, BuildingType, Drone, DroneEvent, DroneState, Grid,
    GridPos, StatId,
};

use super::game_state::PlanetState;

/// How much a drill may stockpile while its drone is away, as a multiple of a
/// drone load. Past this the ore is simply not cut: a drill that outruns its
/// logistics is the pressure, not free storage.
const DRILL_BUFFER_LOADS: f32 = 3.0;

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

        self.update_drills(delta_time, core);
        self.update_traffic();
    }

    /// Number of drones currently stalled on a broken route.
    pub fn stalled_drone_count(&self) -> usize {
        self.drones.count_by_state(DroneState::Error)
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
        let grid = &self.grid;
        let mut events = Vec::new();

        for drone in self.drones.drones_mut() {
            match drone.state {
                DroneState::Error => {
                    repath(grid, drone, core);
                }
                DroneState::MovingToCore | DroneState::MovingToDrill => {
                    if !route_is_intact(grid, drone) && !repath(grid, drone, core) {
                        events.push(drone.block());
                    }
                }
                DroneState::Delivering => {
                    // The cargo is already banked by the ReachedCore event, so
                    // drop it before heading home: a drone that walks the route
                    // back is what makes a long run cost throughput.
                    drone.carrying = 0.0;
                    match route_over_network(grid, drone.position, drone.home_drill) {
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
        let base_speed = self.drones.drone_speed;
        let grid = &self.grid;
        let traffic = &self.traffic;

        for drone in self.drones.drones_mut() {
            let mut speed = base_speed;
            if let Some(building) = grid.get(drone.home_drill).and_then(|t| t.building.as_ref()) {
                speed *= building.dust_drone_speed_multiplier();
            }
            if let Some(tile) = drone.path.get(drone.path_index) {
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

    /// Is this tile over its throughput limit right now?
    pub fn is_congested(&self, pos: GridPos) -> bool {
        let capacity = self.config.buildings.conduit_capacity.max(1.0);
        self.traffic
            .get(&(pos.x, pos.y))
            .is_some_and(|load| *load as f32 > capacity)
    }

    /// Cut ore into each drill's buffer, and send a drone the moment there is
    /// a full load waiting for it.
    fn update_drills(&mut self, delta_time: f32, core: GridPos) {
        let rate = self
            .stats
            .apply(StatId::DrillOutput, self.config.buildings.drill_output_rate);
        let load = self.drones.drone_capacity;
        let ceiling = load * DRILL_BUFFER_LOADS;

        for drill_pos in self.grid.find_buildings(BuildingType::Drill) {
            let Some(building) = self.grid.get(drill_pos).and_then(|t| t.building.as_ref()) else {
                continue;
            };
            if !building.powered || building.is_dust_stalled() {
                continue;
            }
            let efficiency = building.dust_efficiency();

            let buffer = self
                .drill_buffers
                .entry((drill_pos.x, drill_pos.y))
                .or_insert(0.0);
            *buffer = (*buffer + rate * efficiency * delta_time).min(ceiling);
            if *buffer < load {
                continue;
            }

            let Some(route) = route_over_network(&self.grid, drill_pos, core) else {
                // Powered but no pipe to the Core: the ore piles up at the
                // drill instead of teleporting into the pool.
                continue;
            };

            let idle_drone = self
                .drones
                .drones()
                .iter()
                .find(|d| d.home_drill == drill_pos && d.state == DroneState::Idle)
                .map(|d| d.id);

            let Some(drone_id) = idle_drone else {
                continue;
            };
            if let Some(buffer) = self.drill_buffers.get_mut(&(drill_pos.x, drill_pos.y)) {
                *buffer -= load;
            }
            if let Some(drone) = self.drones.get_drone_mut(drone_id) {
                drone.dispatch_to_core(core, route, load);
            }
        }
    }
}

/// Send a drone on again from wherever it stands: onward to the Core if it is
/// carrying, otherwise home to its drill. This is both how a re-route around a
/// cut happens and how a stalled drone goes back to work once the network is
/// whole. Returns `false` when the network cannot carry it there at all.
fn repath(grid: &Grid, drone: &mut Drone, core: GridPos) -> bool {
    let carrying = drone.carrying;
    let destination = match drone.state {
        DroneState::MovingToCore => core,
        DroneState::MovingToDrill => drone.home_drill,
        _ if carrying > 0.0 => core,
        _ => drone.home_drill,
    };

    if destination == drone.home_drill && drone.position == destination {
        drone.state = DroneState::Idle;
        return true;
    }

    let Some(route) = route_over_network(grid, drone.position, destination) else {
        return false;
    };

    if destination == core {
        drone.dispatch_to_core(core, route, carrying);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;

    /// One full drone load at the shipped drill rate: 10 minerals at 5/s.
    const FILL_SECONDS: f32 = 2.0;

    /// A Core with a conduit run east and a drill at the far end of it.
    fn state_with_run(length: i32) -> (PlanetState, GridPos, GridPos) {
        let mut state = PlanetState::new("Test", 24, 24, 42, GameConfig::default());
        let core = state.grid.find_core().unwrap();
        state.grid.reveal_around(core, 24);
        state.resources.minerals = 10_000.0;
        state.resources.energy = 10_000.0;
        state.config.resources.max_energy = 10_000.0;
        state.unlock_building(BuildingType::Conduit);
        state.unlock_building(BuildingType::PowerNode);

        for step in 1..=length {
            let pos = GridPos::new(core.x + step, core.y);
            state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
            // A repeater every five tiles keeps a long run powered; power nodes
            // carry traffic too, so the route length is unaffected.
            let piece = if step % 5 == 0 {
                BuildingType::PowerNode
            } else {
                BuildingType::Conduit
            };
            state.select_building(piece);
            assert!(state.try_place_building(pos), "run piece at step {step}");
        }

        let drill = GridPos::new(core.x + length + 1, core.y);
        state.grid.get_mut(drill).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.select_building(BuildingType::Drill);
        assert!(state.try_place_building(drill));
        state.grid.update_power_grid();

        (state, core, drill)
    }

    /// Loads delivered by a single drill over `seconds` of fixed-step
    /// simulation, with the storage cap lifted out of the way.
    fn loads_delivered(state: &mut PlanetState, seconds: f32) -> u32 {
        state.resources.minerals = 0.0;
        state.config.resources.base_mineral_cap = 100_000.0;

        let ticks = (seconds / crate::state::TICK_SECONDS) as u32;
        for _ in 0..ticks {
            state.step(crate::state::TICK_SECONDS, false);
        }

        (state.resources.minerals / state.drones.drone_capacity).round() as u32
    }

    /// Snapshot of the whole harvest loop at the fixed timestep: drill cycle,
    /// dispatch, travel out over the network, delivery, and the walk home. If
    /// the tick length, drone speed, drill cycle or route cost changes, this
    /// number moves.
    #[test]
    fn a_drill_beside_the_core_delivers_four_loads_in_ten_seconds() {
        let (mut state, _core, _drill) = state_with_run(0);
        assert_eq!(loads_delivered(&mut state, 10.0), 4);
    }

    /// The point of the pillar: the same drill on the end of a long run
    /// delivers less, because the round trip now costs more than the cycle.
    #[test]
    fn a_drill_at_the_end_of_a_long_run_delivers_less() {
        let (mut near, _, _) = state_with_run(1);
        let (mut far, _, _) = state_with_run(9);
        assert!(loads_delivered(&mut far, 30.0) < loads_delivered(&mut near, 30.0));
    }

    /// Put `count` drones on the same tile, mid-route, and let the traffic
    /// pass settle.
    fn crowd_one_tile(state: &mut PlanetState, count: usize, tile: GridPos) {
        for _ in 0..count {
            let id = state.drones.spawn_drone(tile);
            let drone = state.drones.get_drone_mut(id).unwrap();
            drone.dispatch_to_core(tile, vec![tile], 1.0);
        }
        state.update_traffic();
    }

    #[test]
    fn traffic_below_capacity_costs_nothing() {
        let (mut state, core, _) = state_with_run(2);
        state.drones.drones_mut().iter_mut().for_each(|drone| {
            drone.state = DroneState::Idle;
        });
        crowd_one_tile(&mut state, 2, core);

        assert_eq!(state.congested_tiles(), 0);
        assert!(!state.is_congested(core));
        for drone in state.drones.drones() {
            if drone.state == DroneState::MovingToCore {
                assert_eq!(drone.speed, state.drones.drone_speed);
            }
        }
    }

    #[test]
    fn a_crowded_tile_slows_everything_crossing_it() {
        let (mut state, core, _) = state_with_run(2);
        state.drones.drones_mut().iter_mut().for_each(|drone| {
            drone.state = DroneState::Idle;
        });
        let capacity = state.config.buildings.conduit_capacity;
        crowd_one_tile(&mut state, 6, core);

        assert!(state.is_congested(core));
        assert_eq!(state.congested_tiles(), 1);

        let expected = state.drones.drone_speed * (capacity / 6.0);
        for drone in state.drones.drones() {
            if drone.state == DroneState::MovingToCore {
                assert!(
                    (drone.speed - expected).abs() < 1e-3,
                    "{} vs {expected}",
                    drone.speed
                );
            }
        }
    }

    #[test]
    fn traffic_only_counts_drones_that_are_actually_moving() {
        let (mut state, core, _) = state_with_run(2);
        state.drones.drones_mut().iter_mut().for_each(|drone| {
            drone.state = DroneState::Idle;
        });
        crowd_one_tile(&mut state, 6, core);
        assert!(state.is_congested(core));

        // Park them all: the jam clears.
        for drone in state.drones.drones_mut() {
            drone.state = DroneState::Idle;
        }
        state.update_traffic();
        assert_eq!(state.congested_tiles(), 0);
        assert_eq!(state.drones.drones()[0].speed, state.drones.drone_speed);
    }

    /// Two drills sharing one run, measured over a minute at a given tile
    /// capacity.
    fn shared_trunk_throughput(capacity: f32) -> u32 {
        let (mut state, _core, drill) = state_with_run(4);
        state.config.buildings.conduit_capacity = capacity;
        // A second drill hanging off the same run.
        let spur = GridPos::new(drill.x - 1, drill.y - 1);
        state.grid.get_mut(spur).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.select_building(BuildingType::Drill);
        assert!(state.try_place_building(spur));
        state.grid.update_power_grid();
        loads_delivered(&mut state, 60.0)
    }

    #[test]
    fn a_saturated_trunk_delivers_less_than_a_clear_one() {
        let clear = shared_trunk_throughput(100.0);
        let saturated = shared_trunk_throughput(0.5);
        assert!(
            saturated < clear,
            "saturated run delivered {saturated}, clear run delivered {clear}"
        );
    }

    #[test]
    fn drills_dispatch_along_the_conduit_run() {
        let (mut state, core, drill) = state_with_run(3);
        state.update_logistics(FILL_SECONDS, false);

        let drone = &state.drones.drones()[0];
        assert_eq!(drone.state, DroneState::MovingToCore);
        assert_eq!(drone.path.len(), 4);
        assert_eq!(drone.path.last(), Some(&core));
        assert_eq!(drone.home_drill, drill);
    }

    #[test]
    fn a_drill_with_no_pipe_to_the_core_never_dispatches() {
        let (mut state, _core, drill) = state_with_run(3);
        // Cut the run right next to the drill.
        state
            .grid
            .remove_building(GridPos::new(drill.x - 1, drill.y));
        state.grid.update_power_grid();

        state.update_logistics(FILL_SECONDS * 4.0, false);
        assert_eq!(state.drones.drones()[0].state, DroneState::Idle);
    }

    #[test]
    fn cutting_the_run_stops_a_drone_in_flight_without_losing_its_cargo() {
        let (mut state, _core, drill) = state_with_run(4);
        state.update_logistics(FILL_SECONDS, false);
        assert_eq!(state.drones.drones()[0].state, DroneState::MovingToCore);

        state
            .grid
            .remove_building(GridPos::new(drill.x - 3, drill.y));
        state.grid.update_power_grid();
        state.update_logistics(0.0, false);

        let drone = &state.drones.drones()[0];
        assert_eq!(drone.state, DroneState::Error);
        assert!(drone.carrying > 0.0);
        assert_eq!(state.stalled_drone_count(), 1);
    }

    #[test]
    fn a_cut_run_is_routed_around_when_a_second_run_exists() {
        let (mut state, core, drill) = state_with_run(4);
        // A parallel run one row south, joined at both ends.
        state.select_building(BuildingType::Conduit);
        for x in core.x..=drill.x {
            let pos = GridPos::new(x, core.y + 1);
            state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
            assert!(state.try_place_building(pos));
        }
        state.grid.update_power_grid();
        state.update_logistics(FILL_SECONDS, false);
        assert_eq!(state.drones.drones()[0].state, DroneState::MovingToCore);

        state
            .grid
            .remove_building(GridPos::new(drill.x - 2, drill.y));
        state.grid.update_power_grid();
        state.update_logistics(0.0, false);

        let drone = &state.drones.drones()[0];
        assert_eq!(drone.state, DroneState::MovingToCore);
        assert_eq!(drone.path.last(), Some(&core));
        assert_eq!(state.stalled_drone_count(), 0);
    }

    #[test]
    fn repairing_the_run_sends_a_stalled_drone_on_its_way_again() {
        let (mut state, core, drill) = state_with_run(4);
        state.update_logistics(FILL_SECONDS, false);
        let cut = GridPos::new(drill.x - 3, drill.y);
        state.grid.remove_building(cut);
        state.grid.update_power_grid();
        state.update_logistics(0.0, false);
        assert_eq!(state.drones.drones()[0].state, DroneState::Error);

        state.select_building(BuildingType::Conduit);
        assert!(state.try_place_building(cut));
        state.update_logistics(0.0, false);

        let drone = &state.drones.drones()[0];
        assert_eq!(drone.state, DroneState::MovingToCore);
        assert_eq!(drone.path.last(), Some(&core));
        assert!(drone.carrying > 0.0);
    }

    #[test]
    fn drones_stranded_by_a_power_collapse_recover_once_it_passes() {
        let (mut state, _core, _drill) = state_with_run(3);
        state.update_logistics(FILL_SECONDS, false);
        state.trigger_power_collapse();
        assert_eq!(state.drones.drones()[0].state, DroneState::Error);
        assert_eq!(state.drones.drones()[0].carrying, 0.0);

        // The collapse blocks the sim; once it lifts, drones head home.
        state.power_collapse_shutdown = 0.0;
        for _ in 0..40 {
            state.update_logistics(0.1, false);
            state.drones.update(0.1);
        }
        assert_ne!(state.drones.drones()[0].state, DroneState::Error);
    }

    #[test]
    fn a_longer_run_takes_a_longer_route_than_a_direct_one() {
        let (mut short_run, _, _) = state_with_run(1);
        let (mut long_run, _, _) = state_with_run(6);
        short_run.update_logistics(FILL_SECONDS, false);
        long_run.update_logistics(FILL_SECONDS, false);

        assert!(long_run.drones.drones()[0].path.len() > short_run.drones.drones()[0].path.len());
    }
}
