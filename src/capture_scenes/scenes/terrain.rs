//! Screenshot staging for terrain and named planetary features.

use crate::{engine, state, Game, GamePhase};

impl Game {
    pub(super) fn stage_feature_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        self.seed_logistics_scene();
        prepare_feature_inspection(self.campaign.current_mut());
    }

    pub(super) fn stage_partial_feature_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        self.seed_logistics_scene();
        let planet = self.campaign.current_mut();
        planet.tutorial_hidden = true;
        planet.tutorial_done = true;
        planet.core_stage = 1;
        planet.refresh_stats();
        planet.notifications.clear();

        let Some(feature) = planet.features.first().cloned() else {
            return;
        };
        let grid_width = planet.grid.width;
        let grid_height = planet.grid.height;
        let Some(selected) = planet
            .grid
            .iter_tiles()
            .find(|(pos, tile)| {
                tile.building.is_none() && feature.contains_tile(*pos, grid_width, grid_height)
            })
            .map(|(pos, _)| pos)
        else {
            return;
        };
        for (pos, tile) in planet.grid.iter_tiles_mut() {
            if feature.contains_tile(pos, grid_width, grid_height) {
                tile.revealed = false;
            }
        }
        planet.grid.get_mut(selected).unwrap().revealed = true;
        planet.selected_building = None;
        planet.selected_tile = Some(selected);
        planet.mobile_panel = state::MobilePanel::Info;
        planet.camera.zoom = 1.2;
    }

    pub(super) fn stage_iron_belt_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        // Keep this a real campaign transition: the Iron Belt belongs to
        // Mercury, so the still first reaches that world instead of swapping
        // the current planet's identity underneath the UI.
        self.seed_logistics_scene();
        self.campaign.colonize(0);
        self.campaign.travel_to(0);
        prepare_feature_inspection(self.campaign.current_mut());
    }

    pub(super) fn stage_iron_belt_drill_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        self.seed_logistics_scene();
        self.campaign.colonize(0);
        self.campaign.travel_to(0);
        let planet = self.campaign.current_mut();
        prepare_feature_inspection(planet);
        let Some(pos) = planet.selected_tile else {
            return;
        };
        planet.resources.minerals = 500.0;
        planet.resources.energy = 500.0;
        planet.config.resources.max_energy = 500.0;
        planet.unlock_building(engine::BuildingType::Drill);
        planet.select_building(engine::BuildingType::Drill);
        assert!(planet.try_place_building(pos));
        planet.selected_building = None;
        planet.selected_tile = Some(pos);
        planet.notifications.clear();
        planet.grid.update_power_grid();
    }

    pub(super) fn stage_mountain_wind_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        self.seed_logistics_scene();
        let planet = self.campaign.current_mut();
        let Some(core) = planet.grid.find_core() else {
            return;
        };
        planet.tutorial_hidden = true;
        planet.tutorial_done = true;
        planet.core_stage = 1;
        planet.refresh_stats();
        planet.notifications.clear();
        planet.mobile_panel = state::MobilePanel::Info;
        let pos = engine::GridPos::new(core.x - 1, core.y);
        if let Some(tile) = planet.grid.get_mut(pos) {
            tile.terrain = engine::TerrainType::Mountain;
            tile.building = None;
            tile.revealed = true;
        }
        planet.grid.reveal_around(pos, 1);
        planet.unlock_building(engine::BuildingType::WindTurbine);
        planet.select_building(engine::BuildingType::WindTurbine);
        assert!(planet.try_place_building(pos));
        planet.selected_building = None;
        planet.selected_tile = Some(pos);
        planet.notifications.clear();
        planet.grid.update_power_grid();
    }

    pub(super) fn stage_forest_harvester_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        self.seed_logistics_scene();
        let planet = self.campaign.current_mut();
        let Some(core) = planet.grid.find_core() else {
            return;
        };
        planet.tutorial_hidden = true;
        planet.tutorial_done = true;
        planet.core_stage = 1;
        planet.refresh_stats();
        planet.notifications.clear();
        planet.mobile_panel = state::MobilePanel::Info;
        let pos = engine::GridPos::new(core.x - 1, core.y);
        if let Some(tile) = planet.grid.get_mut(pos) {
            tile.terrain = engine::TerrainType::Forest;
            tile.building = None;
            tile.revealed = true;
            tile.biomass_amount = 100.0;
        }
        planet.grid.reveal_around(pos, 1);
        planet.unlock_building(engine::BuildingType::BiomassHarvester);
        planet.select_building(engine::BuildingType::BiomassHarvester);
        assert!(planet.try_place_building(pos));
        planet.selected_building = None;
        planet.selected_tile = Some(pos);
        planet.notifications.clear();
        planet.grid.update_power_grid();
    }

    pub(super) fn stage_filter_forest_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        self.seed_logistics_scene();
        let planet = self.campaign.current_mut();
        let Some(core) = planet.grid.find_core() else {
            return;
        };
        planet.tutorial_hidden = true;
        planet.tutorial_done = true;
        planet.core_stage = 1;
        planet.refresh_stats();
        planet.notifications.clear();
        planet.mobile_panel = state::MobilePanel::Info;
        let pos = engine::GridPos::new(core.x - 1, core.y);
        if let Some(tile) = planet.grid.get_mut(pos) {
            tile.terrain = engine::TerrainType::Forest;
            tile.building = None;
            tile.revealed = true;
            tile.biomass_amount = 100.0;
        }
        planet.grid.reveal_around(pos, 1);
        assert!(planet.try_convert_forest_to_filter(pos));
        planet.selected_building = None;
        planet.selected_tile = Some(pos);
        planet.notifications.clear();
        planet.grid.update_power_grid();
    }

    pub(super) fn stage_acid_sea_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        // Keep the capture on Venus so the feature's local corrosion effect
        // can be read beside the authored global acid field.
        self.seed_logistics_scene();
        self.campaign.colonize(1);
        self.campaign.travel_to(1);
        prepare_feature_inspection(self.campaign.current_mut());
    }

    pub(super) fn stage_ring_shadow_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        // Keep the capture on Saturn so the feature's local cold effect can
        // be read beside the authored Ring Cold field.
        self.seed_logistics_scene();
        self.campaign.colonize(4);
        self.campaign.travel_to(4);
        prepare_feature_inspection(self.campaign.current_mut());
    }

    pub(super) fn stage_ice_rift_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        // Keep the capture on Jupiter so the feature's local cooling effect
        // can be read beside the authored Polar Cold field.
        self.seed_logistics_scene();
        self.campaign.colonize(3);
        self.campaign.travel_to(3);
        prepare_feature_inspection(self.campaign.current_mut());
    }

    pub(super) fn stage_landing_crater_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        // Keep the capture on Mars so the first foothold feature remains tied
        // to the starting world and its terrain harvest.
        self.seed_logistics_scene();
        prepare_feature_inspection(self.campaign.current_mut());
    }
}

