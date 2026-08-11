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
mod tests;
