use super::*;
use crate::data::GameConfig;

fn state() -> PlanetState {
    PlanetState::new(2, 42, GameConfig::default())
}

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

#[test]
fn try_harvest_terrain_converts_mountain_and_grants_minerals() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 5, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Mountain;
    state.grid.reveal_around(pos, 1);
    let before = state.resources.minerals;

    assert!(state.can_harvest(pos));
    assert!(state.try_harvest_terrain(pos));
    assert!(state.resources.minerals > before);
    let tile = state.grid.get(pos).unwrap();
    assert_eq!(tile.terrain, TerrainType::Rough);
    assert!(tile.mountain_harvested);
    assert!(!state.can_harvest(pos));
}

#[test]
fn research_makes_the_same_mountain_worth_more() {
    let harvest = |techs: &[&str]| {
        let mut state = state();
        for tech in techs {
            state.research.unlocked_techs.push((*tech).to_string());
        }
        state.refresh_stats();
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 5, core.y);
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Mountain;
        state.grid.reveal_around(pos, 1);
        let before = state.resources.minerals;
        assert!(state.try_harvest_terrain(pos));
        state.resources.minerals - before
    };

    assert!(
        harvest(&["excavation_charges"]) > harvest(&[]),
        "the charges paid for nothing"
    );
}

#[test]
fn try_harvest_terrain_fails_on_unrevealed_tile() {
    let mut state = state();
    let far_pos = GridPos::new(0, 0);
    state.grid.get_mut(far_pos).unwrap().terrain = TerrainType::Mountain;
    assert!(!state.can_harvest(far_pos));
    assert!(!state.try_harvest_terrain(far_pos));
}

#[test]
fn try_convert_forest_to_filter_requires_forest_terrain() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 5, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Forest;
    state.grid.reveal_around(pos, 1);

    assert!(state.try_convert_forest_to_filter(pos));
    let tile = state.grid.get(pos).unwrap();
    assert!(tile.filter);
    assert!(tile.forest_cleared);
    assert_eq!(tile.terrain, TerrainType::Rough);

    // Already converted: no longer forest, so a second attempt fails.
    assert!(!state.try_convert_forest_to_filter(pos));
}

#[test]
fn processor_boost_is_touch_toggleable_and_changes_live_power_demand() {
    let mut state = state();
    state
        .research
        .unlocked_techs
        .push("adaptive_clocking".into());
    state.auto_clocking = true;
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    state.config.resources.max_energy = 10_000.0;
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 2);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(pos));
    let normal = state.power_consumption();

    assert!(state.toggle_building_overclock(pos));
    assert!(
        !state.auto_clocking,
        "manual control should disable the policy"
    );
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
    assert!(state.power_consumption() > normal);
    assert!(state.toggle_building_overclock(pos));
    assert!(
        !state
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
fn processor_boost_waits_for_adaptive_clocking_research() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 2);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.get_mut(pos).unwrap().building =
        Some(crate::engine::Building::new(BuildingType::Smelter, pos));

    assert!(!state.toggle_building_overclock(pos));
    assert!(
        !state
            .grid
            .get(pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .overclocked
    );
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message.contains("Adaptive Clocking")));
}

#[test]
fn processor_input_priority_is_touch_toggleable_without_research() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let processor = GridPos::new(core.x + 1, core.y);
    let drill = GridPos::new(core.x + 2, core.y);
    state.grid.get_mut(processor).unwrap().building = Some(crate::engine::Building::new(
        BuildingType::Smelter,
        processor,
    ));
    state.grid.get_mut(drill).unwrap().building =
        Some(crate::engine::Building::new(BuildingType::Drill, drill));

    assert!(state.toggle_input_priority(processor));
    assert!(
        state
            .grid
            .get(processor)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .input_priority
    );
    assert!(state.toggle_input_priority(processor));
    assert!(!state.toggle_input_priority(drill));
}

#[test]
fn processor_standby_releases_power_and_preserves_buffers() {
    let mut state = state();
    state.resources.energy = 10_000.0;
    state.config.resources.max_energy = 10_000.0;
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.get_mut(pos).unwrap().building =
        Some(crate::engine::Building::new(BuildingType::Smelter, pos));
    state.input_buffers.insert((pos.x, pos.y), 12.0);
    state.output_buffers.insert((pos.x, pos.y), 4.0);
    state.grid.update_power_grid();
    let running_power = state.power_consumption();

    assert!(state.toggle_processor_standby(pos));
    assert!(state.power_consumption() < running_power);
    assert_eq!(state.input_buffers.get(&(pos.x, pos.y)), Some(&12.0));
    assert_eq!(state.output_buffers.get(&(pos.x, pos.y)), Some(&4.0));
    assert!(state.toggle_processor_standby(pos));
}

#[test]
fn box_selection_changes_every_processor_without_touching_other_buildings() {
    let mut state = state();
    state
        .research
        .unlocked_techs
        .push("adaptive_clocking".into());
    let core = state.grid.find_core().unwrap();
    let smelter = GridPos::new(core.x + 1, core.y);
    let assembler = GridPos::new(core.x + 2, core.y);
    let drill = GridPos::new(core.x + 3, core.y);
    state.grid.get_mut(smelter).unwrap().building =
        Some(crate::engine::Building::new(BuildingType::Smelter, smelter));
    state.grid.get_mut(assembler).unwrap().building = Some(crate::engine::Building::new(
        BuildingType::Assembler,
        assembler,
    ));
    state.grid.get_mut(drill).unwrap().building =
        Some(crate::engine::Building::new(BuildingType::Drill, drill));
    state.box_selected = vec![smelter, assembler, drill];

    assert_eq!(state.set_selected_overclock(true), 2);
    assert!(
        state
            .grid
            .get(smelter)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .overclocked
    );
    assert!(
        state
            .grid
            .get(assembler)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .overclocked
    );
    assert!(
        !state
            .grid
            .get(drill)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .overclocked
    );
    assert_eq!(state.set_selected_overclock(false), 2);
    assert_eq!(state.set_selected_input_priority(true), 2);
    assert!(
        state
            .grid
            .get(smelter)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .input_priority
    );
    assert!(
        state
            .grid
            .get(assembler)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .input_priority
    );
    assert!(
        !state
            .grid
            .get(drill)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .input_priority
    );
    assert_eq!(state.set_selected_input_priority(false), 2);
}

#[test]
fn processor_pad_purge_requires_the_same_visible_control_twice() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().building =
        Some(crate::engine::Building::new(BuildingType::Assembler, pos));
    state.input_hoppers.insert(
        (pos.x, pos.y),
        [(crate::engine::ResourceType::Minerals, 12.0)]
            .into_iter()
            .collect(),
    );
    state.output_buffers.insert((pos.x, pos.y), 30.0);

    assert!(!state.request_processor_pad_purge(pos));
    assert_eq!(state.purge_armed, Some(pos));
    assert_eq!(state.output_buffers.get(&(pos.x, pos.y)), Some(&30.0));
    assert!(state.request_processor_pad_purge(pos));
    assert_eq!(state.purge_armed, None);
    assert!(!state.output_buffers.contains_key(&(pos.x, pos.y)));
    assert_eq!(
        state.input_hoppers[&(pos.x, pos.y)][&crate::engine::ResourceType::Minerals],
        12.0
    );
}
