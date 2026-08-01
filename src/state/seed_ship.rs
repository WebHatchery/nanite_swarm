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
        let required = cost.minerals + cost.data + cost.biomass + cost.alloy;
        if required <= 0.0 {
            return 1.0;
        }
        let paid = self.progress.minerals
            + self.progress.data
            + self.progress.biomass
            + self.progress.alloy;
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

        let paid = self.progress.minerals >= cost.minerals
            && self.progress.data >= cost.data
            && self.progress.biomass >= cost.biomass
            && self.progress.alloy >= cost.alloy;
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
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::engine::{BuildingType, GridPos};

    fn state() -> PlanetState {
        let mut state = PlanetState::new(2, 42, GameConfig::default());
        state.config.resources.base_mineral_cap = 100_000.0;
        state.resources.minerals = 10_000.0;
        state.resources.data = 10_000.0;
        state.resources.biomass = 10_000.0;
        state.resources.alloy = 10_000.0;
        state
    }

    /// The same world, with the research the later stages are gated on.
    fn researched_state() -> PlanetState {
        let mut state = state();
        for stage in &crate::data::game_data().seed_ship.stages {
            if let Some(tech) = stage.requires.as_deref() {
                state.research.unlocked_techs.push(tech.to_string());
            }
        }
        state.refresh_stats();
        state
    }

    fn intake() -> SeedShipCost {
        crate::data::game_data().seed_ship.intake_per_second
    }

    /// A drill beside the Core and a Smelter on its other side, both powered.
    fn state_with_smelter() -> (PlanetState, GridPos, GridPos) {
        let mut state = state();
        state.resources.alloy = 0.0;
        let core = state.grid.find_core().unwrap();
        let drill = GridPos::new(core.x + 1, core.y);
        let smelter = GridPos::new(core.x, core.y + 1);
        for pos in [drill, smelter] {
            state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
            state.grid.reveal_around(pos, 1);
        }
        state.unlock_building(BuildingType::Smelter);
        state.select_building(BuildingType::Smelter);
        assert!(state.try_place_building(smelter));
        state.select_building(BuildingType::Drill);
        assert!(state.try_place_building(drill));
        state.grid.update_power_grid();
        (state, drill, smelter)
    }

    /// The whole chain in one go: ore is carried to a Smelter, refined into
    /// alloy, and the alloy is carried on to the Core by the Smelter's own
    /// drones. Nothing teleports at either end.
    #[test]
    fn ore_is_carried_in_refined_and_the_alloy_carried_out() {
        let (mut state, _drill, smelter) = state_with_smelter();

        // Long enough for a full load of alloy to be made and then delivered.
        for _ in 0..400 {
            state.step(0.1, false);
        }

        assert!(state.resources.alloy > 0.0, "no alloy reached the Core");
        assert!(state.alloy_rate() > 0.0);
        // Ore reached the hopper rather than the global pool...
        assert!(state.input_buffers.contains_key(&(smelter.x, smelter.y)));
        // ...and the Smelter has its own crew to carry the alloy out.
        assert!(!state.drones.drones_at(smelter).is_empty());
    }

    #[test]
    fn alloy_waits_on_the_smelter_pad_until_a_drone_takes_it() {
        let (mut state, _drill, smelter) = state_with_smelter();

        // Short enough that a full load has not been collected yet.
        for _ in 0..100 {
            state.step(0.1, false);
        }

        let on_pad = state
            .output_buffers
            .get(&(smelter.x, smelter.y))
            .copied()
            .unwrap_or(0.0);
        assert!(on_pad > 0.0, "the Smelter refined nothing");
        assert_eq!(
            state.resources.alloy, 0.0,
            "alloy reached the pool without being carried"
        );
    }

    #[test]
    fn a_smelter_nobody_delivers_to_stays_idle_however_full_the_pool_is() {
        let mut state = state();
        state.resources.alloy = 0.0;
        state.resources.minerals = 100_000.0;
        // Powered, but with no drill anywhere to send it anything.
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x, core.y + 1);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.grid.reveal_around(pos, 1);
        state.unlock_building(BuildingType::Smelter);
        state.select_building(BuildingType::Smelter);
        assert!(state.try_place_building(pos));
        state.grid.update_power_grid();

        for _ in 0..100 {
            state.step(0.1, false);
        }
        assert_eq!(state.resources.alloy, 0.0);
    }

    #[test]
    fn ore_goes_to_the_smelter_before_it_goes_to_the_pool() {
        let (mut state, _drill, smelter) = state_with_smelter();
        state.resources.minerals = 0.0;
        state.config.resources.base_mineral_cap = 100_000.0;

        // One delivery's worth of time.
        for _ in 0..40 {
            state.step(0.1, false);
        }

        let refined = state
            .output_buffers
            .get(&(smelter.x, smelter.y))
            .copied()
            .unwrap_or(0.0);
        assert!(refined > 0.0, "nothing reached the smelter");
        assert_eq!(
            state.resources.minerals, 0.0,
            "ore went to the pool while the smelter had room"
        );
    }

    #[test]
    fn an_unpowered_smelter_refines_nothing() {
        let mut state = state();
        state.resources.alloy = 0.0;
        // Far from the Core, so it never gets power.
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 6, core.y + 6);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.grid.reveal_around(pos, 1);
        state.unlock_building(BuildingType::Smelter);
        state.select_building(BuildingType::Smelter);
        assert!(state.try_place_building(pos));
        state.grid.update_power_grid();

        for _ in 0..100 {
            state.step(0.1, false);
        }
        assert_eq!(state.resources.alloy, 0.0);
        assert_eq!(state.alloy_rate(), 0.0);
    }

    #[test]
    fn a_smelter_with_no_minerals_left_simply_slows_down() {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.grid.reveal_around(pos, 1);
        state.unlock_building(BuildingType::Smelter);
        state.select_building(BuildingType::Smelter);
        assert!(state.try_place_building(pos));
        state.grid.update_power_grid();

        state.resources.minerals = 0.0;
        state.resources.alloy = 0.0;
        for _ in 0..100 {
            state.step(0.1, false);
        }

        assert_eq!(state.resources.alloy, 0.0);
        assert!(state.resources.minerals >= 0.0, "minerals went negative");
    }

    #[test]
    fn the_last_stages_of_the_ship_cannot_be_paid_without_alloy() {
        let stages = &crate::data::game_data().seed_ship.stages;
        let alloy_stages = stages.iter().filter(|s| s.cost.alloy > 0.0).count();
        assert!(alloy_stages >= 2, "the chain has no sink");

        let mut ship = SeedShip::default();
        let mut resources = Resources {
            minerals: 100_000.0,
            data: 100_000.0,
            biomass: 100_000.0,
            energy: 0.0,
            alloy: 0.0,
        };
        for _ in 0..stages.len() {
            ship.absorb(&mut resources, intake(), 10_000.0);
        }
        // Everything else is paid, so the ship stalls on the first alloy stage.
        assert!(!ship.is_complete());
        assert!(ship.stage().unwrap().cost.alloy > 0.0);

        resources.alloy = 100_000.0;
        for _ in 0..stages.len() {
            ship.absorb(&mut resources, intake(), 10_000.0);
        }
        assert!(ship.is_complete());
    }

    #[test]
    fn the_skyline_grows_with_the_ship_and_not_before_it() {
        let mut state = researched_state();
        assert!(
            !state.seed_ship.has_broken_ground(),
            "a yard nobody has committed to is standing"
        );
        assert_eq!(state.seed_ship.built_fraction(), 0.0);

        state.toggle_seed_ship_commitment();
        assert!(state.seed_ship.has_broken_ground());

        let mut last = 0.0;
        for _ in 0..400 {
            state.update_seed_ship(1.0);
            let now = state.seed_ship.built_fraction();
            assert!(now >= last, "the ship shrank: {} then {}", last, now);
            assert!((0.0..=1.0).contains(&now), "out of range: {}", now);
            last = now;
        }
        assert!(state.seed_ship.is_complete());
        assert_eq!(state.seed_ship.built_fraction(), 1.0);
    }

    #[test]
    fn a_stage_part_paid_counts_for_part_of_the_ship() {
        let mut state = researched_state();
        state.toggle_seed_ship_commitment();
        // Part-way into the first stage, and no further.
        state.update_seed_ship(1.0);
        let fraction = state.seed_ship.built_fraction();
        assert_eq!(state.seed_ship.stage_index(), 0);
        assert!(
            fraction > 0.0 && fraction < 1.0 / state.seed_ship.stage_count() as f32,
            "a part-paid first stage counted as {}",
            fraction
        );
    }

    #[test]
    fn a_launched_yard_is_bare_ground_again() {
        let mut state = researched_state();
        state.toggle_seed_ship_commitment();
        for _ in 0..400 {
            state.update_seed_ship(1.0);
        }
        assert_eq!(state.seed_ship.built_fraction(), 1.0);
        state.seed_ship.mark_launched();
        assert_eq!(state.seed_ship.built_fraction(), 0.0);
        assert!(!state.seed_ship.has_broken_ground());
    }

    #[test]
    fn the_ship_cannot_be_finished_on_minerals_alone() {
        let mut state = state();
        // Every resource in the world, and no research past the start.
        state.toggle_seed_ship_commitment();
        for _ in 0..2_000 {
            state.update_seed_ship(1.0);
        }

        assert!(
            !state.seed_ship.is_complete(),
            "the tech tree can be skipped entirely"
        );
        assert!(state.seed_ship_blocked_by().is_some());
    }

    #[test]
    fn a_blocked_yard_takes_nothing_rather_than_banking_it() {
        let mut state = state();
        state.toggle_seed_ship_commitment();
        // Clear the first stage, which needs no research.
        while state.seed_ship.stage_index() == 0 {
            state.update_seed_ship(1.0);
        }
        assert!(state.seed_ship_blocked_by().is_some(), "stage two is gated");

        let minerals = state.resources.minerals;
        for _ in 0..100 {
            state.update_seed_ship(1.0);
        }
        assert_eq!(
            state.resources.minerals, minerals,
            "the yard ate resources it could not use"
        );
        assert_eq!(state.seed_ship.stage_fraction(), 0.0);
    }

    #[test]
    fn the_research_that_unblocks_a_stage_gets_it_moving() {
        let mut state = state();
        state.toggle_seed_ship_commitment();
        while state.seed_ship.stage_index() == 0 {
            state.update_seed_ship(1.0);
        }

        let required = state
            .seed_ship
            .blocked_by(&state.research.unlocked_techs)
            .expect("stage two is gated")
            .to_string();
        state.research.unlocked_techs.push(required);
        state.refresh_stats();

        assert!(state.seed_ship_blocked_by().is_none());
        state.update_seed_ship(1.0);
        assert!(state.seed_ship.stage_fraction() > 0.0);
    }

    #[test]
    fn the_shipped_stages_declare_only_modifiers_the_game_can_read() {
        let stages = &crate::data::game_data().seed_ship.stages;
        let with_boons = stages.iter().filter(|s| !s.modifiers.is_empty()).count();
        assert!(with_boons >= 3, "the ship is still all cost and no payoff");
        for stage in stages {
            for modifier in &stage.modifiers {
                assert!(
                    crate::engine::parse_modifier(modifier).is_ok(),
                    "stage {} declares an unreadable modifier",
                    stage.id
                );
            }
            if !stage.modifiers.is_empty() {
                assert!(!stage.boon.is_empty(), "stage {} does not say", stage.id);
            }
        }
    }

    #[test]
    fn a_standing_stage_works_for_the_world_it_stands_on() {
        let mut state = state();
        let before = state.stats.multiplier(crate::engine::StatId::DrillOutput);

        // Finish the first stage, which pays the drills back.
        state.toggle_seed_ship_commitment();
        while state.seed_ship.stage_index() == 0 {
            state.update_seed_ship(1.0);
        }

        assert_eq!(state.seed_ship.standing_stages().len(), 1);
        assert!(
            state.stats.multiplier(crate::engine::StatId::DrillOutput) > before,
            "the finished stage did nothing"
        );
    }

    #[test]
    fn the_yards_advantages_leave_with_the_ship() {
        let mut state = researched_state();
        state.toggle_seed_ship_commitment();
        for _ in 0..2_000 {
            state.update_seed_ship(1.0);
        }
        assert!(state.seed_ship.is_complete());
        let boosted = state.stats.multiplier(crate::engine::StatId::DrillOutput);
        assert!(boosted > 1.0);

        state.seed_ship.mark_launched();
        state.refresh_stats();

        assert!(state.seed_ship.standing_stages().is_empty());
        assert!(
            state.stats.multiplier(crate::engine::StatId::DrillOutput) < boosted,
            "the ship left but its yard did not"
        );
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
            alloy: 0.0,
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
        let mut state = researched_state();
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
        let mut state = researched_state();
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
