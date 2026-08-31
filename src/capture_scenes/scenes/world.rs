//! General planetary and lifecycle staging for the screenshot harness.

use crate::{engine, state, Game, GamePhase};

impl Game {
    pub(super) fn stage_world_capture_scene(&mut self, scene: &str) -> bool {
        match scene {
            "mobile_build" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.mobile_panel = state::MobilePanel::Build;
                planet.select_building(engine::BuildingType::Drill);
            }
            "mobile_info" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                self.research_state.current_research = Some("power_grid".to_string());
                self.research_state.research_progress = 4.0;
                self.sync_research_to_planet();
                let planet = self.campaign.current_mut();
                // Keep a live research job in this otherwise quiet gameplay
                // frame so the mobile header's progress readout is verifiable
                // without covering the selected-structure information.
                planet.resources.data = 1.0;
                planet.mobile_panel = state::MobilePanel::Info;
                planet.selected_building = None;
                planet.selected_tile = planet
                    .grid
                    .find_buildings(engine::BuildingType::Drill)
                    .pop();
            }
            "mobile_help" | "mobile_help_narrow" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                self.campaign.current_mut().show_help = true;
            }
            "mobile_narrow" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
            }
            "focus" | "focus_compact" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.focus_mode = true;
                planet.camera.zoom = 1.35;
            }
            // The same world with a Drill in hand, which brings the ground up.
            "prospect" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                self.campaign
                    .current_mut()
                    .select_building(engine::BuildingType::Drill);
            }
            "forest_buffer" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                let Some(core) = planet.grid.find_core() else {
                    return true;
                };
                let forest = engine::GridPos::new(core.x - 2, core.y + 2);
                let biomass = planet.config.resources.forest_biomass;
                if let Some(tile) = planet.grid.get_mut(forest) {
                    tile.terrain = engine::TerrainType::Forest;
                    tile.building = None;
                    tile.filter = false;
                    tile.forest_cleared = false;
                    tile.biomass_amount = biomass;
                }
                planet.grid.reveal_around(forest, 4);
                planet.selected_building = None;
                planet.selected_tile = Some(forest);
                planet.notifications.clear();
            }
            // A Core that has grown past its landing pod.
            "core_stage" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                if let Some(core) = planet.grid.find_core() {
                    planet.grid.reveal_around(core, 24);
                    // Enough standing to reach the Foundry and then the
                    // Fortress, through the real path.
                    for step in 1..=26 {
                        let pos =
                            engine::GridPos::new(core.x - 1 - step % 8, core.y + 1 + step / 8);
                        if let Some(tile) = planet.grid.get_mut(pos) {
                            tile.terrain = engine::TerrainType::Empty;
                        }
                        planet
                            .grid
                            .place_building(pos, engine::BuildingType::Conduit);
                    }
                }
                for tech in ["a", "b", "c", "d", "e", "f"] {
                    planet.research.unlocked_techs.push(tech.to_string());
                }
                planet.grid.update_power_grid();
                planet.step(state::TICK_SECONDS, false);
                // The Core progression scene should also prove that selecting
                // the evolving structure exposes its stage in the ordinary
                // inspector, not only in the Factory Deck.
                planet.selected_building = None;
                planet.selected_tile = planet.grid.find_core();
            }
            // The final Core stage makes its planetary-scale silhouette
            // visible in the ordinary map, not only in the inspector text.
            "core_ring" | "core_ring_focus" | "mobile_core_ring" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.core_stage = (planet.core_stages().len().saturating_sub(1)) as u8;
                planet.refresh_stats();
                if let Some(core) = planet.grid.find_core() {
                    planet.grid.reveal_around(core, 24);
                }
                planet.tutorial_hidden = true;
                planet.tutorial_done = true;
                planet.notifications.clear();
                planet.selected_building = None;
                planet.selected_tile = planet.grid.find_core();
                if scene == "core_ring_focus" {
                    // Center the final-stage silhouette in the unobstructed
                    // map for a stable visual regression frame.
                    planet.focus_mode = true;
                    planet.camera.pan_x = 240.0;
                    planet.camera.pan_y = -100.0;
                } else if scene == "mobile_core_ring" {
                    // Mirror the visible mobile CORE recenter action so the
                    // late-stage world can be checked in its touch layout.
                    planet.camera.pan_x = -230.0;
                    planet.camera.pan_y = -100.0;
                }
            }
            // A run cut in the middle, with drones stalled beyond the break.
            "severed" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                // Long enough for a drill to fill a load and put a drone out
                // on the run; cutting it before that strands nobody.
                for _ in 0..300 {
                    planet.step(state::TICK_SECONDS, false);
                }
                // ...then take a piece of it away.
                if let Some(core) = planet.grid.find_core() {
                    let cut = engine::GridPos::new(core.x + 3, core.y);
                    planet.grid.remove_building(cut);
                    planet.grid.update_power_grid();
                }
                for _ in 0..10 {
                    planet.step(state::TICK_SECONDS, false);
                }
            }
            // The same mid-build ship, seen from the ground it is eating.
            "skyline" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
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
                let planet = self.campaign.current_mut();
                planet.config.resources.base_mineral_cap = 100_000.0;
                planet.resources.minerals = 100_000.0;
                planet.resources.data = 10_000.0;
                planet.resources.biomass = 10_000.0;
                planet.resources.alloy = 10_000.0;
                planet.toggle_seed_ship_commitment();
                for _ in 0..60 {
                    planet.update_seed_ship(1.0);
                }
            }
            "records" | "mobile_records_narrow" => {
                self.phase = GamePhase::Records;
                self.seed_logistics_scene();
                // A world part-way through the set, so the screen shows earned
                // rows, locked rows and a few half-finished bars at once.
                let planet = self.campaign.current_mut();
                planet.config.resources.base_mineral_cap = 100_000.0;
                planet.resources.minerals = 260.0;
                planet.resources.data = 30.0;
                planet.resources.alloy = 12.0;
                planet.resources.components = 10.0;
                planet.forest_harvested_count = 2;
                for tech in ["power_grid", "data_processing", "efficient_drills"] {
                    planet.research.unlocked_techs.push(tech.to_string());
                }
                // Exercise the factory-policy records end to end rather than
                // showing the new rows only as empty definitions.
                if let Some(core) = planet.grid.find_core() {
                    for step in 1..=3 {
                        let pos = engine::GridPos::new(core.x + step, core.y + 2);
                        let mut building = engine::Building::new_with_data(
                            planet.data.clone(),
                            engine::BuildingType::Smelter,
                            pos,
                        );
                        building.input_priority = true;
                        building.standby = true;
                        if let Some(tile) = planet.grid.get_mut(pos) {
                            tile.building = Some(building);
                        }
                    }
                }
                planet.refresh_stats();
                // A step is what fires them, the same as in play.
                for _ in 0..4 {
                    planet.step(state::TICK_SECONDS, false);
                }
            }
            "seedship" | "desktop_seedship_short" | "mobile_seedship" => {
                self.phase = GamePhase::SeedShip;
                self.seed_logistics_scene();
                self.campaign.current_mut().resources.alloy = 80.0;
                // Mid-build, with the swarm diverting production into the yard.
                let planet = self.campaign.current_mut();
                planet.notifications.clear();
                planet.config.resources.base_mineral_cap = 100_000.0;
                planet.resources.minerals = 400.0;
                planet.resources.data = 120.0;
                planet.toggle_seed_ship_commitment();
                for _ in 0..20 {
                    planet.update_seed_ship(1.0);
                }
                // Settle one-time fixture milestones before the first captured
                // frame so achievement and Core toasts do not cover the yard.
                planet.step(0.0, false);
                planet.notifications.clear();
                if scene == "mobile_seedship" {
                    planet.seed_ship_scroll.set_offset(90.0);
                }
            }
            "paused" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                for _ in 0..200 {
                    planet.step(state::TICK_SECONDS, false);
                }
                planet.change_speed(true);
                planet.toggle_pause();
            }
            "saved" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                // Long enough to have earned an autosave, then take it.
                for _ in 0..70 {
                    self.campaign.current_mut().step(1.0, false);
                    self.campaign.update_directive(1.0);
                }
                self.campaign.mark_saved();
            }
            "save_failed" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.save_failed = true;
                planet.notifications.clear();
            }
            "backup_recovery" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.restored_from_backup = true;
                planet.restored_from_backup_generation = 1;
                planet.notifications.clear();
            }
            "hibernation" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.battery_seconds = 0.0;
                planet.notifications.clear();
            }
            "offline" | "mobile_offline" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.last_offline_seconds = 6.0 * 60.0 * 60.0;
                planet.last_offline_simulated = 4.0 * 60.0 * 60.0;
                planet.offline_notice_timer = 8.0;
                planet.last_offline_report = state::OfflineReport {
                    elapsed_seconds: planet.last_offline_seconds,
                    capped_seconds: planet.last_offline_seconds,
                    minerals_gained: 740.0,
                    alloy_gained: 120.0,
                    components_gained: 64.0,
                    data_gained: 360.0,
                    power_gained: 18.0,
                    research_data_spent: 110.0,
                    research_completed: true,
                    seed_ship_stages_completed: 1,
                    ..state::OfflineReport::default()
                };
                planet.notifications.clear();
            }
            "demolish" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                self.campaign.current_mut().toggle_demolish_mode();
            }
            "toasts" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                // The drill in the seeded scene earns its own achievement
                // toast; these are the ones a longer session would have shown.
                planet
                    .notifications
                    .success("Research complete: Wind Power");
                planet.notifications.info("Available: Wind Turbine");
                planet
                    .notifications
                    .warning("Seed Ship: Ion Spine under way");
                planet.notifications.success("Next: tap SHIP, then MAP.");
            }
            // The banner reports a shutdown that scales with the base, so it
            // has to be looked at on a base that has some size to it.
            "collapse" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.resources.data = 400.0;
                planet.trigger_power_collapse();
            }
            "camera" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                // Framed as if the player had zoomed in and dragged the map.
                let camera = &mut self.campaign.current_mut().camera;
                camera.zoom = 1.8;
                camera.pan_x = -420.0;
                camera.pan_y = -180.0;
            }
            _ => return false,
        }
        true
    }
}
