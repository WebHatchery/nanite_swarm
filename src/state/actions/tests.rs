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
