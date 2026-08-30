use super::*;
use crate::data::GameConfig;

mod action_feedback;
mod blueprints;
mod processor_controls;

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
fn a_conduit_route_lays_every_segment_between_two_tiles() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 8);
    state.unlock_building(BuildingType::Conduit);
    state.select_building(BuildingType::Conduit);

    let end = GridPos::new(core.x + 3, core.y);
    for x in (core.x + 1)..=end.x {
        state.grid.get_mut(GridPos::new(x, core.y)).unwrap().terrain = TerrainType::Empty;
    }

    assert!(state.try_place_conduit_path(core, end));
    for x in (core.x + 1)..=end.x {
        assert_eq!(
            state
                .grid
                .get(GridPos::new(x, core.y))
                .and_then(|tile| tile.building.as_ref())
                .map(|building| building.building_type),
            Some(BuildingType::Conduit)
        );
    }
}

#[test]
fn smart_conduit_route_starts_at_an_existing_building() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 8);
    state.unlock_building(BuildingType::Conduit);

    let end = GridPos::new(core.x + 3, core.y);
    for x in (core.x + 1)..=end.x {
        state.grid.get_mut(GridPos::new(x, core.y)).unwrap().terrain = TerrainType::Empty;
    }

    assert!(state.try_place_smart_conduit_path(core, end));
    for x in (core.x + 1)..=end.x {
        assert_eq!(
            state
                .grid
                .get(GridPos::new(x, core.y))
                .and_then(|tile| tile.building.as_ref())
                .map(|building| building.building_type),
            Some(BuildingType::Conduit)
        );
    }
}

#[test]
fn smart_conduit_route_rejects_an_empty_source_tile() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 8);
    state.unlock_building(BuildingType::Conduit);
    let source = GridPos::new(core.x + 1, core.y);
    let end = GridPos::new(core.x + 3, core.y);
    for x in (core.x + 1)..=end.x {
        state.grid.get_mut(GridPos::new(x, core.y)).unwrap().terrain = TerrainType::Empty;
    }

    assert!(!state.try_place_smart_conduit_path(source, end));
    assert!(state.grid.get(source).unwrap().building.is_none());
    assert!(state.grid.get(end).unwrap().building.is_none());
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
    assert_eq!(state.set_selected_standby(true), 2);
    assert!(
        state
            .grid
            .get(smelter)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .standby
    );
    assert!(
        state
            .grid
            .get(assembler)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .standby
    );
    assert!(
        !state
            .grid
            .get(drill)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .standby
    );
    assert_eq!(state.set_selected_standby(false), 2);
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

#[test]
fn selected_pad_purge_requires_two_taps_and_only_clears_blocked_selection() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let blocked = GridPos::new(core.x + 1, core.y);
    let partial = GridPos::new(core.x + 2, core.y);
    let unselected = GridPos::new(core.x + 3, core.y);
    for pos in [blocked, partial, unselected] {
        let mut building = crate::engine::Building::new(BuildingType::Smelter, pos);
        building.powered = true;
        state.grid.get_mut(pos).unwrap().building = Some(building);
    }
    let capacity = state.processor_pad_capacity();
    state
        .output_buffers
        .insert((blocked.x, blocked.y), capacity);
    state.output_buffers.insert((partial.x, partial.y), 4.0);
    state
        .output_buffers
        .insert((unselected.x, unselected.y), capacity);
    state.input_buffers.insert((blocked.x, blocked.y), 12.0);
    state.box_selected = vec![blocked, partial];

    assert_eq!(state.request_selected_pad_purge(), 0);
    assert!(state.bulk_purge_armed);
    assert_eq!(
        state.output_buffers.get(&(blocked.x, blocked.y)),
        Some(&capacity)
    );
    assert_eq!(state.request_selected_pad_purge(), 1);
    assert!(!state.output_buffers.contains_key(&(blocked.x, blocked.y)));
    assert_eq!(
        state.output_buffers.get(&(partial.x, partial.y)),
        Some(&4.0)
    );
    assert_eq!(
        state.output_buffers.get(&(unselected.x, unselected.y)),
        Some(&capacity)
    );
    assert_eq!(
        state.input_buffers.get(&(blocked.x, blocked.y)),
        Some(&12.0)
    );
}
