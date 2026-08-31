use super::*;

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
