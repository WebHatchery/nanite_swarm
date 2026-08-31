//! Runtime coverage for dust, terrain buffers, corrosion, and cooling.

use super::state;
use crate::data::GameConfig;
use crate::engine::{BuildingType, GridPos, TerrainType};
use crate::state::PlanetState;

#[test]
fn the_dust_rate_is_data_rather_than_a_constant() {
    let mut slow = GameConfig::default();
    slow.upkeep.dust_rate = 0.05;
    let mut fast = GameConfig::default();
    fast.upkeep.dust_rate = 0.5;

    let settled = |config: GameConfig| {
        let mut state = PlanetState::new(2, 42, config);
        let core = state.grid.find_core().unwrap();
        for _ in 0..300 {
            state.step(crate::state::TICK_SECONDS, false);
        }
        state
            .grid
            .get(core)
            .and_then(|tile| tile.building.as_ref())
            .map(|building| building.dust)
            .unwrap_or(0.0)
    };

    assert!(
        settled(fast) > settled(slow),
        "a bigger declared dust rate should settle more dust"
    );
}

#[test]
fn a_sweepers_reach_is_the_one_the_config_declares() {
    let mut config = GameConfig::default();
    config.upkeep.sweeper_radius = 9;
    let state = PlanetState::new(2, 42, config);
    assert_eq!(state.coverage_radius(BuildingType::Sweeper), Some(9));
    assert_eq!(
        state.coverage_radius(BuildingType::ShieldGenerator),
        Some(state.config.upkeep.hazard_counter_radius)
    );
}

#[test]
fn dust_accumulates_on_powered_buildings_over_time() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    state.update_dust(10.0);
    let dust = state.grid.get(pos).unwrap().building.as_ref().unwrap().dust;
    assert!(dust > 0.0);
}

fn server_bank_heat_on_ice_rift(on_rift: bool) -> f32 {
    let mut state = PlanetState::new(3, 42, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    let water = GridPos::new(core.x + 2, core.y);
    state.grid.reveal_around(pos, 2);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.get_mut(water).unwrap().terrain = TerrainType::Water;
    state.features[0].bounds = if on_rift {
        (0.45, 0.45, 0.20, 0.20)
    } else {
        (0.0, 0.0, 0.05, 0.05)
    };
    state.resources.minerals = 1_000.0;
    state.resources.energy = 1_000.0;
    state.config.buildings.server_bank_heat_per_second = 4.0;
    state.config.buildings.water_cooling_rate = 2.0;
    state.unlock_building(BuildingType::ServerBank);
    state.select_building(BuildingType::ServerBank);
    assert!(state.try_place_building(pos));
    state.grid.update_power_grid();

    state.update_heat(1.0);
    state
        .grid
        .get(pos)
        .and_then(|tile| tile.building.as_ref())
        .map(|building| building.heat)
        .unwrap_or(0.0)
}

#[test]
fn ice_rift_amplifies_water_cooling_for_server_banks() {
    let ordinary = server_bank_heat_on_ice_rift(false);
    let rift = server_bank_heat_on_ice_rift(true);

    assert!((ordinary - 2.0).abs() < 0.001);
    assert!((rift - 1.0).abs() < 0.001);
    assert!(rift < ordinary);
}

fn conduit_wear_on_feature(on_feature: bool) -> (f32, f32) {
    let mut state = PlanetState::new(1, 42, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    if let Some(tile) = state.grid.get_mut(pos) {
        tile.terrain = TerrainType::Empty;
        tile.revealed = true;
    }
    state.features[0].bounds = if on_feature {
        (0.45, 0.45, 0.20, 0.20)
    } else {
        (0.0, 0.0, 0.05, 0.05)
    };
    state.unlock_building(BuildingType::Conduit);
    state.select_building(BuildingType::Conduit);
    assert!(state.try_place_building(pos));
    state.grid.update_power_grid();

    state.update_dust(10.0);
    state
        .grid
        .get(pos)
        .and_then(|tile| tile.building.as_ref())
        .map(|building| (building.dust, building.acid_wear))
        .unwrap_or((0.0, 0.0))
}

#[test]
fn acid_sea_increases_local_corrosion_without_changing_dust() {
    let ordinary = conduit_wear_on_feature(false);
    let basin = conduit_wear_on_feature(true);

    assert!(ordinary.0 > 0.0);
    assert!((basin.0 - ordinary.0).abs() < 0.001);
    assert!(
        (basin.1 - ordinary.1 * 1.8).abs() < 0.001,
        "Acid Sea corrosion was {}, expected 1.8x ordinary exposure {}",
        basin.1,
        ordinary.1
    );
}

/// Dust on a drill with an optional intact forest placed at a known distance.
fn drill_dust_with_forest_at(distance: Option<i32>) -> f32 {
    let mut state = state();
    state.config.upkeep.forest_buffer_radius = 2;
    state.config.upkeep.forest_buffer_multiplier = 0.5;
    // The default temperate world has forests of its own. Clear the fixture
    // first so the comparison names exactly one preserved tile.
    for (_, tile) in state.grid.iter_tiles_mut() {
        if tile.terrain == TerrainType::Forest {
            tile.terrain = TerrainType::Empty;
            tile.biomass_amount = 0.0;
        }
    }

    let core = state.grid.find_core().unwrap();
    let drill = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(drill, 4);
    state.grid.get_mut(drill).unwrap().terrain = TerrainType::Empty;
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));

    if let Some(distance) = distance {
        let forest = GridPos::new(drill.x, drill.y + distance);
        let tile = state.grid.get_mut(forest).unwrap();
        tile.terrain = TerrainType::Forest;
        tile.filter = false;
        tile.forest_cleared = false;
    }

    state.update_dust(10.0);
    state
        .grid
        .get(drill)
        .and_then(|tile| tile.building.as_ref())
        .map(|building| building.dust)
        .unwrap_or(0.0)
}

#[test]
fn an_intact_forest_buffers_nearby_dust() {
    let bare = drill_dust_with_forest_at(None);
    let buffered = drill_dust_with_forest_at(Some(1));

    assert!(
        buffered < bare,
        "nearby forest left {buffered} dust versus bare ground at {bare}"
    );
}

#[test]
fn a_preserved_forest_stops_buffering_at_its_declared_radius() {
    let bare = drill_dust_with_forest_at(None);
    let outside = drill_dust_with_forest_at(Some(3));

    assert!(
        (outside - bare).abs() < 0.001,
        "forest outside the radius changed dust from {bare} to {outside}"
    );
}
