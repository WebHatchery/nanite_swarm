use super::*;
use crate::data::GameConfig;
use crate::engine::{BuildingType, GridPos, StatId, TerrainType};

fn state() -> PlanetState {
    PlanetState::new(2, 42, GameConfig::default())
}

/// Stand `count` conduits beside the Core, which is both structures and
/// network as far as the milestones are concerned.
fn build(state: &mut PlanetState, count: i32) {
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 32);
    for step in 1..=count {
        let pos = GridPos::new(core.x + step % 10, core.y + 1 + step / 10);
        if let Some(tile) = state.grid.get_mut(pos) {
            tile.terrain = TerrainType::Empty;
        }
        state.grid.place_building(pos, BuildingType::Conduit);
    }
}

#[test]
fn a_world_lands_on_the_first_stage_and_no_further() {
    let mut state = state();
    state.update_core_stage();
    assert_eq!(state.core_stage, 0);
    assert_eq!(state.core_stage_def().unwrap().id, "crash_lander");
}

#[test]
fn building_enough_of_a_base_grows_the_core() {
    let mut state = state();
    build(&mut state, 12);
    state.update_core_stage();

    assert_eq!(state.core_stage, 1);
    assert_eq!(state.core_stage_def().unwrap().id, "foundry");
    assert!(
        !state.notifications.is_empty(),
        "the Core grew without saying so"
    );
}

#[test]
fn a_stage_the_core_reached_is_not_given_back_when_the_base_is_torn_down() {
    let mut state = state();
    build(&mut state, 12);
    state.update_core_stage();
    assert_eq!(state.core_stage, 1);

    let standing: Vec<GridPos> = state
        .grid
        .iter_tiles()
        .filter(|(_, tile)| {
            tile.building
                .as_ref()
                .is_some_and(|b| b.building_type == BuildingType::Conduit)
        })
        .map(|(pos, _)| pos)
        .collect();
    for pos in standing {
        state.grid.remove_building(pos);
    }
    state.update_core_stage();

    assert_eq!(state.core_stage, 1, "the Core walked backwards");
}

#[test]
fn a_stage_that_is_standing_works_for_the_world_it_stands_on() {
    let mut state = state();
    let before = state.mineral_capacity();
    build(&mut state, 12);
    state.update_core_stage();

    assert!(
        state.mineral_capacity() > before,
        "the Foundry did nothing: {} then {}",
        before,
        state.mineral_capacity()
    );
}

#[test]
fn the_core_can_run_through_several_stages_in_one_go() {
    let mut state = state();
    // Everything the second and third stages ask for at once.
    build(&mut state, 30);
    for tech in ["a", "b", "c", "d", "e", "f"] {
        state.research.unlocked_techs.push(tech.to_string());
    }
    state.update_core_stage();

    assert_eq!(state.core_stage, 2);
    assert_eq!(state.core_stages_reached().len(), 3);
}

#[test]
fn every_declared_stage_asks_for_something_the_game_can_measure() {
    for stage in core_stages() {
        assert!(!stage.name.is_empty());
        for requirement in &stage.requires {
            assert!(
                Milestone::from_id(&requirement.kind).is_some(),
                "{} asks for \"{}\"",
                stage.id,
                requirement.kind
            );
        }
        for modifier in &stage.modifiers {
            assert!(crate::engine::parse_modifier(modifier).is_ok());
        }
    }
    // And the art has a frame for each of them.
    assert_eq!(core_stages().len(), 5);
}

#[test]
fn the_last_stage_leaves_the_core_where_it_is() {
    let mut state = state();
    state.core_stage = (core_stages().len() - 1) as u8;
    // The same rebuild a loaded save gets, since nothing advanced here.
    state.refresh_stats();
    state.update_core_stage();

    assert_eq!(state.core_stage as usize, core_stages().len() - 1);
    assert!(
        state.stats.multiplier(StatId::DrillOutput) > 1.0,
        "the last stage stopped working once it was the last one"
    );
}
