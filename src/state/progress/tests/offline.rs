//! Offline simulation and storage-cap safety coverage.

use super::{approx_eq, state};
use crate::engine::{Building, BuildingType, GridPos, ResourceType, TerrainType};
use crate::state::PlanetState;

#[test]
fn offline_simulation_uses_hibernation_rate() {
    let mut state = PlanetState {
        battery_seconds: 4.0 * 60.0 * 60.0,
        ..Default::default()
    };

    let offline = 6.0 * 60.0 * 60.0;
    state.apply_offline_progress(offline);

    let expected_sim = (4.0 * 60.0 * 60.0) + (2.0 * 60.0 * 60.0) * 0.1;
    assert!(approx_eq(state.last_offline_simulated, expected_sim, 0.5));
    assert!(approx_eq(state.last_offline_seconds, offline, 0.5));
    assert!(state.battery_seconds <= 0.0);
    assert!(state.offline_notice_timer > 0.0);
}

#[test]
fn apply_offline_progress_is_noop_for_zero_or_negative_duration() {
    let mut state = PlanetState::default();
    state.apply_offline_progress(0.0);
    assert_eq!(state.last_offline_seconds, 0.0);
    assert_eq!(state.last_offline_simulated, 0.0);
}

#[test]
fn offline_progress_respects_live_storage_caps() {
    let mut state = state();
    state.config.offline.max_resource_gain = 10_000.0;
    state.config.resources.max_energy = 10_000.0;
    state.config.resources.base_mineral_cap = 100_000.0;
    state.config.upkeep.dust_rate = 0.0;
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    let core = state.grid.find_core().unwrap();
    for (building, pos) in [
        (BuildingType::Smelter, GridPos::new(core.x + 1, core.y)),
        (BuildingType::Assembler, GridPos::new(core.x, core.y + 1)),
    ] {
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
        state.grid.reveal_around(pos, 1);
        state.unlock_building(building);
        state.select_building(building);
        assert!(state.try_place_building(pos));
    }
    state.grid.update_power_grid();
    state.resources.data = 999.0;
    state.resources.biomass = 999.0;
    state.resources.alloy = 999.0;
    state.resources.components = 999.0;
    state.apply_offline_progress(10_000.0);

    for resource in [
        ResourceType::Data,
        ResourceType::Biomass,
        ResourceType::Alloy,
        ResourceType::Components,
    ] {
        assert!(
            state.resources.get(resource) <= state.resource_capacity(resource),
            "offline progress exceeded the {:?} cap",
            resource
        );
    }
    assert_eq!(state.resources.data, 1_000.0);
    assert_eq!(state.resources.alloy, 1_000.0);
    assert_eq!(state.resources.components, 1_000.0);
}

#[test]
fn offline_biomass_harvester_consumes_only_available_forest_and_reports_yield() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let forest = GridPos::new(core.x - 1, core.y);
    state.grid.reveal_around(forest, 1);
    let biomass_start = state.config.resources.forest_biomass;
    let tile = state.grid.get_mut(forest).unwrap();
    tile.terrain = TerrainType::Forest;
    tile.biomass_amount = biomass_start;
    state.unlock_building(BuildingType::BiomassHarvester);
    state.select_building(BuildingType::BiomassHarvester);
    assert!(state.try_place_building(forest));

    state.apply_offline_progress(biomass_start + 30.0);

    assert!(approx_eq(
        state.last_offline_report.biomass_gained,
        biomass_start,
        0.001
    ));
    assert!(approx_eq(state.resources.biomass, biomass_start, 0.001));
    let tile = state.grid.get(forest).unwrap();
    assert_eq!(tile.terrain, TerrainType::Empty);
    assert!(approx_eq(tile.biomass_amount, 0.0, 0.001));
    assert!(approx_eq(state.biomass_harvest_rate(), 0.0, 0.001));
    assert!(approx_eq(state.power_balance, state.net_power(), 0.001));
}

#[test]
fn offline_drills_apply_feature_bonus_and_deplete_richness_bonus() {
    let mut state = PlanetState::new(0, 42, crate::data::GameConfig::default());
    let core = state.grid.find_core().unwrap();
    let drill = GridPos::new(core.x + 1, core.y);
    let tile = state.grid.get_mut(drill).unwrap();
    tile.terrain = TerrainType::Empty;
    tile.ore_richness = 2.0;
    tile.revealed = true;
    state.features[0].bounds = (0.45, 0.45, 0.20, 0.20);
    state.config.resources.base_mineral_cap = 1_000.0;
    state.config.ore.depletion_per_unit = 0.01;
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));
    let minerals_before = state.resources.minerals;

    state.apply_offline_progress(10.0);

    assert!(approx_eq(
        state.last_offline_report.minerals_gained,
        150.0,
        0.001
    ));
    assert!(approx_eq(
        state.resources.minerals - minerals_before,
        150.0,
        0.001
    ));
    assert!(approx_eq(
        state.grid.get(drill).unwrap().ore_richness,
        1.0,
        0.001
    ));
}

#[test]
fn offline_catch_up_advances_dust_and_server_heat() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let server = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(server, 1);
    state.grid.get_mut(server).unwrap().terrain = TerrainType::Empty;
    state.config.upkeep.dust_rate = 1.0;
    state.config.buildings.server_bank_heat_per_second = 1.0;
    state.unlock_building(BuildingType::ServerBank);
    state.select_building(BuildingType::ServerBank);
    assert!(state.try_place_building(server));

    state.apply_offline_progress(10.0);

    let building = state.grid.get(server).unwrap().building.as_ref().unwrap();
    assert!(approx_eq(building.dust, 10.0, 0.001));
    assert!(approx_eq(building.heat, 10.0, 0.001));
}

#[test]
fn offline_catch_up_honors_adaptive_clocking_for_a_fed_processor() {
    let mut state = PlanetState::new(2, 42, crate::data::GameConfig::default());
    state
        .research
        .unlocked_techs
        .push("adaptive_clocking".to_string());
    state.auto_clocking = true;
    let core = state.grid.find_core().unwrap();
    let processor = GridPos::new(core.x + 1, core.y);
    let turbine = GridPos::new(core.x - 1, core.y);
    for pos in [processor, turbine] {
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
        state.grid.reveal_around(pos, 1);
    }
    state.grid.get_mut(processor).unwrap().building =
        Some(Building::new(BuildingType::Smelter, processor));
    state.grid.get_mut(turbine).unwrap().building =
        Some(Building::new(BuildingType::WindTurbine, turbine));
    state.grid.update_power_grid();
    state.input_hoppers.insert(
        (processor.x, processor.y),
        [(ResourceType::Minerals, 20.0)].into_iter().collect(),
    );

    state.apply_offline_progress(1.0);

    assert!(
        state
            .grid
            .get(processor)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .overclocked
    );
    assert!(approx_eq(state.resources.alloy, 1.5, 0.001));
}
