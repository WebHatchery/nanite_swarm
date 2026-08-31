//! Research, production-rate, power, and live-efficiency coverage.

use super::{approx_eq, state};
use crate::engine::{BuildingType, GridPos, TerrainType};
use crate::state::{FactoryFocus, PlanetState};

/// Every tech that claims a number in `research.json` has to move one here.
/// These are the five that shipped as no-ops.
#[test]
fn researching_storage_optimization_raises_the_mineral_cap() {
    let mut state = PlanetState::default();
    let before = state.mineral_capacity();
    state
        .research
        .unlocked_techs
        .push("storage_optimization".into());
    state.refresh_stats();
    assert_eq!(state.mineral_capacity(), before + 50.0);
}

#[test]
fn buffer_lattices_widen_every_processor_dispatch_pad() {
    let mut state = PlanetState::default();
    let before = state.processor_pad_capacity();
    state.research.unlocked_techs.push("buffer_lattices".into());
    state.refresh_stats();
    assert_eq!(state.processor_pad_capacity(), before * 1.5);
}

#[test]
fn researching_power_efficiency_lowers_consumption() {
    let mut state = PlanetState::default();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    let before = state.power_consumption();
    assert!(before > 0.0);
    state
        .research
        .unlocked_techs
        .push("power_efficiency".to_string());
    state.refresh_stats();
    assert!(approx_eq(state.power_consumption(), before * 0.75, 1e-4));
}

#[test]
fn live_resource_rates_follow_active_buildings_and_dust_research() {
    let mut plain = state();
    let plain_core = plain.grid.find_core().unwrap();
    plain
        .grid
        .get_mut(plain_core)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 26.0;
    let plain_data = plain.data_generation_rate();

    let mut researched = state();
    let researched_core = researched.grid.find_core().unwrap();
    researched
        .grid
        .get_mut(researched_core)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 26.0;
    researched
        .research
        .unlocked_techs
        .push("self_cleaning_servos".to_string());
    researched.refresh_stats();
    assert!(researched.data_generation_rate() > plain_data);

    let mut server_state = state();
    let server_core = server_state.grid.find_core().unwrap();
    let server = GridPos::new(server_core.x + 1, server_core.y);
    server_state.grid.reveal_around(server, 1);
    server_state.grid.get_mut(server).unwrap().terrain = TerrainType::Empty;
    server_state.unlock_building(BuildingType::ServerBank);
    server_state.select_building(BuildingType::ServerBank);
    assert!(server_state.try_place_building(server));
    assert!(server_state.data_generation_rate() > state().data_generation_rate());

    let drill = GridPos::new(researched_core.x + 1, researched_core.y);
    researched.grid.reveal_around(drill, 1);
    researched.grid.get_mut(drill).unwrap().terrain = TerrainType::Empty;
    researched.select_building(BuildingType::Drill);
    assert!(researched.try_place_building(drill));
    researched
        .grid
        .get_mut(drill)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 26.0;
    assert_eq!(researched.mineral_output_rate(), 5.0);

    let biomass = GridPos::new(researched_core.x - 1, researched_core.y);
    researched.grid.reveal_around(biomass, 1);
    let biomass_tile = researched.grid.get_mut(biomass).unwrap();
    biomass_tile.terrain = TerrainType::Forest;
    biomass_tile.biomass_amount = researched.config.resources.forest_biomass;
    researched.unlock_building(BuildingType::BiomassHarvester);
    researched.select_building(BuildingType::BiomassHarvester);
    assert!(researched.try_place_building(biomass));
    assert_eq!(researched.biomass_harvest_rate(), 1.0);
}

