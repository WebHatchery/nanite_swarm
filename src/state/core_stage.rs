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

/// Live progress toward one requirement of the next Core stage.
///
/// This is a snapshot rather than a reference so remote world telemetry can
/// carry the same actionable blocker that the local inspector reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoreRequirementProgress {
    pub milestone: Milestone,
    pub current: f32,
    pub target: f32,
}

impl PlanetState {
    /// Every declared stage, starting with the one a world lands with.
    pub fn core_stages(&self) -> &[CoreStageDef] {
        &self.data().core_stages
    }

    /// The stage the Core has reached, clamped to what is declared.
    pub fn core_stage_index(&self) -> usize {
        (self.core_stage as usize).min(self.core_stages().len().saturating_sub(1))
    }

    pub fn core_stage_def(&self) -> Option<&CoreStageDef> {
        self.core_stages().get(self.core_stage_index())
    }

    /// The stages standing, which is every one reached so far. Their effects
    /// are cumulative: the Core does not trade one for the next.
    pub fn core_stages_reached(&self) -> &[CoreStageDef] {
        let reached = (self.core_stage as usize + 1).min(self.core_stages().len());
        &self.core_stages()[..reached]
    }

    /// The unmet requirements for the next stage, in authored order.
    ///
    /// A stage is normally advanced in the same simulation tick that its last
    /// requirement is met. Keeping only unmet requirements here makes the
    /// readout an instruction rather than a second checklist of completed
    /// work.
    pub fn core_requirement_progress(&self) -> Vec<CoreRequirementProgress> {
        let Some(next) = self.core_stages().get(self.core_stage_index() + 1) else {
            return Vec::new();
        };

        next.requires
            .iter()
            .filter_map(|requirement| {
                let milestone = Milestone::from_id(&requirement.kind)?;
                let current = self.measure(milestone);
                (current + 0.001 < requirement.target).then_some(CoreRequirementProgress {
                    milestone,
                    current,
                    target: requirement.target,
                })
            })
            .collect()
    }

    /// The first unmet requirement, useful when a compact remote summary has
    /// room for one clear recovery target rather than the whole checklist.
    pub fn next_core_requirement(&self) -> Option<CoreRequirementProgress> {
        self.core_requirement_progress().into_iter().next()
    }

    /// Advance the Core through any stage whose milestones are all met.
    ///
    /// Only forward. A stage reached by a base that is later torn apart stays
    /// reached — the Core grew, and growth is not something the swarm undoes.
    pub(super) fn update_core_stage(&mut self) {
        let mut advanced = None;
        while let Some(next) = self
            .core_stages()
            .get(self.core_stage as usize + 1)
            .cloned()
        {
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
