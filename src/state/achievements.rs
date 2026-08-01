//! What the swarm gets told it has done.
//!
//! The set lives in `assets/achievements.json`. Only the *conditions* are code,
//! for the same reason the directive kinds are: something has to know how to
//! measure them. Adding an achievement that reuses a condition is a line of
//! JSON.

use crate::data::AchievementDef;
use crate::engine::BuildingType;

use super::game_state::PlanetState;

/// What has to be true for an achievement to fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AchievementCondition {
    /// Announced by code at the moment it happens, not measured from state.
    Manual,
    Drills,
    ServerBanks,
    Structures,
    NetworkTiles,
    Drones,
    MineralsHeld,
    DataHeld,
    AlloyHeld,
    Technologies,
    ForestsHarvested,
    PowerSurplus,
    SeedShipStages,
}

impl AchievementCondition {
    pub fn id(self) -> &'static str {
        match self {
            AchievementCondition::Manual => "manual",
            AchievementCondition::Drills => "drills",
            AchievementCondition::ServerBanks => "server_banks",
            AchievementCondition::Structures => "structures",
            AchievementCondition::NetworkTiles => "network_tiles",
            AchievementCondition::Drones => "drones",
            AchievementCondition::MineralsHeld => "minerals_held",
            AchievementCondition::DataHeld => "data_held",
            AchievementCondition::AlloyHeld => "alloy_held",
            AchievementCondition::Technologies => "technologies",
            AchievementCondition::ForestsHarvested => "forests_harvested",
            AchievementCondition::PowerSurplus => "power_surplus",
            AchievementCondition::SeedShipStages => "seed_ship_stages",
        }
    }

    pub const ALL: [AchievementCondition; 13] = [
        AchievementCondition::Manual,
        AchievementCondition::Drills,
        AchievementCondition::ServerBanks,
        AchievementCondition::Structures,
        AchievementCondition::NetworkTiles,
        AchievementCondition::Drones,
        AchievementCondition::MineralsHeld,
        AchievementCondition::DataHeld,
        AchievementCondition::AlloyHeld,
        AchievementCondition::Technologies,
        AchievementCondition::ForestsHarvested,
        AchievementCondition::PowerSurplus,
        AchievementCondition::SeedShipStages,
    ];

    pub fn from_id(id: &str) -> Option<Self> {
        AchievementCondition::ALL
            .into_iter()
            .find(|condition| condition.id() == id)
    }
}

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
            .filter(|def| self.condition_met(def))
            .map(|def| def.id.clone())
            .collect();
        for id in earned {
            self.announce_achievement(&id);
        }
    }

    fn condition_met(&self, def: &AchievementDef) -> bool {
        let Some(condition) = AchievementCondition::from_id(&def.condition.kind) else {
            return false;
        };
        let target = def.condition.target;
        match condition {
            AchievementCondition::Manual => false,
            AchievementCondition::Drills => self.count_of(BuildingType::Drill) >= target,
            AchievementCondition::ServerBanks => self.count_of(BuildingType::ServerBank) >= target,
            AchievementCondition::Structures => self.grid.total_buildings() as f32 >= target,
            AchievementCondition::NetworkTiles => self.network_tile_count() as f32 >= target,
            AchievementCondition::Drones => self.drones.total_count() as f32 >= target,
            AchievementCondition::MineralsHeld => self.resources.minerals >= target,
            AchievementCondition::DataHeld => self.resources.data >= target,
            AchievementCondition::AlloyHeld => self.resources.alloy >= target,
            AchievementCondition::Technologies => {
                self.research.unlocked_techs.len() as f32 >= target
            }
            AchievementCondition::ForestsHarvested => self.forest_harvested_count as f32 >= target,
            // A surplus is a surplus; the target is only there to ask for a
            // bigger one.
            AchievementCondition::PowerSurplus => self.power_balance > 0.0,
            AchievementCondition::SeedShipStages => self.seed_ship.stage_index() as f32 >= target,
        }
    }

    fn count_of(&self, building_type: BuildingType) -> f32 {
        self.grid.find_buildings(building_type).len() as f32
    }

    /// Tiles that carry drones, which is the size of the network the player
    /// has actually laid.
    fn network_tile_count(&self) -> usize {
        self.grid
            .iter_tiles()
            .filter(|(_, tile)| {
                tile.building
                    .as_ref()
                    .is_some_and(|building| building.carries_traffic())
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::engine::GridPos;

    fn state() -> PlanetState {
        PlanetState::new(2, 42, GameConfig::default())
    }

    #[test]
    fn every_shipped_condition_is_one_the_game_can_measure() {
        for def in &crate::data::game_data().achievements {
            assert!(
                AchievementCondition::from_id(&def.condition.kind).is_some(),
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
