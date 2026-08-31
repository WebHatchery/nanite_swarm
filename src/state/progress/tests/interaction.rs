use super::*;

#[test]
fn unlock_building_does_not_duplicate_entries() {
    let mut state = PlanetState::default();
    state.unlock_building(BuildingType::Storage);
    state.unlock_building(BuildingType::Storage);
    assert_eq!(
        state
            .unlocked_buildings
            .iter()
            .filter(|b| **b == BuildingType::Storage)
            .count(),
        1
    );
}

#[test]
fn cancel_interaction_modes_disarms_every_pending_map_action() {
    let mut state = state();
    let core = state.grid.find_core().expect("generated Core");
    let tile = GridPos::new(core.x + 1, core.y);
    state.selected_building = Some(BuildingType::Drill);
    state.selected_tile = Some(tile);
    state.demolish_mode = true;
    state.relocation_source = Some(core);
    state.box_select_mode = true;
    state.box_select_start = Some(core);
    state.box_selected = vec![core, tile];
    state.purge_armed = Some(tile);
    state.bulk_purge_armed = true;
    state.drag_last_pos = Some(tile);
    state.smart_drag_source = Some(core);
    state.touch_camera_active = true;
    state.touch_route_active = true;
    state.touch_gesture_routed = true;
    state.blueprint.push(crate::state::BlueprintEntry {
        offset: (0, 0),
        building_type: BuildingType::Drill,
        overclocked: false,
        input_priority: false,
        standby: false,
    });
    let blueprint_len = state.blueprint.len();

    state.cancel_interaction_modes();

    assert_eq!(state.selected_building, None);
    assert_eq!(state.selected_tile, None);
    assert!(!state.demolish_mode);
    assert_eq!(state.relocation_source, None);
    assert!(!state.box_select_mode);
    assert_eq!(state.box_select_start, None);
    assert!(state.box_selected.is_empty());
    assert_eq!(state.purge_armed, None);
    assert!(!state.bulk_purge_armed);
    assert_eq!(state.drag_last_pos, None);
    assert_eq!(state.smart_drag_source, None);
    assert!(!state.touch_camera_active);
    assert!(!state.touch_route_active);
    assert!(!state.touch_gesture_routed);
    assert_eq!(state.blueprint.len(), blueprint_len);
}

#[test]
fn choosing_a_building_cancels_relocation_before_the_next_tap() {
    let mut state = state();
    let core = state.grid.find_core().expect("generated Core");
    state.relocation_source = Some(core);
    state.box_select_mode = true;
    state.box_select_start = Some(core);
    state.select_building(BuildingType::Drill);

    assert_eq!(state.selected_building, Some(BuildingType::Drill));
    assert_eq!(state.relocation_source, None);
    assert!(!state.box_select_mode);
    assert_eq!(state.box_select_start, None);
}
