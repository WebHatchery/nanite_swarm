use super::*;
use crate::data::GameConfig;
use crate::engine::{BuildingType, DroneState, GridPos};

fn state() -> PlanetState {
    PlanetState::new(2, 42, GameConfig::default())
}

#[test]
fn alert_positions_order_stalled_crews_before_congestion() {
    let mut state = state();
    let core = state.grid.find_core().expect("generated Core");
    let drill = GridPos::new(core.x + 1, core.y);
    let congested = GridPos::new(core.x + 2, core.y);

    state.drones.spawn_drone(drill);
    state.drones.drones_mut()[0].state = DroneState::Error;
    state.traffic.insert(
        (congested.x, congested.y),
        state.config.buildings.conduit_capacity as u32 + 1,
    );

    let positions = state.alert_positions();
    assert_eq!(positions.first(), Some(&drill));
    assert!(positions.contains(&congested));
}

#[test]
fn alert_focus_cycles_and_drops_mutually_exclusive_modes() {
    let mut state = state();
    let core = state.grid.find_core().expect("generated Core");
    let first = GridPos::new(core.x + 1, core.y);
    let second = GridPos::new(core.x + 2, core.y);
    state.drones.spawn_drone(first);
    state.drones.spawn_drone(second);
    for drone in state.drones.drones_mut() {
        drone.state = DroneState::Error;
    }
    state.select_building(BuildingType::Drill);
    state.demolish_mode = true;
    state.relocation_source = Some(core);

    assert_eq!(state.focus_next_alert(), Some(first));
    assert_eq!(state.selected_tile, Some(first));
    assert_eq!(state.selected_building, None);
    assert!(!state.demolish_mode);
    assert_eq!(state.relocation_source, None);
    assert_eq!(state.focus_next_alert(), Some(second));
    assert_eq!(state.focus_next_alert(), Some(first));
}

#[test]
fn alert_focus_is_empty_when_the_world_is_nominal() {
    let mut state = state();
    assert!(state.alert_positions().is_empty());
    assert_eq!(state.focus_next_alert(), None);
}

#[test]
fn alert_positions_include_buildings_when_upkeep_starts_reducing_output() {
    let mut state = state();
    let core = state.grid.find_core().expect("generated Core");
    let drill = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(drill, 1);
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));

    let threshold = state.resolved_dust_response().efficiency_threshold;
    state
        .grid
        .get_mut(drill)
        .expect("drill tile")
        .building
        .as_mut()
        .expect("drill building")
        .dust = threshold;

    assert_eq!(state.upkeep_alert_positions(), vec![drill]);
    assert_eq!(state.alert_positions(), vec![drill]);
}

#[test]
fn corrosion_joins_the_upkeep_alert_before_network_failure() {
    let mut state = state();
    let core = state.grid.find_core().expect("generated Core");
    state
        .grid
        .get_mut(core)
        .expect("core tile")
        .building
        .as_mut()
        .expect("core building")
        .acid_wear = 75.0;

    assert_eq!(state.upkeep_alert_positions(), vec![core]);
}
