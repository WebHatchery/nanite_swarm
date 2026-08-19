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
    assert_eq!(ore.peak_load, 0);
}

#[test]
fn flow_links_report_the_busiest_tile_on_their_route() {
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
        state.traffic.insert((pos.x, pos.y), step as u32 + 4);
    }
    let drill = GridPos::new(core.x + 4, core.y);
    state.grid.get_mut(drill).unwrap().terrain = TerrainType::Empty;
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));
    state.grid.update_power_grid();

    let ore = factory_flow_links(&state)
        .into_iter()
        .find(|link| link.resource == ResourceType::Minerals)
        .expect("drill supply route");
    assert_eq!(ore.peak_load, 7);
    assert_eq!(ore.capacity, state.config.buildings.conduit_capacity);
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

#[test]
fn factory_ledger_names_the_missing_input_and_boosted_processor() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let pos = state.grid.find_core().unwrap();
    state.grid.get_mut(pos).unwrap().building =
        Some(crate::engine::Building::new(BuildingType::Assembler, pos));
    let building = state.grid.get_mut(pos).unwrap().building.as_mut().unwrap();
    building.powered = true;
    building.overclocked = true;
    building.input_priority = true;
    state.input_hoppers.insert(
        (pos.x, pos.y),
        [(ResourceType::Alloy, 8.0)].into_iter().collect(),
    );

    let ledger = factory_ledger(&state);
    assert_eq!(ledger.processors, 1);
    assert_eq!(ledger.active, 0);
    assert_eq!(ledger.starved, 1);
    assert_eq!(ledger.boosted, 1);
    assert_eq!(ledger.priority, 1);
    assert_eq!(ledger.standby, 0);
    assert_eq!(ledger.bottleneck, Some(ResourceType::Minerals));
    assert!(ledger.components_capacity > 0.0);
}

#[test]
fn rated_output_capacity_follows_boost_and_dust_efficiency() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let pos = state.grid.find_core().unwrap();
    let mut building = crate::engine::Building::new(BuildingType::Smelter, pos);
    state.grid.get_mut(pos).unwrap().building = Some(building.clone());
    let normal = processor_output_capacity(&state, ResourceType::Alloy);

    building.overclocked = true;
    building.dust = 30.0;
    state.grid.get_mut(pos).unwrap().building = Some(building);
    let boosted_and_dusty = processor_output_capacity(&state, ResourceType::Alloy);
    assert!((boosted_and_dusty - normal * 1.5 * 0.9).abs() < 0.001);
}

#[test]
fn flow_node_output_gauge_uses_the_real_dispatch_pad_capacity() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let pos = state.grid.find_core().unwrap();
    let mut building = crate::engine::Building::new(BuildingType::Smelter, pos);
    building.input_priority = true;
    state.grid.get_mut(pos).unwrap().building = Some(building);
    state
        .output_buffers
        .insert((pos.x, pos.y), state.processor_pad_capacity() * 0.5);

    let node = factory_flow_nodes(&state).into_iter().next().unwrap();
    assert!((node.output_pressure - 0.5).abs() < 0.001);
    assert!(node.priority);
}
