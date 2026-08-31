//! Staging for factory, freight, and upkeep capture scenes.

use crate::{engine, state, Game, GamePhase};

impl Game {
    pub(super) fn stage_industrial_capture_scene(&mut self, scene: &str) -> bool {
        match scene {
            "logistics" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                // Select the drill's own tile, so the inspector is reading
                // ground rather than a palette choice.
                let planet = self.campaign.current_mut();
                let drill = planet
                    .grid
                    .find_buildings(engine::BuildingType::Drill)
                    .pop();
                planet.selected_tile = drill;
                // Nothing in hand: the quiet state of the ore overlay.
                planet.selected_building = None;
            }
            "freight" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                let Some(core) = planet.grid.find_core() else {
                    return true;
                };
                planet.focus_mode = true;
                planet.camera.zoom = 1.55;
                planet.notifications.clear();

                // Freeze four packets in flight on parallel approaches. The
                // scene verifies colour and silhouette independently without
                // relying on production timing to line them up for a camera.
                for (row, resource) in [
                    engine::ResourceType::Minerals,
                    engine::ResourceType::Biomass,
                    engine::ResourceType::Alloy,
                    engine::ResourceType::Components,
                ]
                .into_iter()
                .enumerate()
                {
                    let start = engine::GridPos::new(core.x + 1, core.y - 9 + row as i32 * 2);
                    let end = engine::GridPos::new(start.x + 4, start.y);
                    let id = planet.drones.spawn_drone(start);
                    if let Some(drone) = planet.drones.get_drone_mut(id) {
                        drone.dispatch(end, vec![start, end], 1.0, resource);
                        drone.path_index = 1;
                        drone.position = start;
                        drone.progress = 0.35;
                    }
                }
            }
            "upkeep" | "mobile_upkeep" => {
                self.phase = GamePhase::Playing;
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                self.research_state
                    .unlocked
                    .push("ceramic_plating".to_string());
                let building_types: Vec<_> = self
                    .data
                    .data()
                    .buildings
                    .iter()
                    .filter_map(|def| engine::BuildingType::from_id(&def.id))
                    .collect();
                let planet = self.campaign.current_mut();
                planet.resources.minerals = 10_000.0;
                planet.resources.energy = 10_000.0;
                planet.config.resources.max_energy = 10_000.0;
                for building_type in building_types {
                    planet.unlock_building(building_type);
                }
                let Some(core) = planet.grid.find_core() else {
                    return true;
                };
                planet.grid.reveal_around(core, 14);

                // A run heading east, with a shield covering only its first half.
                for step in 1..=10 {
                    let pos = engine::GridPos::new(core.x + step, core.y);
                    if let Some(tile) = planet.grid.get_mut(pos) {
                        tile.terrain = engine::TerrainType::Empty;
                        tile.building = None;
                    }
                    planet.select_building(engine::BuildingType::Conduit);
                    planet.try_place_building(pos);
                }
                let shield = engine::GridPos::new(core.x + 2, core.y + 1);
                if let Some(tile) = planet.grid.get_mut(shield) {
                    tile.terrain = engine::TerrainType::Empty;
                    tile.building = None;
                }
                planet.select_building(engine::BuildingType::ShieldGenerator);
                planet.try_place_building(shield);
                planet.grid.update_power_grid();

                // Long enough for the acid to bite where it is not held off.
                for _ in 0..90 {
                    planet.step(1.0, false);
                }
                // Leave one exposed network tile at the alert threshold so
                // the recovery card names the maintenance cause in the still.
                let alert_pos = engine::GridPos::new(core.x + 10, core.y);
                if let Some(building) = planet
                    .grid
                    .get_mut(alert_pos)
                    .and_then(|tile| tile.building.as_mut())
                {
                    building.acid_wear = 75.0;
                }
                // Leave the shield selected so its coverage is on screen.
                planet.select_building(engine::BuildingType::ShieldGenerator);
                planet.selected_tile = Some(shield);
            }
            "smelting" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.unlock_building(engine::BuildingType::Smelter);
                let Some(core) = planet.grid.find_core() else {
                    return true;
                };
                // A smelter on the run, so the drill's ore is refined on the
                // way in rather than reaching the pool.
                let smelter = engine::GridPos::new(core.x + 1, core.y - 1);
                if let Some(tile) = planet.grid.get_mut(smelter) {
                    tile.terrain = engine::TerrainType::Empty;
                    tile.building = None;
                }
                planet.select_building(engine::BuildingType::Smelter);
                planet.try_place_building(smelter);

                // A smelter costs more power than the Core makes on its own,
                // so the base needs generation before it can refine anything.
                planet.unlock_building(engine::BuildingType::WindTurbine);
                for offset in [(-1, 0), (-1, -1)] {
                    let pos = engine::GridPos::new(core.x + offset.0, core.y + offset.1);
                    if let Some(tile) = planet.grid.get_mut(pos) {
                        tile.terrain = engine::TerrainType::Empty;
                        tile.building = None;
                    }
                    planet.select_building(engine::BuildingType::WindTurbine);
                    planet.try_place_building(pos);
                }
                planet.grid.update_power_grid();
                for _ in 0..600 {
                    planet.step(0.1, false);
                }
                planet.selected_tile = Some(smelter);
            }
            "assembly" | "flow" | "factory_deck" | "mobile_factory_deck" => {
                self.stage_factory_scene(scene);
            }
            "congestion" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                // A deliberately undersized run, so the saturation readout and
                // the tile outlines are visible in a still frame.
                let planet = self.campaign.current_mut();
                for _ in 0..120 {
                    planet.step(state::TICK_SECONDS, false);
                }
                // Pile a shift of drones onto one run so the tile is over its
                // limit, and crawl them so the still frame catches the jam.
                let (Some(core), Some(drill)) = (
                    planet.grid.find_core(),
                    planet
                        .grid
                        .find_buildings(engine::BuildingType::Drill)
                        .first()
                        .copied(),
                ) else {
                    return true;
                };
                if let Some(route) = engine::route_over_network(&planet.grid, drill, core) {
                    for _ in 0..3 {
                        let id = planet.drones.spawn_drone(drill);
                        if let Some(drone) = planet.drones.get_drone_mut(id) {
                            drone.dispatch(
                                core,
                                route.clone(),
                                5.0,
                                engine::ResourceType::Minerals,
                            );
                        }
                    }
                }
                planet.drones.drone_speed = 0.05;
            }
            _ => return false,
        }
        true
    }
}
