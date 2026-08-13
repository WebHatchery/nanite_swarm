use super::*;
use crate::engine::{GridPos, TerrainType};

fn state() -> PlanetState {
    PlanetState::new(2, 42, crate::data::GameConfig::default())
}

#[test]
fn the_shipped_tutorial_declares_only_goals_the_game_can_check() {
    let steps = crate::data::game_data().tutorial.clone();
    assert!(!steps.is_empty(), "no tutorial at all");
    for step in &steps {
        assert!(
            TutorialGoal::parse(&step.goal).is_some(),
            "step {} has an unreadable goal",
            step.id
        );
        assert!(
            !step.instruction.is_empty(),
            "step {} says nothing",
            step.id
        );
    }
}

#[test]
fn every_tutorial_step_names_the_visible_touch_action() {
    for step in &crate::data::game_data().tutorial {
        assert!(
            step.instruction.contains("Tap ") || step.instruction.contains("Drag "),
            "step {} does not explain its touch action: {}",
            step.id,
            step.instruction
        );
    }
}

#[test]
fn a_new_planet_starts_on_the_first_step() {
    let state = state();
    assert_eq!(state.tutorial_step, 0);
    assert!(!state.tutorial_done);
    let step = state.tutorial_current().expect("a first step");
    assert_eq!(step.id, "first_drill");
    assert_eq!(state.tutorial_highlight(), Some(BuildingType::Drill));
}

#[test]
fn placing_the_drill_finishes_the_first_step_and_says_so() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(pos));

    state.update_tutorial();

    assert_eq!(state.tutorial_step, 1);
    assert_eq!(state.tutorial_current().unwrap().id, "power_grid");
    assert!(!state.notifications.is_empty(), "the step passed silently");
}

#[test]
fn a_step_the_player_has_not_done_does_not_advance() {
    let mut state = state();
    for _ in 0..10 {
        state.update_tutorial();
    }
    assert_eq!(state.tutorial_step, 0);
}

#[test]
fn a_research_step_waits_for_the_research() {
    let mut state = state();
    state.tutorial_step = 1;
    assert_eq!(state.tutorial_current().unwrap().id, "power_grid");
    assert_eq!(state.tutorial_highlight(), None, "nothing to point at");

    state.update_tutorial();
    assert_eq!(state.tutorial_step, 1);

    state.research.unlocked_techs.push("power_grid".to_string());
    state.update_tutorial();
    assert_eq!(state.tutorial_step, 2);
}

#[test]
fn finishing_the_last_step_ends_the_tutorial() {
    let mut state = state();
    state.tutorial_step = (state.tutorial_step_count() - 1) as u8;
    let last = state.tutorial_current().unwrap();
    let goal = TutorialGoal::parse(&last.goal).unwrap();
    let TutorialGoal::Build(building) = goal else {
        panic!("the last step is expected to be a build goal");
    };

    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(building);
    state.select_building(building);
    assert!(state.try_place_building(pos));

    state.update_tutorial();
    assert!(state.tutorial_done);
    assert!(state.tutorial_current().is_none());
    assert_eq!(state.tutorial_highlight(), None);
}
