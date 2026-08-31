//! The Seed Ship: what a world is ultimately converted into.
//!
//! The GDD's loop ends with "build the Mega-Structure, launch, and begin the
//! cycle on a new world". The ship is built in stages declared in
//! `assets/seed_ship.json`; while the swarm is committed to it, the yard
//! swallows resources out of the pool at a capped rate until a stage is paid
//! for. That cap is the point — the ship cannot be bought with one full
//! storehouse, only with sustained production.

use serde::{Deserialize, Serialize};

use crate::data::{GameDataContext, SeedShipCost, SeedShipStageDef};
use crate::engine::ResourceType;

use super::game_state::Resources;

mod planet;

/// What has been poured into the stage under construction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct StageProgress {
    pub minerals: f32,
    pub data: f32,
    pub biomass: f32,
    #[serde(default)]
    pub alloy: f32,
    #[serde(default)]
    pub components: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeedShip {
    #[serde(skip, default)]
    data: GameDataContext,
    /// Index of the stage being built; equal to the stage count once finished.
    stage: usize,
    progress: StageProgress,
    /// Whether the swarm is currently diverting production into the yard.
    pub committed: bool,
    /// How many ships this world has already sent. Leaving again means
    /// building another one from the keel up.
    #[serde(default)]
    launches: u32,
}

impl SeedShip {
    pub(crate) fn with_data(data: GameDataContext) -> Self {
        Self {
            data,
            ..Self::default()
        }
    }

    pub(crate) fn attach_data(&mut self, data: GameDataContext) {
        self.data = data;
    }

    fn stages(&self) -> &[SeedShipStageDef] {
        &self.data.data().seed_ship.stages
    }

    pub fn stage_index(&self) -> usize {
        self.stage
    }

    pub fn stage_count(&self) -> usize {
        self.stages().len()
    }

    /// The stage under construction, or `None` once the ship is finished.
    pub fn stage(&self) -> Option<&SeedShipStageDef> {
        self.stages().get(self.stage)
    }

    /// The stages already built and still on the pad.
    ///
    /// They stop counting the moment the ship launches, which is the point:
    /// the yard's advantages leave with it, and the next ship has to earn them
    /// again.
    pub fn standing_stages(&self) -> &[SeedShipStageDef] {
        let built = self.stage.min(self.stages().len());
        &self.stages()[..built]
    }

    /// Every stage is paid for and the ship is ready to leave.
    pub fn is_complete(&self) -> bool {
        let count = self.stage_count();
        count > 0 && self.stage >= count
    }

    pub fn progress(&self) -> StageProgress {
        self.progress
    }

    /// Built, and still sitting on the pad.
    pub fn is_ready_to_launch(&self) -> bool {
        self.is_complete()
    }

    /// How many ships have left this world.
    pub fn launches(&self) -> u32 {
        self.launches
    }

    /// Reject impossible persisted yard state before it can feed a malformed
    /// progress value into the next intake tick.
    pub(crate) fn validate_loaded_state(&self) -> Result<(), String> {
        let stages = self.stages();
        if self.stage > stages.len() {
            return Err("has a Seed Ship stage past the end of its plan".to_string());
        }
        if self.stage == stages.len() && self.committed {
            return Err("has a committed completed Seed Ship".to_string());
        }

        let values = [
            ("minerals", self.progress.minerals),
            ("data", self.progress.data),
            ("biomass", self.progress.biomass),
            ("alloy", self.progress.alloy),
            ("components", self.progress.components),
        ];
        if values
            .into_iter()
            .any(|(_, value)| !value.is_finite() || value < 0.0)
        {
            return Err("has invalid Seed Ship progress".to_string());
        }
        if let Some(stage) = stages.get(self.stage) {
            let costs = [
                ("minerals", self.progress.minerals, stage.cost.minerals),
                ("data", self.progress.data, stage.cost.data),
                ("biomass", self.progress.biomass, stage.cost.biomass),
                ("alloy", self.progress.alloy, stage.cost.alloy),
                (
                    "components",
                    self.progress.components,
                    stage.cost.components,
                ),
            ];
            if costs.into_iter().any(|(_, progress, cost)| progress > cost) {
                return Err("has Seed Ship progress beyond the current stage cost".to_string());
            }
        } else if values.into_iter().any(|(_, value)| value != 0.0) {
            return Err("has progress on a completed Seed Ship".to_string());
        }
        Ok(())
    }

