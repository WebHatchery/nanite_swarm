use super::*;
use crate::data::GameConfig;
use crate::engine::{BuildingType, GridPos, TerrainType};

fn state() -> PlanetState {
    PlanetState::new(2, 42, GameConfig::default())
}

fn place_unlocked(state: &mut PlanetState, kind: BuildingType, offset: i32) {
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + offset, core.y);
    state.grid.reveal_around(pos, 1);
    if let Some(tile) = state.grid.get_mut(pos) {
        tile.terrain = TerrainType::Empty;
    }
    state.unlock_building(kind);
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    state.select_building(kind);
    assert!(state.try_place_building(pos));
}

#[test]
fn focus_cycles_through_the_four_control_room_profiles() {
    let mut state = state();
    assert_eq!(state.factory_focus, FactoryFocus::Balanced);
    state.cycle_factory_focus();
    assert_eq!(state.factory_focus, FactoryFocus::Extraction);
    state.cycle_factory_focus();
    assert_eq!(state.factory_focus, FactoryFocus::Refining);
    state.cycle_factory_focus();
    assert_eq!(state.factory_focus, FactoryFocus::Assembly);
    state.cycle_factory_focus();
    assert_eq!(state.factory_focus, FactoryFocus::Balanced);
}

#[test]
fn focus_only_accelerates_the_deck_it_names() {
    let state = state();
    assert_eq!(state.factory_focus_multiplier(BuildingType::Drill), 1.0);

    let mut focused = state;
    focused.set_factory_focus(FactoryFocus::Extraction);
    assert_eq!(focused.factory_focus_multiplier(BuildingType::Drill), 1.25);
    assert_eq!(focused.factory_focus_multiplier(BuildingType::Smelter), 1.0);
}

#[test]
fn a_focused_powered_deck_pays_the_declared_energy_tax() {
    let mut state = state();
    place_unlocked(&mut state, BuildingType::Drill, 1);
    let balanced_draw = state.power_consumption();
    state.set_factory_focus(FactoryFocus::Extraction);

    assert!(state.factory_focus_power_tax() > 0.0);
    assert!(state.power_consumption() > balanced_draw);
}

#[test]
fn depth_opens_from_foundry_to_assembly_to_orbital() {
    let mut state = state();
    assert_eq!(state.factory_depth(), 0);

    place_unlocked(&mut state, BuildingType::Smelter, 2);
    assert_eq!(state.factory_depth(), 1);
    place_unlocked(&mut state, BuildingType::Assembler, 3);
    assert_eq!(state.factory_depth(), 2);
    place_unlocked(&mut state, BuildingType::MassDriver, 4);
    assert_eq!(state.factory_depth(), 3);
    assert_eq!(state.factory_depth_progress(), (1.0, "ALL DECKS ONLINE"));
}
