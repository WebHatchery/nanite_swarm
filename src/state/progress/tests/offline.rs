//! Offline simulation and storage-cap safety coverage.

use super::{approx_eq, state};
use crate::engine::{BuildingType, GridPos, ResourceType, TerrainType};
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
