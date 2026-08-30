//! Interplanetary screenshot staging.

use crate::{engine, state, Game, GamePhase};

impl Game {
    pub(super) fn stage_interplanetary_scene(&mut self) {
        self.phase = GamePhase::Interplanetary;
        self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Worlds;
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

    pub(super) fn stage_interplanetary_pad_offline_scene(&mut self) {
        self.stage_interplanetary_scene();
        let pad = self.campaign.planet(4).and_then(|planet| {
            planet
                .grid
                .find_buildings(engine::BuildingType::LandingPad)
                .first()
                .copied()
        });
        if let Some(pad) = pad {
            if let Some(building) = self
                .campaign
                .planet_mut(4)
                .and_then(|planet| planet.grid.get_mut(pad))
                .and_then(|tile| tile.building.as_mut())
            {
                building.standby = true;
            }
        }
    }

    pub(super) fn stage_interplanetary_remote_orders_scene(&mut self) {
        self.stage_interplanetary_scene();
        // Open the route that lives on the remote world. The local Mars order
        // is useful for the ordinary shipping capture, but this scene exists
        // to prove that a player can inspect and edit a standing order after
        // leaving its source world.
        self.shipping_edit_world = 4;
        self.shipping_scroll.set_offset(0.0);
        if let Some(planet) = self.campaign.planet_mut(4) {
            planet.export = Some(state::ExportOrder {
                target: 2,
                cargo: engine::ResourceType::Alloy,
                ..state::ExportOrder::default()
            });
        }
    }

    pub(super) fn stage_interplanetary_core_stages_scene(&mut self) {
        self.stage_interplanetary_scene();
        for index in [1, 3] {
            self.campaign.colonize(index);
        }

        // A map should show progression as a system-wide story, not only for
        // the world currently underfoot. These snapshots deliberately span
        // the named Core silhouettes from Crash-Lander through Ring.
        for (index, stage) in [(1, 1_u8), (2, 5), (3, 4), (4, 3)] {
            if let Some(planet) = self.campaign.planet_mut(index) {
                planet.core_stage = stage.saturating_sub(1);
                planet.refresh_stats();
                planet.notifications.clear();
            }
        }
        let achievement_ids = self
            .campaign
            .current()
            .achievements
            .iter()
            .map(|achievement| achievement.id.clone())
            .collect::<Vec<_>>();
        let current = self.campaign.current_mut();
        for id in achievement_ids {
            current.achievements.unlock(&id);
        }
        current.notifications.clear();
        self.campaign.travel_to(2);
    }

    pub(super) fn stage_mobile_interplanetary_alert_scene(&mut self) {
        self.stage_interplanetary_scene();
        self.campaign.travel_to(4);
        self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Worlds;
    }
}
