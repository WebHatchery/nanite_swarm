//! Factory and production-room screenshot staging.

use crate::{engine, state, Game, GamePhase};

impl Game {
    pub(super) fn stage_factory_scene(&mut self, scene: &str) {
        self.phase = GamePhase::Playing;
        self.seed_logistics_scene();
        for tech in [
            "power_grid",
            "wind_power",
            "smelting",
            "data_processing",
            "advanced_research",
            "precision_assembly",
            "adaptive_clocking",
        ] {
            if !self.research_state.unlocked.iter().any(|id| id == tech) {
                self.research_state.unlocked.push(tech.to_string());
            }
        }
        self.sync_building_unlocks();
        self.sync_research_to_planet();
        let planet = self.campaign.current_mut();
        let Some(core) = planet.grid.find_core() else {
            return;
        };
        planet.unlock_building(engine::BuildingType::Assembler);
        planet.unlock_building(engine::BuildingType::Smelter);
        planet.unlock_building(engine::BuildingType::WindTurbine);

        let assembler = engine::GridPos::new(core.x + 2, core.y - 1);
        if let Some(tile) = planet.grid.get_mut(assembler) {
            tile.terrain = engine::TerrainType::Empty;
            tile.building = None;
        }
        planet.select_building(engine::BuildingType::Assembler);
        planet.try_place_building(assembler);
        if let Some(building) = planet
            .grid
            .get_mut(assembler)
            .and_then(|tile| tile.building.as_mut())
        {
            building.overclocked = true;
            building.input_priority = true;
        }

        let smelter = engine::GridPos::new(core.x + 4, core.y - 1);
        if let Some(tile) = planet.grid.get_mut(smelter) {
            tile.terrain = engine::TerrainType::Empty;
            tile.building = None;
        }
        planet.select_building(engine::BuildingType::Smelter);
        planet.try_place_building(smelter);

        for offset in [(-1, 0), (-1, -1), (0, -1)] {
            let pos = engine::GridPos::new(core.x + offset.0, core.y + offset.1);
            if let Some(tile) = planet.grid.get_mut(pos) {
                tile.terrain = engine::TerrainType::Empty;
                tile.building = None;
            }
            planet.select_building(engine::BuildingType::WindTurbine);
            planet.try_place_building(pos);
        }
        planet.grid.update_power_grid();
        let key = (assembler.x, assembler.y);
        planet.input_hoppers.insert(
            key,
            [
                (engine::ResourceType::Minerals, 20.0),
                (engine::ResourceType::Alloy, 10.0),
            ]
            .into_iter()
            .collect(),
        );
        planet.input_buffers.insert(key, 30.0);
        let smelter_key = (smelter.x, smelter.y);
        planet.input_hoppers.insert(
            smelter_key,
            [(engine::ResourceType::Minerals, 24.0)]
                .into_iter()
                .collect(),
        );
        planet.input_buffers.insert(smelter_key, 24.0);
        for _ in 0..60 {
            planet.step(state::TICK_SECONDS, false);
        }
        let pad_capacity = planet.processor_pad_capacity() + planet.drones.drone_capacity;
        planet.output_buffers.insert(key, pad_capacity);
        if let Some(hopper) = planet.input_hoppers.get_mut(&key) {
            hopper.insert(engine::ResourceType::Minerals, 0.0);
            planet.input_buffers.insert(key, hopper.values().sum());
        }
        planet.select_building(engine::BuildingType::Assembler);
        planet.selected_tile = Some(assembler);
        planet.box_selected = vec![assembler, smelter];
        if scene == "flow" {
            planet.box_selected.clear();
            planet.auto_clocking = true;
            if let Some(building) = planet
                .grid
                .get_mut(assembler)
                .and_then(|tile| tile.building.as_mut())
            {
                building.overclocked = false;
            }
            planet.flow_overlay = true;
            planet.focus_mode = true;
            planet.camera.zoom = 1.45;
            planet.camera.pan_y = -72.0;
            planet.selected_building = None;
            planet.notifications.clear();
        } else if matches!(scene, "factory_deck" | "mobile_factory_deck") {
            planet.factory_deck_open = true;
            planet.factory_focus = state::FactoryFocus::Assembly;
            planet.core_stage = 2;
            planet.camera.zoom = 1.22;
            planet.camera.pan_y = -36.0;
            planet.selected_building = None;
            planet.selected_tile = None;
            planet.notifications.clear();
        }
    }
}
