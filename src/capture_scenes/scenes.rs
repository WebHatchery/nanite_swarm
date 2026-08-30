//! Screenshot scene staging implementation.

use crate::state::LaunchSequence;
use crate::{engine, slot_name, state, Game, GamePhase};

mod arrival;
mod ending;
mod factory;
mod interplanetary;
mod power;

impl Game {
    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.capture_still = true;
        // Batched captures reuse one game process; view-only state from one
        // scene must not leak into the next image.
        self.campaign.current_mut().focus_mode = false;
        self.campaign.current_mut().mobile_panel = state::MobilePanel::Map;
        self.campaign.current_mut().flow_overlay = false;
        self.campaign.current_mut().factory_deck_open = false;
        self.campaign.current_mut().show_help = false;
        self.campaign.current_mut().auto_clocking = false;
        self.campaign.current_mut().save_failed = false;
        self.has_active_campaign = false;
        self.campaign.current_mut().restored_from_backup = false;
        self.campaign.current_mut().restored_from_backup_generation = 0;
        self.research_state.current_research = None;
        self.research_state.research_progress = 0.0;
        self.research_state.parked_progress.clear();
        self.launch_confirmation_target = None;
        match scene {
            "mainmenu" => {
                self.phase = GamePhase::MainMenu;
                self.menu_notice = None;
                self.delete_confirmation_slot = None;
            }
            "desktop_mainmenu_short" => {
                self.phase = GamePhase::MainMenu;
                self.has_save = true;
                self.has_active_campaign = true;
                self.menu_notice = None;
                self.delete_confirmation_slot = None;
            }
            "mobile_mainmenu_narrow" => {
                self.phase = GamePhase::MainMenu;
                self.has_save = true;
                self.has_active_campaign = true;
                self.menu_notice = None;
                self.delete_confirmation_slot = None;
            }
            "save_failed_menu" | "mobile_save_failed_menu" => {
                self.phase = GamePhase::MainMenu;
                self.has_save = true;
                self.has_active_campaign = true;
                self.menu_notice =
                    Some("Save failed: progress is in memory. Tap SAVE to retry.".to_string());
                self.campaign.current_mut().save_failed = true;
                self.delete_confirmation_slot = None;
            }
            "mainmenu_confirm_delete" | "mobile_mainmenu_confirm_delete" => {
                self.phase = GamePhase::MainMenu;
                self.has_save = true;
                self.delete_confirmation_slot = Some(self.active_slot);
                self.menu_notice = Some(format!(
                    "Tap Delete Slot again to permanently clear {}.",
                    slot_name(self.active_slot)
                ));
            }
            "settings" => {
                self.phase = GamePhase::Settings;
                self.settings_tab = crate::screens::SettingsTab::Display;
            }
            "desktop_settings_short" => {
                self.phase = GamePhase::Settings;
                self.settings_tab = crate::screens::SettingsTab::Display;
            }
            "settings_audio" | "mobile_settings_audio" | "mobile_settings_audio_narrow" => {
                self.phase = GamePhase::Settings;
                self.settings_tab = crate::screens::SettingsTab::Audio;
            }
            "desktop_settings_audio_short" => {
                self.phase = GamePhase::Settings;
                self.settings_tab = crate::screens::SettingsTab::Audio;
            }
            "mobile_settings_narrow" => {
                self.phase = GamePhase::Settings;
                self.settings_tab = crate::screens::SettingsTab::Display;
            }
            "research" | "mobile_research" | "mobile_research_narrow" => {
                self.stage_research_scene()
            }
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
            "focus" | "focus_compact" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.focus_mode = true;
                planet.camera.zoom = 1.35;
            }
            "freight" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                let Some(core) = planet.grid.find_core() else {
                    return;
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
                    return;
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
            "core_ring" | "core_ring_focus" => {
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
            // The two beats of a launch worth looking at in a still frame: the
            // ship clearing the world it was built on, and the one it reaches.
            "launch" | "arrival" | "mobile_launch" | "mobile_arrival" => {
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                let mut sequence = LaunchSequence::new_with_data(self.data.clone(), 0, 1);
                sequence.advance(if matches!(scene, "launch" | "mobile_launch") {
                    sequence.beat_start(state::LaunchBeat::Ascent) + 1.2
                } else {
                    sequence.beat_start(state::LaunchBeat::Arrival) + 1.5
                });
                self.launch = Some(sequence);
                self.phase = GamePhase::Launch;
            }
            "mobile_arrival_notice" => self.stage_arrival_notice_scene(),
            "venus" => self.stage_venus_scene(),
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
                    return;
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
                    return;
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
                self.stage_factory_scene(scene)
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
            "geothermal" => self.stage_geothermal_scene(),
            "fusion" => self.stage_fusion_scene(),
            "demolish" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.toggle_demolish_mode();
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
            }
            "ending" => self.stage_ending_scene(),
            // The banner reports a shutdown that scales with the base, so it
            // has to be looked at on a base that has some size to it.
            "collapse" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.resources.data = 400.0;
                planet.trigger_power_collapse();
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
                    return;
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
            "camera" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                // Framed as if the player had zoomed in and dragged the map.
                let camera = &mut self.campaign.current_mut().camera;
                camera.zoom = 1.8;
                camera.pan_x = -420.0;
                camera.pan_y = -180.0;
            }
            "interplanetary" => self.stage_interplanetary_scene(),
            "desktop_interplanetary_short" => {
                self.stage_interplanetary_scene();
            }
            "interplanetary_launch_confirm" => {
                self.stage_interplanetary_scene();
                self.launch_confirmation_target = Some(0);
            }
            "interplanetary_driver_offline" | "mobile_driver_offline" => {
                self.stage_interplanetary_scene();
                let driver = self
                    .campaign
                    .current()
                    .grid
                    .find_buildings(engine::BuildingType::MassDriver)
                    .first()
                    .copied();
                if let Some(driver) = driver {
                    if let Some(building) = self
                        .campaign
                        .current_mut()
                        .grid
                        .get_mut(driver)
                        .and_then(|tile| tile.building.as_mut())
                    {
                        building.standby = true;
                    }
                }
                if scene == "mobile_driver_offline" {
                    self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Shipping;
                }
            }
            "interplanetary_pad_offline" => self.stage_interplanetary_pad_offline_scene(),
            "interplanetary_remote_orders" => self.stage_interplanetary_remote_orders_scene(),
            "mobile_remote_orders" => {
                self.stage_interplanetary_remote_orders_scene();
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Shipping;
            }
            "mobile_interplanetary_narrow" => self.stage_interplanetary_scene(),
            "mobile_shipping" => {
                self.stage_interplanetary_scene();
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Shipping;
            }
            "mobile_map" => {
                self.stage_interplanetary_scene();
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Worlds;
            }
            "interplanetary_rules" => {
                self.stage_interplanetary_scene();
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                self.campaign.current_mut().power_balance = -1.0;
            }
            "mobile_interplanetary_rules" => {
                self.stage_interplanetary_scene();
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Worlds;
            }
            "mobile_interplanetary_alert" => {
                self.stage_mobile_interplanetary_alert_scene();
            }
            "mobile_shipping_reserve" => {
                self.stage_interplanetary_scene();
                self.campaign.cycle_export_reserve_for(2);
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Shipping;
            }
            _ => {
                // Default: jump straight into gameplay on the starting planet.
                self.phase = GamePhase::Playing;
            }
        }
    }
}
