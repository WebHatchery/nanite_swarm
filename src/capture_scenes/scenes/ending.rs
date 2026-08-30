//! Campaign-ending screenshot staging.

use super::super::fixtures::seed_capture_launch_rail;
use crate::state::Campaign;
use crate::{state, Game, GamePhase};

impl Game {
    pub(super) fn stage_ending_scene(&mut self) {
        self.phase = GamePhase::CampaignComplete;
        // Play the campaign out the way it is actually played: every world
        // reached by building a ship and riding it there, so the numbers on
        // the ending screen are real.
        let required_techs: Vec<String> = self
            .data
            .data()
            .seed_ship
            .stages
            .iter()
            .filter_map(|stage| stage.requires.clone())
            .collect();
        self.research_state.unlocked.extend(required_techs);
        self.sync_research_to_planet();

        let build_ship = |campaign: &mut Campaign| {
            let planet = campaign.current_mut();
            planet.config.resources.base_mineral_cap = 1_000_000.0;
            planet.resources.minerals = 100_000.0;
            planet.resources.data = 100_000.0;
            planet.resources.biomass = 100_000.0;
            planet.resources.alloy = 100_000.0;
            planet.resources.components = 100_000.0;
            // The fixture runs enough world time to make the ending statistics
            // legible. Keep its launch rail powered and clean while that
            // presentation time passes.
            planet.config.resources.max_energy = 100_000.0;
            planet.resources.energy = 100_000.0;
            planet.config.upkeep.dust_rate = 0.0;
            planet.config.collapse.negative_power_seconds = f32::MAX;
            // A Seed Ship still needs a live launch rail on every world. The
            // ending scene walks the same infrastructure path as real play.
            seed_capture_launch_rail(planet);
            if !planet.seed_ship.committed {
                planet.toggle_seed_ship_commitment();
            }
            for _ in 0..2_000 {
                planet.update_seed_ship(1.0);
                planet.step(1.0, false);
            }
        };

        for _ in 0..state::PLANET_COUNT {
            build_ship(&mut self.campaign);
            let target = (0..state::PLANET_COUNT).find(|index| !self.campaign.is_colonized(*index));
            match target {
                Some(index) => {
                    self.campaign.launch_seed_ship(index);
                }
                // Nowhere left: the last ship is the ending.
                None => break,
            }
        }
    }
}
