use super::*;
use crate::data::GameConfig;

mod action_feedback;
mod blueprints;
mod construction;
mod processor_controls;
mod relocation;
mod terrain;

fn state() -> PlanetState {
    PlanetState::new(2, 42, GameConfig::default())
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
