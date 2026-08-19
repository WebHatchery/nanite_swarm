use super::*;
use crate::data::GameConfig;
use crate::engine::{Building, BuildingType};

#[test]
fn assembler_art_tracks_each_hopper_and_its_waiting_components() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let pos = state.grid.find_core().unwrap();
    state.grid.get_mut(pos).unwrap().building = Some(Building::new(BuildingType::Assembler, pos));
    state
        .grid
        .get_mut(pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .powered = true;
    let building = state.grid.get_mut(pos).unwrap().building.as_mut().unwrap();
    building.input_priority = true;
    building.overclocked = true;
    state.input_hoppers.insert(
        (pos.x, pos.y),
        [(ResourceType::Minerals, 5.0), (ResourceType::Alloy, 5.0)]
            .into_iter()
            .collect(),
    );
    state.output_buffers.insert((pos.x, pos.y), 5.0);
    let recipe = &crate::data::game_data().building("assembler").recipe;

    let visual = processor_visual(&state, pos, recipe).unwrap();
    assert_eq!(visual.inputs[0], (ResourceType::Minerals, 0.5));
    assert_eq!(visual.inputs[1], (ResourceType::Alloy, 1.0));
    assert_eq!(
        visual.output,
        (
            ResourceType::Components,
            5.0 / state.processor_pad_capacity()
        )
    );
    assert!(visual.active);
    assert!(visual.priority);
    assert!(visual.overclocked);
    assert!(!visual.blocked);
}

#[test]
fn full_output_pad_closes_the_processor_art_shutters() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let pos = state.grid.find_core().unwrap();
    let mut building = Building::new(BuildingType::Smelter, pos);
    building.powered = true;
    state.grid.get_mut(pos).unwrap().building = Some(building);
    state.input_hoppers.insert(
        (pos.x, pos.y),
        [(ResourceType::Minerals, 20.0)].into_iter().collect(),
    );
    state
        .output_buffers
        .insert((pos.x, pos.y), state.processor_pad_capacity());
    let recipe = &crate::data::game_data().building("smelter").recipe;

    let visual = processor_visual(&state, pos, recipe).unwrap();
    assert!(visual.blocked);
    assert!(!visual.active);
}

#[test]
fn empty_hopper_marks_processor_art_idle() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let pos = state.grid.find_core().unwrap();
    state.grid.get_mut(pos).unwrap().building = Some(Building::new(BuildingType::Smelter, pos));
    state
        .grid
        .get_mut(pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .powered = true;
    let recipe = &crate::data::game_data().building("smelter").recipe;

    assert!(!processor_visual(&state, pos, recipe).unwrap().active);
}
