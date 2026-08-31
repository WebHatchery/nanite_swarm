use super::*;

#[test]
fn try_place_building_spends_resources_and_spawns_drone() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    let before_minerals = state.resources.minerals;

    assert!(state.try_place_building(pos));
    assert!(state.resources.minerals < before_minerals);
    assert_eq!(state.drones.total_count(), 1);
    assert!(state.output_buffers.contains_key(&(pos.x, pos.y)));
}

#[test]
fn try_place_building_fails_when_unaffordable() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.resources.minerals = 0.0;

    assert!(!state.try_place_building(pos));
    assert_eq!(state.drones.total_count(), 0);
}

#[test]
fn a_conduit_route_lays_every_segment_between_two_tiles() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 8);
    state.unlock_building(BuildingType::Conduit);
    state.select_building(BuildingType::Conduit);

    let end = GridPos::new(core.x + 3, core.y);
    for x in (core.x + 1)..=end.x {
        state.grid.get_mut(GridPos::new(x, core.y)).unwrap().terrain = TerrainType::Empty;
    }

    assert!(state.try_place_conduit_path(core, end));
    for x in (core.x + 1)..=end.x {
        assert_eq!(
            state
                .grid
                .get(GridPos::new(x, core.y))
                .and_then(|tile| tile.building.as_ref())
                .map(|building| building.building_type),
            Some(BuildingType::Conduit)
        );
    }
}

#[test]
fn smart_conduit_route_starts_at_an_existing_building() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 8);
    state.unlock_building(BuildingType::Conduit);

    let end = GridPos::new(core.x + 3, core.y);
    for x in (core.x + 1)..=end.x {
        state.grid.get_mut(GridPos::new(x, core.y)).unwrap().terrain = TerrainType::Empty;
    }

    assert!(state.try_place_smart_conduit_path(core, end));
    for x in (core.x + 1)..=end.x {
        assert_eq!(
            state
                .grid
                .get(GridPos::new(x, core.y))
                .and_then(|tile| tile.building.as_ref())
                .map(|building| building.building_type),
            Some(BuildingType::Conduit)
        );
    }
}

#[test]
fn smart_conduit_route_rejects_an_empty_source_tile() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 8);
    state.unlock_building(BuildingType::Conduit);
    let source = GridPos::new(core.x + 1, core.y);
    let end = GridPos::new(core.x + 3, core.y);
    for x in (core.x + 1)..=end.x {
        state.grid.get_mut(GridPos::new(x, core.y)).unwrap().terrain = TerrainType::Empty;
    }

    assert!(!state.try_place_smart_conduit_path(source, end));
    assert!(state.grid.get(source).unwrap().building.is_none());
    assert!(state.grid.get(end).unwrap().building.is_none());
}

#[test]
fn demolish_mode_and_a_build_cursor_cannot_both_be_armed() {
    let mut state = state();
    state.select_building(BuildingType::Drill);
    assert_eq!(state.selected_building, Some(BuildingType::Drill));

    state.toggle_demolish_mode();
    assert!(state.demolish_mode);
    assert_eq!(state.selected_building, None, "still holding a building");

    // Picking a building back up puts the wrecking ball down.
    state.select_building(BuildingType::Drill);
    assert!(!state.demolish_mode);
}

#[test]
fn demolishing_a_run_of_conduits_refunds_each_of_them() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 8);
    state.unlock_building(BuildingType::Conduit);
    state.select_building(BuildingType::Conduit);

    let run: Vec<GridPos> = (1..=3).map(|x| GridPos::new(core.x + x, core.y)).collect();
    for pos in &run {
        state.grid.get_mut(*pos).unwrap().terrain = TerrainType::Empty;
        assert!(state.try_place_building(*pos));
    }
    let after_building = state.resources.minerals;

    state.toggle_demolish_mode();
    for pos in &run {
        assert!(state.try_sell_building(*pos));
    }

    assert!(state.resources.minerals > after_building);
    for pos in &run {
        assert!(state.grid.get(*pos).unwrap().building.is_none());
    }
}

#[test]
fn try_sell_building_refunds_half_cost_and_cannot_sell_core() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    assert!(!state.try_sell_building(core));
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message.contains("Core cannot be demolished")));

    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);
    let minerals_after_build = state.resources.minerals;

    assert!(state.try_sell_building(pos));
    assert!(state.resources.minerals > minerals_after_build);
    assert!(state.grid.get(pos).unwrap().building.is_none());
    assert_eq!(state.drones.total_count(), 0);
}
