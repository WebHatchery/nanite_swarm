use super::*;

fn processor_at(state: &mut PlanetState, pos: GridPos) {
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.get_mut(pos).unwrap().building =
        Some(crate::engine::Building::new(BuildingType::Smelter, pos));
}

#[test]
fn standby_processor_rejects_manual_boost_until_resumed() {
    let mut state = state();
    state
        .research
        .unlocked_techs
        .push("adaptive_clocking".into());
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    processor_at(&mut state, pos);

    assert!(state.toggle_processor_standby(pos));
    assert!(!state.toggle_building_overclock(pos));
    let building = state.grid.get(pos).unwrap().building.as_ref().unwrap();
    assert!(building.standby);
    assert!(!building.overclocked);
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message.contains("Resume")));

    assert!(state.toggle_processor_standby(pos));
    assert!(state.toggle_building_overclock(pos));
    assert!(
        state
            .grid
            .get(pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .overclocked
    );
}

#[test]
fn bulk_boost_skips_standby_processors_in_a_mixed_selection() {
    let mut state = state();
    state
        .research
        .unlocked_techs
        .push("adaptive_clocking".into());
    let core = state.grid.find_core().unwrap();
    let active = GridPos::new(core.x + 1, core.y);
    let paused = GridPos::new(core.x + 2, core.y);
    processor_at(&mut state, active);
    processor_at(&mut state, paused);
    state
        .grid
        .get_mut(paused)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .standby = true;
    state.box_selected = vec![active, paused];

    assert_eq!(state.set_selected_overclock(true), 1);
    assert!(
        state
            .grid
            .get(active)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .overclocked
    );
    let paused_building = state.grid.get(paused).unwrap().building.as_ref().unwrap();
    assert!(paused_building.standby);
    assert!(!paused_building.overclocked);
}

#[test]
fn stamping_a_standby_blueprint_does_not_reactivate_its_boost() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(core, 3);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.resources.minerals = 1_000.0;
    state.resources.energy = 1_000.0;
    state.config.resources.max_energy = 1_000.0;
    state.unlock_building(BuildingType::Smelter);
    state.blueprint.push(crate::state::BlueprintEntry {
        offset: (0, 0),
        building_type: BuildingType::Smelter,
        overclocked: true,
        input_priority: true,
        standby: true,
    });

    assert_eq!(state.place_blueprint(pos), 1);
    let building = state.grid.get(pos).unwrap().building.as_ref().unwrap();
    assert!(building.standby);
    assert!(building.input_priority);
    assert!(!building.overclocked);
}

#[test]
fn restoring_a_standby_action_does_not_reactivate_its_boost() {
    let mut state = state();
    state.unlock_building(BuildingType::Smelter);
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(core, 3);
    processor_at(&mut state, pos);
    state.resources.minerals = 1_000.0;
    state.resources.energy = 1_000.0;
    state.config.resources.max_energy = 1_000.0;
    {
        let building = state.grid.get_mut(pos).unwrap().building.as_mut().unwrap();
        building.standby = true;
        building.overclocked = true;
    }
    assert!(state.try_sell_building(pos));
    assert!(state.undo_last_action());

    let building = state.grid.get(pos).unwrap().building.as_ref().unwrap();
    assert!(building.standby);
    assert!(!building.overclocked);
}