fn prepare_feature_inspection(planet: &mut state::PlanetState) {
    planet.tutorial_hidden = true;
    planet.tutorial_done = true;
    // The seeded network is large enough to awaken the first Core stage
    // on the next frame. Settle that authored milestone here so the
    // feature dossier, not a one-time progression toast, owns the still.
    planet.core_stage = 1;
    planet.refresh_stats();
    planet.notifications.clear();

    let Some(feature) = planet.features.first().cloned() else {
        return;
    };
    let selected = (0..planet.grid.height)
        .flat_map(|y| (0..planet.grid.width).map(move |x| engine::GridPos::new(x as i32, y as i32)))
        .find(|pos| {
            feature.contains_tile(*pos, planet.grid.width, planet.grid.height)
                && planet.grid.get(*pos).is_some_and(|tile| {
                    tile.building.is_none()
                        && match feature.id.as_str() {
                            "landing_crater" => tile.terrain.is_harvestable(planet.data()),
                            "iron_belt" | "acid_sea" | "ring_shadow" | "ice_rift" => {
                                tile.terrain.is_buildable(planet.data())
                                    && if feature.id == "ice_rift" {
                                        pos.neighbors().iter().any(|neighbor| {
                                            planet.grid.get(*neighbor).is_some_and(|adjacent| {
                                                adjacent.terrain == engine::TerrainType::Water
                                            })
                                        })
                                    } else {
                                        true
                                    }
                            }
                            _ => true,
                        }
                })
        })
        .or_else(|| {
            (0..planet.grid.height)
                .flat_map(|y| {
                    (0..planet.grid.width).map(move |x| engine::GridPos::new(x as i32, y as i32))
                })
                .find(|pos| {
                    feature.contains_tile(*pos, planet.grid.width, planet.grid.height)
                        && planet
                            .grid
                            .get(*pos)
                            .is_some_and(|tile| tile.building.is_none())
                })
        });
    let Some(pos) = selected else {
        return;
    };
    let feature_radius = (feature.bounds.2.max(feature.bounds.3)
        * planet.grid.width.max(planet.grid.height) as f32)
        .ceil() as u32
        + 1;
    planet.grid.reveal_around(pos, feature_radius.max(3));
    planet.selected_building = None;
    planet.selected_tile = Some(pos);
    planet.mobile_panel = state::MobilePanel::Info;
    planet.camera.zoom = 1.2;
}
