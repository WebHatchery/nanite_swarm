use super::*;

fn straight_path(len: usize) -> Vec<GridPos> {
    (1..=len as i32).map(|x| GridPos::new(x, 0)).collect()
}

#[test]
fn dispatch_to_core_caps_carrying_at_capacity() {
    let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
    drone.dispatch(
        GridPos::new(3, 0),
        straight_path(3),
        999.0,
        ResourceType::Minerals,
    );
    assert_eq!(drone.carrying, 10.0);
    assert_eq!(drone.state, DroneState::MovingToCore);
    assert_eq!(drone.path_index, 0);
}

#[test]
fn update_does_not_arrive_before_crossing_full_progress() {
    let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
    drone.dispatch(
        GridPos::new(1, 0),
        straight_path(1),
        5.0,
        ResourceType::Minerals,
    );

    // speed(5.0) * delta(0.1) = 0.5 progress: not enough to cross one edge yet.
    let event = drone.update(0.1);
    assert!(event.is_none());
    assert_eq!(drone.state, DroneState::MovingToCore);
    assert_eq!(drone.position, GridPos::new(0, 0));
}

#[test]
fn update_reaches_core_after_crossing_a_single_hop_path() {
    let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
    drone.dispatch(
        GridPos::new(1, 0),
        straight_path(1),
        5.0,
        ResourceType::Minerals,
    );

    // speed(5.0) * delta(0.3) = 1.5 progress: crosses the single edge.
    let event = drone.update(0.3);
    assert!(matches!(
        event,
        Some(DroneEvent::Delivered { amount, .. }) if amount == 5.0
    ));
    assert_eq!(drone.state, DroneState::Delivering);
    assert_eq!(drone.position, GridPos::new(1, 0));
}

#[test]
fn update_eventually_reaches_core_over_a_multi_hop_path() {
    let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
    drone.dispatch(
        GridPos::new(3, 0),
        straight_path(3),
        5.0,
        ResourceType::Minerals,
    );

    let mut delivered = None;
    for _ in 0..20 {
        if let Some(DroneEvent::Delivered { amount, .. }) = drone.update(0.3) {
            delivered = Some(amount);
            break;
        }
    }
    assert_eq!(delivered, Some(5.0));
    assert_eq!(drone.state, DroneState::Delivering);
    assert_eq!(drone.position, GridPos::new(3, 0));
}

#[test]
fn return_to_drill_reaches_drill_and_goes_idle() {
    let mut drone = Drone::new(1, GridPos::new(2, 0), 10.0, 10.0);
    drone.position = GridPos::new(2, 0);
    drone.return_to_drill(straight_path(0));
    // Empty path: the very next update should immediately arrive.
    let event = drone.update(1.0);
    assert!(matches!(event, Some(DroneEvent::ReachedDrill { .. })));
    assert_eq!(drone.state, DroneState::Idle);
}

#[test]
fn idle_and_error_states_do_not_move() {
    let mut idle = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
    assert!(idle.update(1.0).is_none());
    assert_eq!(idle.position, GridPos::new(0, 0));

    let mut errored = Drone::new(2, GridPos::new(0, 0), 10.0, 5.0);
    errored.state = DroneState::Error;
    assert!(errored.update(1.0).is_none());
    assert_eq!(errored.state, DroneState::Error);
}

#[test]
fn manager_spawns_and_tracks_drones_per_drill() {
    let mut manager = DroneManager::new(10.0, 5.0);
    let drill_a = GridPos::new(0, 0);
    let drill_b = GridPos::new(5, 5);
    manager.spawn_drone(drill_a);
    manager.spawn_drone(drill_a);
    manager.spawn_drone(drill_b);

    assert_eq!(manager.total_count(), 3);
    assert_eq!(manager.drones_at(drill_a).len(), 2);
    assert_eq!(manager.count_by_state(DroneState::Idle), 3);

    manager.remove_drones_at(drill_a);
    assert_eq!(manager.total_count(), 1);
}

#[test]
fn manager_assigns_unique_ascending_ids() {
    let mut manager = DroneManager::new(10.0, 5.0);
    let id1 = manager.spawn_drone(GridPos::new(0, 0));
    let id2 = manager.spawn_drone(GridPos::new(0, 0));
    assert_ne!(id1, id2);
    assert!(manager.get_drone_mut(id1).is_some());
    assert!(manager.get_drone_mut(id2).is_some());
    assert!(manager.get_drone_mut(id2 + 100).is_none());
}

#[test]
fn a_long_step_crosses_several_tiles_instead_of_capping_at_one() {
    let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
    drone.dispatch(
        GridPos::new(6, 0),
        straight_path(6),
        5.0,
        ResourceType::Minerals,
    );

    // speed(5.0) * delta(0.7) = 3.5 tiles of travel in a single step.
    assert!(drone.update(0.7).is_none());
    assert_eq!(drone.position, GridPos::new(3, 0));
    assert!((drone.progress - 0.5).abs() < 1e-5);
}

#[test]
fn a_step_long_enough_to_overshoot_still_arrives_once() {
    let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
    drone.dispatch(
        GridPos::new(3, 0),
        straight_path(3),
        5.0,
        ResourceType::Minerals,
    );

    // Sixty seconds of travel over a three tile path: arrive, do not wrap.
    let event = drone.update(60.0);
    assert!(matches!(event, Some(DroneEvent::Delivered { .. })));
    assert_eq!(drone.position, GridPos::new(3, 0));
    assert_eq!(drone.progress, 0.0);
}

#[test]
fn position_tracks_the_tile_the_drone_last_reached() {
    let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
    drone.dispatch(
        GridPos::new(3, 0),
        straight_path(3),
        5.0,
        ResourceType::Minerals,
    );

    // Half way along the first hop: still standing on the drill tile.
    drone.update(0.1);
    assert_eq!(drone.position, GridPos::new(0, 0));
    let (vx, _) = drone.visual_position();
    assert!(vx > 0.0 && vx < 1.0);

    // Crossing the first hop lands the drone on the first path tile.
    drone.update(0.2);
    assert_eq!(drone.position, GridPos::new(1, 0));
}

#[test]
fn blocking_a_drone_stops_it_where_it_stands_and_keeps_its_cargo() {
    let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
    drone.dispatch(
        GridPos::new(3, 0),
        straight_path(3),
        7.0,
        ResourceType::Minerals,
    );
    drone.update(0.3);

    let event = drone.block();
    assert!(matches!(event, DroneEvent::PathBlocked { drone_id: 1 }));
    assert_eq!(drone.state, DroneState::Error);
    assert_eq!(drone.carrying, 7.0);
    assert_eq!(drone.position, GridPos::new(1, 0));
    assert!(drone.path.is_empty());
    // A blocked drone does not drift onwards.
    assert!(drone.update(1.0).is_none());
    assert_eq!(drone.position, GridPos::new(1, 0));
}

#[test]
fn block_is_idempotent_for_an_already_blocked_drone() {
    let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
    drone.block();
    drone.block();
    assert_eq!(drone.state, DroneState::Error);
    assert_eq!(drone.position, GridPos::new(0, 0));
}
