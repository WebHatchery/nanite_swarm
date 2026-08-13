//! What the swarm gets told it has done.
//!
//! The set lives in `assets/achievements.json`; each entry names a
//! [`Milestone`] the simulation can measure, or `manual` for the ones code
//! announces at the moment they happen.

use crate::data::AchievementDef;

use super::game_state::PlanetState;
use super::milestone::Milestone;

impl PlanetState {
    /// Fire whatever the world has just become true for.
    ///
    /// Everything is measured against the world in front of the player, so an
    /// achievement earned on one planet is earned on that planet — which is
    /// also why the two campaign-wide ones are announced by code instead.
    pub(super) fn update_achievements(&mut self) {
        let earned: Vec<String> = crate::data::game_data()
            .achievements
            .iter()
            .filter(|def| self.achievement_met(def))
            .map(|def| def.id.clone())
            .collect();
        for id in earned {
            self.announce_achievement(&id);
        }
    }

    fn achievement_met(&self, def: &AchievementDef) -> bool {
        Milestone::from_id(&def.condition.kind)
            .is_some_and(|milestone| self.meets(milestone, def.condition.target))
    }

    /// The whole set as a screen can show it: earned or not, and how far along
    /// the ones that are not.
    ///
    /// In declaration order, so the list does not reshuffle itself as things
    /// are earned.
    pub fn achievement_records(&self) -> Vec<AchievementRecord> {
        crate::data::game_data()
            .achievements
            .iter()
            .map(|def| {
                let milestone = Milestone::from_id(&def.condition.kind);
                let target = def.condition.target;
                AchievementRecord {
                    name: def.name.as_str(),
                    description: def.description.as_str(),
                    unlocked: self.achievements.is_unlocked(&def.id),
                    // A manual achievement is announced by code at the moment
                    // it happens, so there is no running total to show.
                    progress: milestone.map(|milestone| self.measure(milestone)),
                    target,
                    countable: milestone.is_some_and(|milestone| milestone != Milestone::Manual)
                        && target > 1.0,
                }
            })
            .collect()
    }
}

/// One achievement as the records screen reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AchievementRecord {
    pub name: &'static str,
    pub description: &'static str,
    pub unlocked: bool,
    /// Where the world stands, for the ones measured off state.
    pub progress: Option<f32>,
    pub target: f32,
    /// Whether "7 / 10" says anything useful. A one-shot condition is either
    /// done or not, and a bar for it is noise.
    pub countable: bool,
}

impl AchievementRecord {
    /// How far along, from zero to one. Anything earned reads as full however
    /// the world has moved on since.
    pub fn fraction(&self) -> f32 {
        if self.unlocked {
            return 1.0;
        }
        match self.progress {
            Some(progress) if self.target > 0.0 => (progress / self.target).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests;