#[test]
fn selected_building_power_rate_follows_live_state() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let drill = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(drill, 1);
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));

    assert!(approx_eq(
        state.building_power_delta(drill).unwrap(),
        -2.0,
        1e-4
    ));

    state
        .grid
        .get_mut(drill)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 75.0;
    assert!(approx_eq(
        state.building_power_delta(drill).unwrap(),
        -2.4,
        1e-4
    ));

    state.factory_focus = FactoryFocus::Extraction;
    assert!(approx_eq(
        state.building_power_delta(drill).unwrap(),
        -2.76,
        1e-4
    ));

    state
        .research
        .unlocked_techs
        .push("power_efficiency".to_string());
    state.refresh_stats();
    // Research lowers the normal draw; the focus tax remains the declared
    // operating surcharge, matching the global power balance.
    assert!(approx_eq(
        state.building_power_delta(drill).unwrap(),
        -2.16,
        1e-4
    ));

    state
        .grid
        .get_mut(drill)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .standby = true;
    assert!(approx_eq(
        state.building_power_delta(drill).unwrap(),
        0.0,
        1e-4
    ));
}

#[test]
fn selected_processor_power_rate_includes_overclock_and_focus_tax() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let smelter = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(smelter, 1);
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(smelter));

    state
        .grid
        .get_mut(smelter)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .overclocked = true;
    assert!(approx_eq(
        state.building_power_delta(smelter).unwrap(),
        -7.0,
        1e-4
    ));

    state.factory_focus = FactoryFocus::Refining;
    assert!(approx_eq(
        state.building_power_delta(smelter).unwrap(),
        -8.05,
        1e-4
    ));
}

#[test]
fn biomass_harvester_power_rate_reads_live_fuel_output() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let forest = GridPos::new(core.x - 1, core.y);
    state.grid.reveal_around(forest, 1);
    let tile = state.grid.get_mut(forest).unwrap();
    tile.terrain = TerrainType::Forest;
    tile.biomass_amount = state.config.resources.forest_biomass;
    state.unlock_building(BuildingType::BiomassHarvester);
    state.select_building(BuildingType::BiomassHarvester);
    assert!(state.try_place_building(forest));

    let dust_response = state.resolved_dust_response();
    let expected_generation = state.grid.total_power_generation_with(&dust_response)
        + state.config.resources.biomass_power_output;
    assert!(approx_eq(
        state.power_generation(),
        expected_generation,
        1e-4
    ));
    assert!(approx_eq(
        state.building_power_delta(forest).unwrap(),
        state.config.resources.biomass_power_output,
        1e-4
    ));
    state
        .grid
        .get_mut(forest)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 75.0;
    assert!(approx_eq(
        state.building_power_delta(forest).unwrap(),
        state.config.resources.biomass_power_output * 0.7,
        1e-4
    ));
}

#[test]
fn iron_belt_drill_rate_includes_the_local_output_bonus() {
    let mut state = PlanetState::new(0, 42, crate::data::GameConfig::default());
    let core = state.grid.find_core().unwrap();
    let drill = GridPos::new(core.x + 1, core.y);
    let tile = state.grid.get_mut(drill).unwrap();
    tile.terrain = TerrainType::Empty;
    tile.ore_richness = 1.0;
    tile.revealed = true;
    state.features[0].bounds = (0.45, 0.45, 0.20, 0.20);
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));

    assert_eq!(state.feature_ore_multiplier_at(drill), Some(1.5));
    assert!(approx_eq(state.mineral_output_rate(), 7.5, 1e-4));
}

#[test]
fn researching_drone_capacity_grows_existing_drones_too() {
    let mut state = PlanetState::default();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);
    let before = state.drones.drone_capacity;

    state.research.unlocked_techs.push("drone_capacity".into());
    state.refresh_stats();

    assert_eq!(state.drones.drone_capacity, before * 2.0);
    assert_eq!(state.drones.drones()[0].capacity, before * 2.0);
}

#[test]
fn researching_efficient_drills_and_advanced_research_speeds_production() {
    let mut plain = PlanetState::default();
    let mut upgraded = PlanetState::default();
    upgraded
        .research
        .unlocked_techs
        .extend(["efficient_drills".into(), "advanced_research".into()]);
    upgraded.refresh_stats();

    for state in [&mut plain, &mut upgraded] {
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);
        state.resources.minerals = 0.0;
        // Both runs would otherwise clamp at the storage cap.
        state.config.resources.base_mineral_cap = 100_000.0;
        for _ in 0..300 {
            state.step(0.1, false);
        }
    }

    assert!(upgraded.resources.minerals > plain.resources.minerals);
    assert!(upgraded.resources.data > plain.resources.data);
}
