//! Capture scenes for world-specific power systems.

use crate::{engine, Game, GamePhase};

impl Game {
    pub(super) fn stage_venus_scene(&mut self) {
        self.phase = GamePhase::Playing;
        self.campaign.colonize(1);
        self.campaign.travel_to(1);
        self.research_state.unlocked.extend(
            ["ceramic_plating", "heater_nodes"]
                .into_iter()
                .map(str::to_string),
        );
        let building_types: Vec<_> = self
            .data
            .data()
            .buildings
            .iter()
            .filter_map(|def| engine::BuildingType::from_id(&def.id))
            .collect();
        let planet = self.campaign.current_mut();
        // Everything researched, so the palette shows what this world
        // refuses rather than what the swarm has not reached yet.
        for building_type in building_types {
            planet.unlock_building(building_type);
        }
        if let Some(core) = planet.grid.find_core() {
            planet.grid.reveal_around(core, 12);
            // Venus is the world of gaps, so bridge one: it is the only piece
            // of network that can stand on void.
            let void: Vec<engine::GridPos> = planet
                .grid
                .iter_tiles()
                .filter(|(_, tile)| tile.terrain == engine::TerrainType::Void)
                .map(|(pos, _)| pos)
                .filter(|pos| (pos.x - core.x).abs() + (pos.y - core.y).abs() < 6)
                .take(3)
                .collect();
            for pos in void {
                planet
                    .grid
                    .place_building(pos, engine::BuildingType::Bridge);
            }
            planet.grid.update_power_grid();
        }
    }

    pub(super) fn stage_geothermal_scene(&mut self) {
        self.stage_power_source_scene(1, engine::BuildingType::GeothermalTap);
    }

    pub(super) fn stage_fusion_scene(&mut self) {
        self.stage_power_source_scene(3, engine::BuildingType::FusionReactor);
    }

    fn stage_power_source_scene(&mut self, planet_index: usize, source: engine::BuildingType) {
        self.phase = GamePhase::Playing;
        self.campaign.colonize(planet_index);
        self.campaign.travel_to(planet_index);

        let research = match source {
            engine::BuildingType::GeothermalTap => ["power_grid", "geothermal_tapping"].as_slice(),
            engine::BuildingType::FusionReactor => {
                ["power_grid", "heater_nodes", "fusion_reactors"].as_slice()
            }
            _ => return,
        };
        self.research_state
            .unlocked
            .extend(research.iter().map(|id| (*id).to_string()));
        self.sync_research_to_planet();

        let planet = self.campaign.current_mut();
        planet.resources.minerals = 10_000.0;
        planet.resources.energy = 10_000.0;
        planet.config.resources.max_energy = 10_000.0;
        let Some(core) = planet.grid.find_core() else {
            return;
        };
        let source_pos = engine::GridPos::new(core.x - 1, core.y);
        if let Some(tile) = planet.grid.get_mut(source_pos) {
            tile.terrain = engine::TerrainType::Empty;
            tile.building = None;
            tile.revealed = true;
        }
        planet.grid.reveal_around(source_pos, 3);
        planet.unlock_building(source);
        planet.select_building(source);
        assert!(planet.try_place_building(source_pos));
        planet.grid.update_power_grid();
        planet.selected_tile = Some(source_pos);
        planet.build_palette_scroll.set_offset(10_000.0);
        planet.notifications.clear();
    }
}