    /// Spend the ship. The yard is left empty: leaving again means building
    /// another from the keel up, which is also why this cannot double-launch.
    pub(super) fn mark_launched(&mut self) {
        self.launches += 1;
        self.stage = 0;
        self.progress = StageProgress::default();
        self.committed = false;
    }

    /// How far the current stage is from paid, 0..1 across all its resources.
    pub fn stage_fraction(&self) -> f32 {
        let Some(stage) = self.stage() else {
            return 1.0;
        };
        let cost = stage.cost;
        let required = cost.minerals + cost.data + cost.biomass + cost.alloy + cost.components;
        if required <= 0.0 {
            return 1.0;
        }
        let paid = self.progress.minerals
            + self.progress.data
            + self.progress.biomass
            + self.progress.alloy
            + self.progress.components;
        (paid / required).clamp(0.0, 1.0)
    }

    /// Whether the yard has broken ground: the swarm has committed to a ship,
    /// or at least one stage of one already stands.
    pub fn has_broken_ground(&self) -> bool {
        self.committed || self.stage > 0
    }

    /// How much of the whole ship is paid for, 0..1 across every stage. The
    /// skyline is drawn from this, so a stage part-paid has to count for part
    /// of it rather than for nothing.
    pub fn built_fraction(&self) -> f32 {
        if self.is_complete() {
            return 1.0;
        }
        let count = self.stage_count();
        if count == 0 {
            return 0.0;
        }
        (self.stage as f32 + self.stage_fraction()) / count as f32
    }

    /// The research the current stage is waiting on, if it is waiting.
    pub fn blocked_by(&self, unlocked: &[String]) -> Option<&str> {
        let stage = self.stage()?;
        let required = stage.requires.as_deref()?;
        if unlocked.iter().any(|id| id == required) {
            return None;
        }
        Some(required)
    }

    /// Take up to `delta_time` seconds of intake out of `resources`, and
    /// advance a stage when it is fully paid. Returns true if a stage was
    /// finished by this call.
    pub fn absorb(
        &mut self,
        resources: &mut Resources,
        intake: SeedShipCost,
        delta_time: f32,
    ) -> bool {
        let Some(stage) = self.stage() else {
            return false;
        };
        let cost = stage.cost;

        let take = |resource: ResourceType,
                    pool: &mut f32,
                    paid: &mut f32,
                    needed: f32,
                    rate: f32,
                    totals: &mut super::consumption::ConsumptionTotals| {
            let remaining = (needed - *paid).max(0.0);
            let moved = remaining.min(rate * delta_time).min(*pool).max(0.0);
            *pool -= moved;
            *paid += moved;
            totals.record(resource, moved);
        };

        take(
            ResourceType::Minerals,
            &mut resources.minerals,
            &mut self.progress.minerals,
            cost.minerals,
            intake.minerals,
            &mut resources.consumed,
        );
        take(
            ResourceType::Data,
            &mut resources.data,
            &mut self.progress.data,
            cost.data,
            intake.data,
            &mut resources.consumed,
        );
        take(
            ResourceType::Biomass,
            &mut resources.biomass,
            &mut self.progress.biomass,
            cost.biomass,
            intake.biomass,
            &mut resources.consumed,
        );
        take(
            ResourceType::Alloy,
            &mut resources.alloy,
            &mut self.progress.alloy,
            cost.alloy,
            intake.alloy,
            &mut resources.consumed,
        );
        take(
            ResourceType::Components,
            &mut resources.components,
            &mut self.progress.components,
            cost.components,
            intake.components,
            &mut resources.consumed,
        );

        let paid = self.progress.minerals >= cost.minerals
            && self.progress.data >= cost.data
            && self.progress.biomass >= cost.biomass
            && self.progress.alloy >= cost.alloy
            && self.progress.components >= cost.components;
        if paid {
            self.stage += 1;
            self.progress = StageProgress::default();
            if self.is_complete() {
                self.committed = false;
            }
        }
        paid
    }
}

#[cfg(test)]
mod tests;
