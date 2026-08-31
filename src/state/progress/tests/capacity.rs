//! Storage-capacity contract coverage.

use super::state;
use crate::engine::{BuildingType, GridPos, ResourceType};

#[test]
fn mineral_capacity_grows_with_storage_buildings() {
    let mut state = state();
    let base = state.mineral_capacity();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::Storage);
    state.select_building(BuildingType::Storage);
    state.try_place_building(pos);
    assert!(state.mineral_capacity() > base);
}

#[test]
fn every_resource_reports_the_same_cap_used_by_simulation() {
    let state = state();
    assert_eq!(state.resource_capacity(ResourceType::Energy), 50.0);
    assert_eq!(
        state.resource_capacity(ResourceType::Minerals),
        state.mineral_capacity()
    );
    for resource in [
        ResourceType::Data,
        ResourceType::Biomass,
        ResourceType::Alloy,
        ResourceType::Components,
    ] {
        assert_eq!(state.resource_capacity(resource), 1_000.0);
    }
}
