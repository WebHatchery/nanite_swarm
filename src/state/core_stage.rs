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
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::engine::{BuildingType, GridPos, StatId, TerrainType};

    fn state() -> PlanetState {
        PlanetState::new(2, 42, GameConfig::default())
    }

    /// Stand `count` conduits beside the Core, which is both structures and
    /// network as far as the milestones are concerned.
    fn build(state: &mut PlanetState, count: i32) {
        let core = state.grid.find_core().unwrap();
        state.grid.reveal_around(core, 32);
        for step in 1..=count {
            let pos = GridPos::new(core.x + step % 10, core.y + 1 + step / 10);
            if let Some(tile) = state.grid.get_mut(pos) {
                tile.terrain = TerrainType::Empty;
            }
            state.grid.place_building(pos, BuildingType::Conduit);
        }
    }

    #[test]
    fn a_world_lands_on_the_first_stage_and_no_further() {
        let mut state = state();
        state.update_core_stage();
        assert_eq!(state.core_stage, 0);
        assert_eq!(state.core_stage_def().unwrap().id, "crash_lander");
    }

    #[test]
    fn building_enough_of_a_base_grows_the_core() {
        let mut state = state();
        build(&mut state, 12);
        state.update_core_stage();

        assert_eq!(state.core_stage, 1);
        assert_eq!(state.core_stage_def().unwrap().id, "foundry");
        assert!(
            !state.notifications.is_empty(),
            "the Core grew without saying so"
        );
    }

    #[test]
    fn a_stage_the_core_reached_is_not_given_back_when_the_base_is_torn_down() {
        let mut state = state();
        build(&mut state, 12);
        state.update_core_stage();
        assert_eq!(state.core_stage, 1);

        let standing: Vec<GridPos> = state
            .grid
            .iter_tiles()
            .filter(|(_, tile)| {
                tile.building
                    .as_ref()
                    .is_some_and(|b| b.building_type == BuildingType::Conduit)
            })
            .map(|(pos, _)| pos)
            .collect();
        for pos in standing {
            state.grid.remove_building(pos);
        }
        state.update_core_stage();

        assert_eq!(state.core_stage, 1, "the Core walked backwards");
    }

    #[test]
    fn a_stage_that_is_standing_works_for_the_world_it_stands_on() {
        let mut state = state();
        let before = state.mineral_capacity();
        build(&mut state, 12);
        state.update_core_stage();

        assert!(
            state.mineral_capacity() > before,
            "the Foundry did nothing: {} then {}",
            before,
            state.mineral_capacity()
        );
    }

    #[test]
    fn the_core_can_run_through_several_stages_in_one_go() {
        let mut state = state();
        // Everything the second and third stages ask for at once.
        build(&mut state, 30);
        for tech in ["a", "b", "c", "d", "e", "f"] {
            state.research.unlocked_techs.push(tech.to_string());
        }
        state.update_core_stage();

        assert_eq!(state.core_stage, 2);
        assert_eq!(state.core_stages_reached().len(), 3);
    }

    #[test]
    fn every_declared_stage_asks_for_something_the_game_can_measure() {
        for stage in core_stages() {
            assert!(!stage.name.is_empty());
            for requirement in &stage.requires {
                assert!(
                    Milestone::from_id(&requirement.kind).is_some(),
                    "{} asks for \"{}\"",
                    stage.id,
                    requirement.kind
                );
            }
            for modifier in &stage.modifiers {
                assert!(crate::engine::parse_modifier(modifier).is_ok());
            }
        }
        // And the art has a frame for each of them.
        assert_eq!(core_stages().len(), 5);
    }

    #[test]
    fn the_last_stage_leaves_the_core_where_it_is() {
        let mut state = state();
        state.core_stage = (core_stages().len() - 1) as u8;
        // The same rebuild a loaded save gets, since nothing advanced here.
        state.refresh_stats();
        state.update_core_stage();

        assert_eq!(state.core_stage as usize, core_stages().len() - 1);
        assert!(
            state.stats.multiplier(StatId::DrillOutput) > 1.0,
            "the last stage stopped working once it was the last one"
        );
    }
}
