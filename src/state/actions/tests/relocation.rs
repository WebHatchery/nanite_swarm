use super::*;

#[test]
fn relocating_a_drill_keeps_its_cargo_and_drone_crew_without_a_rebuild_cost() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let source = GridPos::new(core.x + 1, core.y);
    let destination = GridPos::new(core.x + 2, core.y);
    state.grid.reveal_around(core, 4);
    for pos in [source, destination] {
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    }
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(source));
    state.output_buffers.insert((source.x, source.y), 17.5);
    state.selected_tile = Some(source);
    let minerals = state.resources.minerals;
    let energy = state.resources.energy;

    assert!(state.relocate_building(source, destination));
    assert_eq!(state.selected_building, None);
    assert_eq!(state.selected_tile, None);
    assert!(state.grid.get(source).unwrap().building.is_none());
    assert_eq!(
        state
            .grid
            .get(destination)
            .and_then(|tile| tile.building.as_ref())
            .map(|building| (building.building_type, building.position)),
        Some((BuildingType::Drill, destination))
    );
    assert_eq!(
        state.output_buffers.get(&(destination.x, destination.y)),
        Some(&17.5)
    );
    assert!(!state.output_buffers.contains_key(&(source.x, source.y)));
    assert_eq!(state.drones.drones_at(destination).len(), 1);
    assert_eq!(state.resources.minerals, minerals);
    assert_eq!(state.resources.energy, energy);
}

#[test]
fn relocating_a_processor_keeps_typed_buffers_and_operating_flags() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let source = GridPos::new(core.x + 1, core.y);
    let destination = GridPos::new(core.x + 2, core.y);
    state.grid.reveal_around(core, 4);
    for pos in [source, destination] {
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    }
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(source));
    state.input_buffers.insert((source.x, source.y), 8.0);
    state.output_buffers.insert((source.x, source.y), 4.5);
    state.input_hoppers.insert(
        (source.x, source.y),
        [(crate::engine::ResourceType::Minerals, 8.0)]
            .into_iter()
            .collect(),
    );
    {
        let building = state
            .grid
            .get_mut(source)
            .unwrap()
            .building
            .as_mut()
            .unwrap();
        building.overclocked = true;
        building.input_priority = true;
        building.standby = true;
    }
    let minerals = state.resources.minerals;
    let energy = state.resources.energy;

    assert!(state.relocate_building(source, destination));
    assert_eq!(
        state.input_buffers.get(&(destination.x, destination.y)),
        Some(&8.0)
    );
    assert_eq!(
        state.output_buffers.get(&(destination.x, destination.y)),
        Some(&4.5)
    );
    assert_eq!(
        state
            .input_hoppers
            .get(&(destination.x, destination.y))
            .and_then(|hopper| hopper.get(&crate::engine::ResourceType::Minerals)),
        Some(&8.0)
    );
    let building = state
        .grid
        .get(destination)
        .unwrap()
        .building
        .as_ref()
        .unwrap();
    assert!(building.overclocked);
    assert!(building.input_priority);
    assert!(building.standby);
    assert_eq!(state.resources.minerals, minerals);
    assert_eq!(state.resources.energy, energy);
}

#[test]
fn relocating_shipping_structures_keeps_half_loaded_and_landed_cargo() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let driver_source = GridPos::new(core.x + 1, core.y);
    let driver_destination = GridPos::new(core.x + 2, core.y);
    let pad_source = GridPos::new(core.x + 1, core.y + 1);
    let pad_destination = GridPos::new(core.x + 2, core.y + 1);
    state.grid.reveal_around(core, 5);
    for pos in [
        driver_source,
        driver_destination,
        pad_source,
        pad_destination,
    ] {
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    }
    state.resources.minerals = 1_000.0;
    state.resources.energy = 1_000.0;
    state.config.resources.max_energy = 1_000.0;
    state.unlock_building(BuildingType::MassDriver);
    state.unlock_building(BuildingType::LandingPad);
    state.select_building(BuildingType::MassDriver);
    assert!(state.try_place_building(driver_source));
    state.select_building(BuildingType::LandingPad);
    assert!(state.try_place_building(pad_source));
    state
        .pod_loads
        .insert((driver_source.x, driver_source.y), 32.0);
    state
        .output_buffers
        .insert((pad_source.x, pad_source.y), 44.0);
    state.pad_cargo.insert(
        (pad_source.x, pad_source.y),
        crate::engine::ResourceType::Alloy,
    );
    let minerals = state.resources.minerals;
    let energy = state.resources.energy;

    assert!(state.relocate_building(driver_source, driver_destination));
    assert!(state.relocate_building(pad_source, pad_destination));
    assert_eq!(
        state
            .pod_loads
            .get(&(driver_destination.x, driver_destination.y)),
        Some(&32.0)
    );
    assert!(!state
        .pod_loads
        .contains_key(&(driver_source.x, driver_source.y)));
    assert_eq!(
        state
            .output_buffers
            .get(&(pad_destination.x, pad_destination.y)),
        Some(&44.0)
    );
    assert_eq!(
        state.pad_cargo.get(&(pad_destination.x, pad_destination.y)),
        Some(&crate::engine::ResourceType::Alloy)
    );
    assert_eq!(state.resources.minerals, minerals);
    assert_eq!(state.resources.energy, energy);
}

#[test]
fn undoing_a_relocation_restores_the_original_tile_and_cargo() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let source = GridPos::new(core.x + 1, core.y);
    let destination = GridPos::new(core.x + 2, core.y);
    state.grid.reveal_around(core, 4);
    for pos in [source, destination] {
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    }
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(source));
    state.output_buffers.insert((source.x, source.y), 9.0);
    state.undo_history.clear();

    assert!(state.relocate_building(source, destination));
    assert!(state.undo_last_action());
    assert!(state.grid.get(destination).unwrap().building.is_none());
    assert_eq!(state.output_buffers.get(&(source.x, source.y)), Some(&9.0));
    assert!(state.grid.get(source).unwrap().building.is_some());
    assert_eq!(state.selected_building, None);
    assert_eq!(state.selected_tile, None);
    assert!(state.undo_history.is_empty());
}
