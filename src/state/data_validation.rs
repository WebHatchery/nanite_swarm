//! Checks that data declarations can be understood by the durable game state.

use crate::data::GameData;

use super::{Milestone, TutorialGoal};

#[cfg(test)]
mod tests;

pub(crate) fn validate_game_data(data: &GameData) -> Result<(), String> {
    if data.planets.len() != super::campaign::PLANET_COUNT {
        return Err(format!(
            "assets/planets.json has {} planets; expected {}",
            data.planets.len(),
            super::campaign::PLANET_COUNT
        ));
    }
    for def in &data.achievements {
        if Milestone::from_id(&def.condition.kind).is_none() {
            return Err(format!(
                "assets/achievements.json achievement \"{}\" has unknown condition \"{}\"",
                def.id, def.condition.kind
            ));
        }
        if !def.condition.target.is_finite() || def.condition.target < 0.0 {
            return Err(format!(
                "assets/achievements.json achievement \"{}\" has invalid target",
                def.id
            ));
        }
    }
    for stage in &data.core_stages {
        for requirement in &stage.requires {
            if Milestone::from_id(&requirement.kind).is_none() {
                return Err(format!(
                    "assets/core_stages.json stage \"{}\" has unknown requirement \"{}\"",
                    stage.id, requirement.kind
                ));
            }
        }
    }
    for step in &data.tutorial {
        if TutorialGoal::parse(&step.goal).is_none() {
            return Err(format!(
                "assets/tutorial.json step \"{}\" has unknown goal kind \"{}\"",
                step.id, step.goal.kind
            ));
        }
    }
    Ok(())
}
