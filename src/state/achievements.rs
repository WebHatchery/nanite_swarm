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
                    countable: milestone.is_some_and(|milestone| {
                        milestone != Milestone::Manual && milestone != Milestone::PowerSurplus
                    }) && target > 1.0,
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
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::engine::{BuildingType, GridPos};

    fn state() -> PlanetState {
        PlanetState::new(2, 42, GameConfig::default())
    }

    #[test]
    fn every_shipped_condition_is_one_the_game_can_measure() {
        for def in &crate::data::game_data().achievements {
            assert!(
                Milestone::from_id(&def.condition.kind).is_some(),
                "{} asks for \"{}\"",
                def.id,
                def.condition.kind
            );
            assert!(!def.name.is_empty());
            assert!(!def.description.is_empty());
        }
    }

    #[test]
    fn a_world_that_has_done_nothing_has_earned_almost_nothing() {
        let mut state = state();
        state.power_balance = -1.0;
        state.update_achievements();
        let (unlocked, total) = state.achievements_progress();
        assert_eq!(unlocked, 0, "something fired on an untouched world");
        assert!(total >= 10, "the shipped set is still tiny: {}", total);
    }

    #[test]
    fn holding_the_ore_a_declared_achievement_asks_for_earns_it() {
        let mut state = state();
        state.config.resources.base_mineral_cap = 100_000.0;
        state.resources.minerals = 500.0;

        state.update_achievements();

        assert!(state.achievements.is_unlocked("stockpile"));
        // And one it has not reached stays locked.
        assert!(!state.achievements.is_unlocked("refinery"));
    }

    #[test]
    fn laying_network_is_counted_by_the_tiles_that_carry_drones() {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        state.grid.reveal_around(core, 24);
        for step in 1..=20 {
            let pos = GridPos::new(core.x + step % 10, core.y + 1 + step / 10);
            if let Some(tile) = state.grid.get_mut(pos) {
                tile.terrain = crate::engine::TerrainType::Empty;
            }
            state.grid.place_building(pos, BuildingType::Conduit);
        }

        state.update_achievements();

        assert!(state.achievements.is_unlocked("network"));
    }

    #[test]
    fn a_manual_achievement_never_fires_by_itself() {
        let mut state = state();
        // Everything a measured condition could want, several times over.
        state.config.resources.base_mineral_cap = 100_000.0;
        state.resources.minerals = 100_000.0;
        state.resources.data = 100_000.0;
        state.resources.alloy = 100_000.0;
        state.power_balance = 100.0;

        state.update_achievements();

        assert!(!state.achievements.is_unlocked("seed_ship"));
        assert!(!state.achievements.is_unlocked("system_consumed"));
        // It still fires when the code that owns it says so.
        state.announce_achievement("seed_ship");
        assert!(state.achievements.is_unlocked("seed_ship"));
    }

    #[test]
    fn what_the_swarm_is_told_outlives_the_toast_that_said_it() {
        let mut state = state();
        state.config.resources.base_mineral_cap = 100_000.0;
        state.resources.minerals = 500.0;
        state.update_achievements();

        // Long enough that nothing is left on screen.
        state.notifications.update(60.0);
        assert!(state.notifications.get_notifications().is_empty());
        assert!(
            state
                .notifications
                .history()
                .iter()
                .any(|entry| entry.message.contains("Stockpile")),
            "the announcement was the only copy and it is gone"
        );
    }

    fn record(state: &PlanetState, name: &str) -> AchievementRecord {
        state
            .achievement_records()
            .into_iter()
            .find(|record| record.name == name)
            .expect("the shipped set still has this one")
    }

    #[test]
    fn the_records_cover_the_declared_set_in_declaration_order() {
        let records = state().achievement_records();
        let declared = &crate::data::game_data().achievements;
        assert_eq!(records.len(), declared.len());
        for (record, def) in records.iter().zip(declared) {
            assert_eq!(record.name, def.name);
            assert!(!record.unlocked, "nothing is earned on an untouched world");
        }
    }

    #[test]
    fn a_locked_record_says_how_far_along_it_is() {
        let mut state = state();
        state.config.resources.base_mineral_cap = 100_000.0;
        state.resources.minerals = 250.0;

        let stockpile = record(&state, "Stockpile");
        assert!(!stockpile.unlocked);
        assert!(stockpile.countable, "500 minerals is worth counting");
        assert_eq!(stockpile.progress, Some(250.0));
        assert_eq!(stockpile.target, 500.0);
        assert!((stockpile.fraction() - 0.5).abs() < 0.001);
    }

    #[test]
    fn an_earned_record_stays_full_however_the_world_moves_on() {
        let mut state = state();
        state.config.resources.base_mineral_cap = 100_000.0;
        state.resources.minerals = 500.0;
        state.update_achievements();
        // Spent again, right back down to nothing.
        state.resources.minerals = 0.0;

        let stockpile = record(&state, "Stockpile");
        assert!(stockpile.unlocked);
        assert_eq!(stockpile.fraction(), 1.0);
    }

    #[test]
    fn a_record_with_nothing_to_count_does_not_pretend_otherwise() {
        let state = state();
        // Announced by code, so there is no running total behind it.
        let manual = record(&state, "Seed Ship");
        assert!(!manual.countable);
        assert_eq!(manual.fraction(), 0.0);
        // A one-shot condition is done or not; "0 / 1" says nothing.
        let surplus = record(&state, "Power Surplus");
        assert!(!surplus.countable);
    }
}
