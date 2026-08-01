//! The opening lesson, driven from `assets/tutorial.json`.
//!
//! It was five hardcoded match arms and a "Tutorial step 4 / 5" counter, which
//! told the player how far through they were but never what to do. The steps
//! are data now, each with a goal the simulation can check, and the panel says
//! the next thing rather than the current number.

use crate::data::{TutorialGoalDef, TutorialStepDef};
use crate::engine::BuildingType;

use super::game_state::PlanetState;

/// What finishes a step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TutorialGoal {
    /// One of these exists anywhere on the planet.
    Build(BuildingType),
    /// This research has been completed.
    Research(String),
    /// One of these is joined to the Core's network.
    Connect(BuildingType),
}

impl TutorialGoal {
    /// Read a declared goal, or `None` if it names something unknown.
    ///
    /// This runs while the data files are still being loaded, so it resolves
    /// nothing against `game_data()`: asking for the game data from inside its
    /// own initialiser deadlocks.
    pub fn parse(def: &TutorialGoalDef) -> Option<Self> {
        match def.kind.as_str() {
            "build" => BuildingType::from_id(&def.target).map(TutorialGoal::Build),
            "connect" => BuildingType::from_id(&def.target).map(TutorialGoal::Connect),
            "research" => Some(TutorialGoal::Research(def.target.clone())),
            _ => None,
        }
    }

    /// The building this step wants the player to place, if any. The build
    /// palette pulses it, which is the difference between an instruction and a
    /// hint you can follow without reading.
    pub fn highlighted_building(&self) -> Option<BuildingType> {
        match self {
            TutorialGoal::Build(building) => Some(*building),
            TutorialGoal::Connect(_) => Some(BuildingType::Conduit),
            TutorialGoal::Research(_) => None,
        }
    }
}

impl PlanetState {
    fn steps() -> &'static [TutorialStepDef] {
        &crate::data::game_data().tutorial
    }

    pub fn tutorial_step_count(&self) -> usize {
        Self::steps().len()
    }

    /// The step the player is on, or `None` once they are through it.
    pub fn tutorial_current(&self) -> Option<&'static TutorialStepDef> {
        if self.tutorial_done {
            return None;
        }
        Self::steps().get(self.tutorial_step as usize)
    }

    /// The building the current step wants placed, for the palette to point at.
    pub fn tutorial_highlight(&self) -> Option<BuildingType> {
        let step = self.tutorial_current()?;
        TutorialGoal::parse(&step.goal)?.highlighted_building()
    }

    /// Has the current step's goal been met?
    fn tutorial_goal_met(&self, goal: &TutorialGoal) -> bool {
        match goal {
            TutorialGoal::Build(building) => !self.grid.find_buildings(*building).is_empty(),
            TutorialGoal::Research(tech) => {
                self.research.unlocked_techs.iter().any(|id| id == tech)
            }
            TutorialGoal::Connect(building) => self.grid.iter_tiles().any(|(_, tile)| {
                tile.building.as_ref().is_some_and(|placed| {
                    placed.building_type == *building && placed.connected_to_core
                })
            }),
        }
    }

    pub(super) fn update_tutorial(&mut self) {
        if self.tutorial_done {
            return;
        }
        let Some(step) = self.tutorial_current() else {
            // No steps defined at all: nothing to teach.
            self.tutorial_done = true;
            return;
        };
        let Some(goal) = TutorialGoal::parse(&step.goal) else {
            return;
        };
        if !self.tutorial_goal_met(&goal) {
            return;
        }

        let title = step.title.clone();
        self.tutorial_step = self.tutorial_step.saturating_add(1);
        if self.tutorial_step as usize >= Self::steps().len() {
            self.tutorial_done = true;
            self.notifications
                .success("The swarm needs no more guidance.");
        } else {
            self.notifications.success(format!("Done: {}", title));
        }
    }
}

#[cfg(test)]
mod tests {
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
}
