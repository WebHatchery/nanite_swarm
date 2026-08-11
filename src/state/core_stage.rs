//! The Evolving Core (GDD §5).
//!
//! The stage used to be a number the *renderer* worked out from time played
//! plus whatever was in the stockpile, so spending minerals visibly walked the
//! Core backwards and nothing about it meant anything. It is now earned state:
//! declared milestones in `assets/core_stages.json`, reached in order, never
//! given back, and each one does something for the world it stands on.

use crate::data::CoreStageDef;

use super::game_state::PlanetState;
use super::milestone::Milestone;

/// Every declared stage, starting with the one a world lands with.
pub fn core_stages() -> &'static [CoreStageDef] {
    &crate::data::game_data().core_stages
}

impl PlanetState {
    /// The stage the Core has reached, clamped to what is declared.
    pub fn core_stage_index(&self) -> usize {
        (self.core_stage as usize).min(core_stages().len().saturating_sub(1))
    }

    pub fn core_stage_def(&self) -> Option<&'static CoreStageDef> {
        core_stages().get(self.core_stage_index())
    }

    /// The stages standing, which is every one reached so far. Their effects
    /// are cumulative: the Core does not trade one for the next.
    pub fn core_stages_reached(&self) -> &'static [CoreStageDef] {
        let reached = (self.core_stage as usize + 1).min(core_stages().len());
        &core_stages()[..reached]
    }

    /// Advance the Core through any stage whose milestones are all met.
    ///
    /// Only forward. A stage reached by a base that is later torn apart stays
    /// reached — the Core grew, and growth is not something the swarm undoes.
    pub(super) fn update_core_stage(&mut self) {
        let mut advanced = None;
        while let Some(next) = core_stages().get(self.core_stage as usize + 1) {
            let met = next.requires.iter().all(|requirement| {
                Milestone::from_id(&requirement.kind)
                    .is_some_and(|milestone| self.meets(milestone, requirement.target))
            });
            if !met {
                break;
            }
            self.core_stage += 1;
            advanced = Some(next);
        }

        let Some(stage) = advanced else {
            return;
        };
        // A new stage changes what the world can do, so the sheet has to be
        // rebuilt before anything reads it again.
        self.refresh_stats();
        self.notifications
            .success(format!("Core: {}. {}", stage.name, stage.description));
    }
}

#[cfg(test)]
mod tests;
