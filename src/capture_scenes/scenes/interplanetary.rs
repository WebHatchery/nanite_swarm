//! Interplanetary screenshot staging.

use crate::{engine, Game, GamePhase};

impl Game {
    pub(super) fn stage_interplanetary_scene(&mut self) {
        self.phase = GamePhase::Interplanetary;
        // Every tech the ship's later stages are gated on, or the yard
        // stalls on the Spine and the map's "Seed Ship: READY" is a
        // promise the scene cannot keep.
        for tech in ["efficient_drills", "advanced_research", "mass_driver"] {
            self.research_state.unlocked.push(tech.to_string());
        }
        self.sync_research_to_planet();
        self.campaign.colonize(4);
        // Something producing on the world left behind, so the map has
        // a stockpile to report.
        if let Some(core) = self
            .campaign
            .stockpile(4)
            .and(self.campaign.current().grid.find_core())
        {
            self.campaign.travel_to(4);
            let away = self.campaign.current_mut();
            away.config.resources.base_mineral_cap = 100_000.0;
            let drill = engine::GridPos::new(core.x + 1, core.y);
            if let Some(tile) = away.grid.get_mut(drill) {
                tile.terrain = engine::TerrainType::Empty;
            }
            away.grid.reveal_around(drill, 1);
            away.select_building(engine::BuildingType::Drill);
            away.try_place_building(drill);
            away.grid.update_power_grid();
            self.campaign.travel_to(2);
            for _ in 0..400 {
                self.campaign.update_background(1.0);
            }
        }
        // A ship on the pad, so the map shows a launch is possible.
        let planet = self.campaign.current_mut();
        planet.config.resources.base_mineral_cap = 1_000_000.0;
        planet.resources.minerals = 100_000.0;
        planet.resources.data = 100_000.0;
        planet.resources.biomass = 100_000.0;
        planet.resources.alloy = 100_000.0;
        planet.resources.components = 100_000.0;
        planet.toggle_seed_ship_commitment();
        for _ in 0..2_000 {
            planet.update_seed_ship(1.0);
        }
        self.seed_shipping_scene(4);
    }
}
