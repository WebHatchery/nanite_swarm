use super::*;

use crate::data::GameConfig;
use crate::engine::TerrainType;

fn state() -> PlanetState {
    PlanetState::new(2, 42, GameConfig::default())
}

#[test]
fn placement_reason_reports_the_world_ban_before_unlock_or_cost() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.banned_buildings.push(BuildingType::WindTurbine);

    assert_eq!(
        state.placement_block_reason(pos, BuildingType::WindTurbine),
        Some(PlacementBlockReason::Banned)
    );
    assert_eq!(
        PlacementBlockReason::Banned.message(&state, BuildingType::WindTurbine),
        "This world bans that building"
    );
}

#[test]
fn placement_reason_catches_resources_after_terrain_and_research() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.resources.minerals = 0.0;
    state.resources.energy = 0.0;

    assert_eq!(
        state.placement_block_reason(pos, BuildingType::Drill),
        Some(PlacementBlockReason::Resources)
    );
}

#[test]
fn direct_build_action_reports_the_same_terrain_reason_as_the_preview() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let water = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(water).unwrap().terrain = TerrainType::Water;
    state.select_building(BuildingType::Drill);

    assert!(!state.try_place_building(water));
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message == "That terrain cannot hold this building"));
}

#[test]
fn direct_build_action_reports_a_cost_shortfall() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.resources.minerals = 0.0;
    state.resources.energy = 0.0;

    assert!(!state.try_place_building(pos));
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message == "Need M 20 and E 10 to build"));
}

#[test]
fn a_conduit_drag_reports_the_next_segment_cost_when_resources_run_out() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let first = GridPos::new(core.x + 1, core.y);
    let second = GridPos::new(core.x + 2, core.y);
    for pos in [first, second] {
        let tile = state.grid.get_mut(pos).unwrap();
        tile.terrain = TerrainType::Empty;
    }
    state.grid.reveal_around(second, 1);
    state.unlock_building(BuildingType::Conduit);
    state.resources.minerals = 5.0;
    state.resources.energy = 2.0;

    assert!(state.try_place_conduit_path(core, second));
    assert!(state.grid.get(first).unwrap().building.is_some());
    assert!(state.grid.get(second).unwrap().building.is_none());
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification
            .message
            .contains("Route paused: need M 5 and E 2 per conduit")));
}

#[test]
fn a_blocked_conduit_segment_warns_once_until_the_gesture_ends() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let first = GridPos::new(core.x + 1, core.y);
    let second = GridPos::new(core.x + 2, core.y);
    for pos in [first, second] {
        let tile = state.grid.get_mut(pos).unwrap();
        tile.terrain = TerrainType::Empty;
    }
    state.grid.reveal_around(second, 1);
    state.unlock_building(BuildingType::Conduit);
    state.resources.minerals = 0.0;
    state.resources.energy = 0.0;

    assert!(!state.try_place_conduit_path(core, second));
    assert_eq!(state.notifications.count(), 1);
    assert!(!state.try_place_conduit_path(core, second));
    assert_eq!(state.notifications.count(), 1);

    state.cancel_interaction_modes();
    assert!(!state.try_place_conduit_path(core, second));
    assert_eq!(state.notifications.count(), 2);
}

#[test]
fn a_conduit_route_explains_a_water_gap_and_deduplicates_until_reset() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let water = GridPos::new(core.x + 1, core.y);
    let tile = state.grid.get_mut(water).unwrap();
    tile.terrain = TerrainType::Water;
    tile.revealed = true;
    state.unlock_building(BuildingType::Conduit);

    assert!(!state.try_place_conduit_path(core, water));
    assert_eq!(state.notifications.count(), 1);
    assert_eq!(
        state.notifications.get_notifications()[0].message,
        "Route blocked: place a Bridge on the water or void gap first"
    );
    assert!(!state.try_place_conduit_path(core, water));
    assert_eq!(state.notifications.count(), 1);

    state.cancel_interaction_modes();
    assert!(!state.try_place_conduit_path(core, water));
    assert_eq!(state.notifications.count(), 2);
}

#[test]
fn a_conduit_route_started_on_empty_ground_owns_its_first_tile() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let first = GridPos::new(core.x + 1, core.y);
    let second = GridPos::new(core.x + 2, core.y);
    for pos in [first, second] {
        let tile = state.grid.get_mut(pos).unwrap();
        tile.terrain = TerrainType::Empty;
    }
    state.grid.reveal_around(second, 1);
    state.unlock_building(BuildingType::Conduit);
    state.resources.minerals = 10.0;
    state.resources.energy = 4.0;

    assert!(state.try_place_conduit_path(first, second));
    assert!(state.grid.get(first).unwrap().building.is_some());
    assert!(state.grid.get(second).unwrap().building.is_some());
    assert!(state.notifications.get_notifications().is_empty());
}

#[test]
fn placement_reason_treats_world_requirements_as_a_separate_block() {
    let mut state = PlanetState::new(0, 42, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::MassDriver);
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;

    assert_eq!(
        state.placement_block_reason(pos, BuildingType::MassDriver),
        Some(PlacementBlockReason::Constraint)
    );
    assert!(PlacementBlockReason::Constraint
        .message(&state, BuildingType::MassDriver)
        .contains("Mass Driver"));
}

#[test]
fn placement_reason_explains_when_a_power_source_needs_another_world() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::GeothermalTap);
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;

    assert_eq!(
        state.placement_block_reason(pos, BuildingType::GeothermalTap),
        Some(PlacementBlockReason::WorldCondition)
    );
    assert_eq!(
        PlacementBlockReason::WorldCondition.message(&state, BuildingType::GeothermalTap),
        "This building requires an acid world"
    );
}

#[test]
fn an_acid_world_can_place_geothermal_power_beside_the_core() {
    let mut state = PlanetState::new(1, 42, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::GeothermalTap);
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    state.select_building(BuildingType::GeothermalTap);

    assert!(state.try_place_building(pos));
    assert_eq!(state.power_generation(), 16.0);
}

#[test]
fn a_cold_world_can_place_fusion_power_beside_the_core() {
    let mut state = PlanetState::new(3, 42, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::FusionReactor);
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    state.select_building(BuildingType::FusionReactor);

    assert!(state.try_place_building(pos));
    assert_eq!(state.power_generation(), 22.0);
}
