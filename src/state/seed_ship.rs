//! The Seed Ship: what a world is ultimately converted into.
//!
//! The GDD's loop ends with "build the Mega-Structure, launch, and begin the
//! cycle on a new world". The ship is built in stages declared in
//! `assets/seed_ship.json`; while the swarm is committed to it, the yard
//! swallows resources out of the pool at a capped rate until a stage is paid
//! for. That cap is the point — the ship cannot be bought with one full
//! storehouse, only with sustained production.

use serde::{Deserialize, Serialize};

use crate::data::{SeedShipCost, SeedShipStageDef};

use super::game_state::{PlanetState, Resources};

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
    fn stages() -> &'static [SeedShipStageDef] {
        &crate::data::game_data().seed_ship.stages
    }

    pub fn stage_index(&self) -> usize {
        self.stage
    }

    pub fn stage_count(&self) -> usize {
        Self::stages().len()
    }

    /// The stage under construction, or `None` once the ship is finished.
    pub fn stage(&self) -> Option<&'static SeedShipStageDef> {
        Self::stages().get(self.stage)
    }

    /// The stages already built and still on the pad.
    ///
    /// They stop counting the moment the ship launches, which is the point:
    /// the yard's advantages leave with it, and the next ship has to earn them
    /// again.
    pub fn standing_stages(&self) -> &'static [SeedShipStageDef] {
        let built = self.stage.min(Self::stages().len());
        &Self::stages()[..built]
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
    pub fn blocked_by(&self, unlocked: &[String]) -> Option<&'static str> {
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

        let mut take = |pool: &mut f32, paid: &mut f32, needed: f32, rate: f32| {
            let remaining = (needed - *paid).max(0.0);
            let moved = remaining.min(rate * delta_time).min(*pool).max(0.0);
            *pool -= moved;
            *paid += moved;
        };

        take(
            &mut resources.minerals,
            &mut self.progress.minerals,
            cost.minerals,
            intake.minerals,
        );
        take(
            &mut resources.data,
            &mut self.progress.data,
            cost.data,
            intake.data,
        );
        take(
            &mut resources.biomass,
            &mut self.progress.biomass,
            cost.biomass,
            intake.biomass,
        );
        take(
            &mut resources.alloy,
            &mut self.progress.alloy,
            cost.alloy,
            intake.alloy,
        );
        take(
            &mut resources.components,
            &mut self.progress.components,
            cost.components,
            intake.components,
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

impl PlanetState {
    /// Pour this tick's production into the Seed Ship yard, if the swarm has
    /// been told to.
    pub fn update_seed_ship(&mut self, delta_time: f32) {
        if !self.seed_ship.committed || self.seed_ship.is_complete() {
            return;
        }
        // A stage nobody has worked out how to build yet takes nothing: the
        // yard sits idle rather than quietly banking resources against it.
        if self
            .seed_ship
            .blocked_by(&self.research.unlocked_techs)
            .is_some()
        {
            return;
        }

        let intake = crate::data::game_data().seed_ship.intake_per_second;
        let finished_stage = self
            .seed_ship
            .absorb(&mut self.resources, intake, delta_time);
        if !finished_stage {
            return;
        }

        // A finished stage changes what this world can do, so the stat sheet
        // has to be rebuilt before anything reads it again.
        self.refresh_stats();

        if self.seed_ship.is_complete() {
            self.notifications
                .success("Seed Ship complete. This world is spent.");
            self.announce_achievement("seed_ship");
            return;
        }

        // Report the boon of the stage just finished, not the next one's name:
        // what the player gained is the interesting half.
        let built = self.seed_ship.standing_stages();
        match built.last().filter(|stage| !stage.boon.is_empty()) {
            Some(stage) => self
                .notifications
                .success(format!("{} standing. {}", stage.name, stage.boon)),
            None => {
                if let Some(stage) = self.seed_ship.stage() {
                    self.notifications
                        .info(format!("Seed Ship: {} under way", stage.name));
                }
            }
        }
    }

    /// The research the yard is waiting on, and what it is called, for the
    /// ship screen to say so.
    pub fn seed_ship_blocked_by(&self) -> Option<&'static str> {
        let id = self.seed_ship.blocked_by(&self.research.unlocked_techs)?;
        crate::data::game_data()
            .research
            .nodes
            .iter()
            .find(|node| node.id == id)
            .map(|node| node.name.as_str())
    }

    /// Stop or start diverting production into the yard.
    pub fn toggle_seed_ship_commitment(&mut self) {
        if self.seed_ship.is_complete() {
            return;
        }
        self.seed_ship.committed = !self.seed_ship.committed;
    }
}

#[cfg(test)]
mod tests;
