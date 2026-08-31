//! Traffic pressure and shared-trunk throughput coverage.

use super::*;

/// Put `count` drones on the same tile, mid-route, and let the traffic
/// pass settle.
fn crowd_one_tile(state: &mut PlanetState, count: usize, tile: GridPos) {
    for _ in 0..count {
        let id = state.drones.spawn_drone(tile);
        let drone = state.drones.get_drone_mut(id).unwrap();
        drone.dispatch(tile, vec![tile], 1.0, ResourceType::Minerals);
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
    let (mut state, _core, drill) = state_with_run(8);
    state.config.buildings.conduit_capacity = capacity;
    // The spur drill gets ordinary ground too, so this measures the
    // trunk and not which drill landed on a deposit.
    if let Some(tile) = state.grid.get_mut(GridPos::new(drill.x - 1, drill.y - 1)) {
        tile.ore_richness = 1.0;
    }
    // A second drill hanging off the same run.
    let spur = GridPos::new(drill.x - 1, drill.y - 1);
    state.grid.get_mut(spur).unwrap().terrain = crate::engine::TerrainType::Empty;
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(spur));
    state.grid.update_power_grid();
    ore_delivered(&mut state, 60.0)
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
