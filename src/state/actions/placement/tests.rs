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
