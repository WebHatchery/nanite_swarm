use super::*;

#[test]
fn stamping_without_a_saved_blueprint_explains_the_required_step() {
    let mut state = state();
    let anchor = state.grid.find_core().unwrap();

    assert_eq!(state.place_blueprint(anchor), 0);
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message.contains("Save a blueprint")));
}

#[test]
fn saved_blueprint_restores_processor_flags_when_stamped() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let source = GridPos::new(core.x + 1, core.y);
    let destination = GridPos::new(core.x + 3, core.y);
    state.grid.reveal_around(core, 5);
    for pos in [source, destination] {
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    }
    state.resources.minerals = 1_000.0;
    state.resources.energy = 1_000.0;
    state.config.resources.max_energy = 1_000.0;
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(source));
    {
        let building = state
            .grid
            .get_mut(source)
            .unwrap()
            .building
            .as_mut()
            .unwrap();
        building.overclocked = true;
        building.input_priority = true;
        building.standby = false;
    }

    assert!(state.save_blueprint(source, &[source]));
    assert_eq!(state.place_blueprint(destination), 1);

    let building = state
        .grid
        .get(destination)
        .unwrap()
        .building
        .as_ref()
        .unwrap();
    assert_eq!(building.building_type, BuildingType::Smelter);
    assert_eq!(building.position, destination);
    assert!(building.overclocked);
    assert!(building.input_priority);
    assert!(!building.standby);
}

#[test]
fn stamping_clears_build_selection_before_the_next_map_tap() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let anchor = GridPos::new(core.x + 1, core.y);
    let next_tap = GridPos::new(core.x + 2, core.y);
    state.grid.reveal_around(core, 4);
    for pos in [anchor, next_tap] {
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    }
    state.resources.minerals = 1_000.0;
    state.resources.energy = 1_000.0;
    state.config.resources.max_energy = 1_000.0;
    state.blueprint.push(crate::state::BlueprintEntry {
        offset: (0, 0),
        building_type: BuildingType::Drill,
        overclocked: false,
        input_priority: false,
        standby: false,
    });
    state.selected_building = Some(BuildingType::Drill);
    state.selected_tile = Some(anchor);

    assert_eq!(state.place_blueprint(anchor), 1);
    assert_eq!(state.selected_building, None);
    assert_eq!(state.selected_tile, None);
    assert_eq!(state.relocation_source, None);
    assert!(!state.box_select_mode);
    assert_eq!(state.box_select_start, None);
    assert_eq!(state.drag_last_pos, None);
    assert_eq!(state.smart_drag_source, None);
}
