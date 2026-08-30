use super::*;
use crate::engine::{GridPos, TerrainType};

fn state() -> PlanetState {
    PlanetState::new(2, 42, crate::data::GameConfig::default())
}

#[test]
fn the_shipped_tutorial_declares_only_goals_the_game_can_check() {
    let steps = crate::data::test_data().data().tutorial.clone();
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
    for step in &crate::data::test_data().data().tutorial {
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
    assert_eq!(state.tutorial_research_target(), Some("power_grid"));

    state.update_tutorial();
    assert_eq!(state.tutorial_step, 1);

    state.research.unlocked_techs.push("power_grid".to_string());
    state.update_tutorial();
    assert_eq!(state.tutorial_step, 2);
    assert_eq!(state.tutorial_research_target(), None);
}

#[test]
fn the_opening_lesson_reaches_the_first_campaign_launch_rail() {
    let steps = crate::data::test_data().data().tutorial.clone();
    let efficient_drills = steps
        .iter()
        .position(|step| step.id == "efficient_drills_research")
        .expect("the tutorial should teach efficient drills first");
    let power_efficiency = steps
        .iter()
        .position(|step| step.id == "power_efficiency_research")
        .expect("the tutorial should teach power efficiency first");
    let mass_driver_research_index = steps
        .iter()
        .position(|step| step.id == "mass_driver_research")
        .expect("the tutorial should teach the launch-rail research");
    assert!(efficient_drills < mass_driver_research_index);
    assert!(power_efficiency < mass_driver_research_index);

    let efficient_drills_step = &steps[efficient_drills];
    assert_eq!(efficient_drills_step.goal.kind, "research");
    assert_eq!(efficient_drills_step.goal.target, "efficient_drills");
    assert!(efficient_drills_step.instruction.contains("Tap RESEARCH"));
    let power_efficiency_step = &steps[power_efficiency];
    assert_eq!(power_efficiency_step.goal.kind, "research");
    assert_eq!(power_efficiency_step.goal.target, "power_efficiency");
    assert!(power_efficiency_step.instruction.contains("Tap RESEARCH"));

    let research = steps
        .iter()
        .find(|step| step.id == "mass_driver_research")
        .expect("the tutorial should teach the launch-rail research");
    let build = steps
        .iter()
        .find(|step| step.id == "mass_driver")
        .expect("the tutorial should teach the launch-rail build");

    assert_eq!(research.goal.kind, "research");
    assert_eq!(research.goal.target, "mass_driver");
    assert!(research.instruction.contains("Tap RESEARCH"));
    assert_eq!(build.goal.kind, "build");
    assert_eq!(build.goal.target, "mass_driver");
    assert!(build.instruction.contains("Tap MASS DRIVER"));
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
    if building == BuildingType::MassDriver {
        state
            .research
            .unlocked_techs
            .push("mass_driver".to_string());
        state.resources.minerals = 10_000.0;
        state.resources.energy = 10_000.0;
    }
    state.unlock_building(building);
    state.select_building(building);
    assert!(state.try_place_building(pos));

    state.update_tutorial();
    assert!(state.tutorial_done);
    assert!(state.tutorial_current().is_none());
    assert_eq!(state.tutorial_highlight(), None);
    assert_eq!(
        state.tutorial_research_target(),
        None,
        "a completed lesson has no research target"
    );
    assert_eq!(
        state
            .notifications
            .get_notifications()
            .last()
            .unwrap()
            .message,
        "Next: tap SHIP, then MAP."
    );
}
