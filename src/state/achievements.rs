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
}
