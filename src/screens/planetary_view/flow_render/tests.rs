use super::*;
use crate::data::GameConfig;
use crate::engine::TerrainType;

#[test]
fn assembler_readiness_uses_the_leanest_of_both_routed_inputs() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let pos = state.grid.find_core().unwrap();
    let recipe = &crate::data::game_data()
        .buildings
        .iter()
        .find(|def| def.id == "assembler")
        .unwrap()
        .recipe;
    state.input_hoppers.insert(
        (pos.x, pos.y),
        [(ResourceType::Minerals, 10.0), (ResourceType::Alloy, 2.5)]
            .into_iter()
            .collect(),
    );

    assert!((recipe_readiness(&state, pos, recipe) - 0.5).abs() < 0.001);
    state
        .input_hoppers
        .get_mut(&(pos.x, pos.y))
        .unwrap()
        .insert(ResourceType::Minerals, 0.0);
    assert_eq!(recipe_readiness(&state, pos, recipe), 0.0);
}

#[test]
fn a_drill_flow_falls_back_to_the_core_when_no_processor_wants_it() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 12);
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    state.config.resources.max_energy = 10_000.0;
    state.unlock_building(BuildingType::Conduit);

    for step in 1..=3 {
        let pos = GridPos::new(core.x + step, core.y);
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
        state.select_building(BuildingType::Conduit);
        assert!(state.try_place_building(pos));
    }
    let drill = GridPos::new(core.x + 4, core.y);
    state.grid.get_mut(drill).unwrap().terrain = TerrainType::Empty;
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));
    state.grid.update_power_grid();

    let links = factory_flow_links(&state);
    let ore = links
        .iter()
        .find(|link| link.resource == ResourceType::Minerals)
        .expect("drill supply route");
    assert_eq!(ore.path.first(), Some(&drill));
    assert_eq!(ore.path.last(), Some(&core));
}

#[test]
fn recipe_icons_follow_resource_order_not_hash_map_order() {
    let ordered = ordered_resources(["components", "minerals", "alloy"].into_iter());
    assert_eq!(
        ordered,
        [
            ResourceType::Minerals,
            ResourceType::Alloy,
            ResourceType::Components
        ]
    );
}
