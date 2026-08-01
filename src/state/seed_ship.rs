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
        let required = cost.minerals + cost.data + cost.biomass;
        if required <= 0.0 {
            return 1.0;
        }
        let paid = self.progress.minerals + self.progress.data + self.progress.biomass;
        (paid / required).clamp(0.0, 1.0)
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

        let paid = self.progress.minerals >= cost.minerals
            && self.progress.data >= cost.data
            && self.progress.biomass >= cost.biomass;
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
        let intake = crate::data::game_data().seed_ship.intake_per_second;
        if self
            .seed_ship
            .absorb(&mut self.resources, intake, delta_time)
            && self.seed_ship.is_complete()
        {
            self.achievements.unlock("seed_ship");
        }
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
mod tests {
    use super::*;
    use crate::data::GameConfig;

    fn state() -> PlanetState {
        let mut state = PlanetState::new(2, 42, GameConfig::default());
        state.config.resources.base_mineral_cap = 100_000.0;
        state.resources.minerals = 10_000.0;
        state.resources.data = 10_000.0;
        state.resources.biomass = 10_000.0;
        state
    }

    fn intake() -> SeedShipCost {
        crate::data::game_data().seed_ship.intake_per_second
    }

    #[test]
    fn a_new_ship_starts_on_the_first_stage_and_is_not_complete() {
        let ship = SeedShip::default();
        assert_eq!(ship.stage_index(), 0);
        assert!(ship.stage_count() >= 4);
        assert!(!ship.is_complete());
        assert_eq!(ship.stage_fraction(), 0.0);
        assert!(!ship.committed);
    }

    #[test]
    fn nothing_is_taken_until_the_swarm_commits() {
        let mut state = state();
        let before = state.resources.minerals;
        state.update_seed_ship(10.0);
        assert_eq!(state.resources.minerals, before);
        assert_eq!(state.seed_ship.stage_fraction(), 0.0);
    }

    #[test]
    fn intake_is_capped_per_second_rather_than_taken_all_at_once() {
        let mut ship = SeedShip::default();
        let mut resources = Resources {
            minerals: 10_000.0,
            ..Default::default()
        };

        ship.absorb(&mut resources, intake(), 1.0);

        assert_eq!(resources.minerals, 10_000.0 - intake().minerals);
        assert_eq!(ship.progress().minerals, intake().minerals);
        assert_eq!(ship.stage_index(), 0);
    }

    #[test]
    fn a_stage_completes_only_once_every_resource_is_paid() {
        let mut ship = SeedShip::default();
        let cost = ship.stage().unwrap().cost;
        let mut resources = Resources {
            minerals: cost.minerals,
            data: 0.0,
            biomass: 0.0,
            energy: 0.0,
        };

        // One very long step: intake is capped by what the stage still needs.
        let finished = ship.absorb(&mut resources, intake(), 10_000.0);

        // Stage one asks only for minerals, so this pays it off exactly.
        assert!(finished);
        assert_eq!(ship.stage_index(), 1);
        assert_eq!(ship.progress(), StageProgress::default());
        assert!(resources.minerals < 1.0);
    }

    #[test]
    fn a_stage_that_needs_data_waits_for_it() {
        let mut ship = SeedShip::default();
        let mut resources = Resources {
            minerals: 100_000.0,
            ..Default::default()
        };
        // Clear stage one, which is minerals only.
        assert!(ship.absorb(&mut resources, intake(), 10_000.0));
        assert_eq!(ship.stage_index(), 1);
        assert!(ship.stage().unwrap().cost.data > 0.0);

        // Minerals alone cannot finish stage two.
        assert!(!ship.absorb(&mut resources, intake(), 10_000.0));
        assert_eq!(ship.stage_index(), 1);
        assert!(ship.stage_fraction() > 0.0 && ship.stage_fraction() < 1.0);

        resources.data = 10_000.0;
        assert!(ship.absorb(&mut resources, intake(), 10_000.0));
        assert_eq!(ship.stage_index(), 2);
    }

    #[test]
    fn a_committed_swarm_eventually_finishes_the_whole_ship() {
        let mut state = state();
        state.toggle_seed_ship_commitment();
        assert!(state.seed_ship.committed);

        for _ in 0..2_000 {
            state.update_seed_ship(1.0);
        }

        assert!(state.seed_ship.is_complete());
        assert!(state.seed_ship.stage().is_none());
        assert_eq!(state.seed_ship.stage_fraction(), 1.0);
        // A finished ship stops drawing on the pool.
        assert!(!state.seed_ship.committed);
    }

    #[test]
    fn finishing_the_ship_unlocks_the_achievement() {
        let mut state = state();
        state.toggle_seed_ship_commitment();
        for _ in 0..2_000 {
            state.update_seed_ship(1.0);
        }
        assert!(state.achievements.is_unlocked("seed_ship"));
    }

    #[test]
    fn a_finished_ship_cannot_be_re_committed() {
        let mut state = state();
        state.toggle_seed_ship_commitment();
        for _ in 0..2_000 {
            state.update_seed_ship(1.0);
        }
        state.toggle_seed_ship_commitment();
        assert!(!state.seed_ship.committed);
    }
}
