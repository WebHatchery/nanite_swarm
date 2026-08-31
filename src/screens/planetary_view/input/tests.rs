use super::*;

use crate::data::GameConfig;
use crate::engine::TerrainType;

fn state() -> PlanetState {
    PlanetState::new(2, 42, GameConfig::default())
}

#[test]
fn blocked_placement_names_the_missing_terrain_prerequisite() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let water = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(water).unwrap().terrain = TerrainType::Water;
    state.unlock_building(BuildingType::Bridge);

    assert_eq!(
        placement_block_message(&state, water, BuildingType::Drill).as_deref(),
        Some("That terrain cannot hold this building")
    );
    assert_eq!(
        placement_block_message(&state, water, BuildingType::Bridge).as_deref(),
        None
    );
}

#[test]
fn unrevealed_and_occupied_tiles_explain_why_building_must_wait() {
    let state = state();
    let core = state.grid.find_core().unwrap();
    let unrevealed = GridPos::new(0, 0);

    assert_eq!(
        placement_block_message(&state, unrevealed, BuildingType::Drill).as_deref(),
        Some("Survey this tile before building")
    );
    assert_eq!(
        placement_block_message(&state, core, BuildingType::Drill).as_deref(),
        Some("This tile is already occupied")
    );
}

#[test]
fn unaffordable_primary_placement_explains_the_resource_shortfall() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.resources.minerals = 0.0;
    state.resources.energy = 0.0;

    assert!(!try_place_selected_building(
        &mut state,
        pos,
        BuildingType::Drill
    ));
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message.contains("Need M 20")));
}

#[test]
fn touch_tap_on_existing_building_selects_without_arming_smart_drag() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();

    handle_existing_building_primary(&mut state, core, false);

    assert_eq!(state.selected_tile, Some(core));
    assert_eq!(state.smart_drag_source, None);
    assert_eq!(state.drag_last_pos, None);
}

#[test]
fn pointer_press_on_existing_building_keeps_smart_drag_available() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();

    handle_existing_building_primary(&mut state, core, true);

    assert_eq!(state.selected_tile, Some(core));
    assert_eq!(state.smart_drag_source, Some(core));
    assert_eq!(state.drag_last_pos, Some(core));
}
