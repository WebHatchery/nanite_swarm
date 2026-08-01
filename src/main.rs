//! Nanite Swarm - A self-replicating AI simulation
//!
//! Entry point, game loop, and phase transitions.

#![allow(clippy::too_many_arguments, clippy::wrong_self_convention)]

use macroquad::prelude::*;
use macroquad_toolkit::capture;
use macroquad_toolkit::debug::DebugOverlay;
use macroquad_toolkit::settings::GameSettings;

mod assets;
mod data;
mod directives;
mod engine;
mod screens;
mod state;
mod ui;

use assets::GameTextures;
use data::{load_game_config, load_game_data, load_ui_theme, set_game_data};
use engine::{ResearchState, ResearchTree};
use screens::{
    render_campaign_complete_view, render_interplanetary_view, render_launch_view,
    render_main_menu, render_planetary_view, render_research_view, render_seed_ship_view,
    render_settings_menu, CampaignCompleteAction, InterplanetaryAction, LaunchAction, MenuAction,
    PlanetaryAction, ResearchAction, SeedShipAction, SettingsAction,
};
use state::{load_from_file, save_to_file, Campaign, LaunchSequence, GAME_NAME};

/// Game phases/screens
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamePhase {
    MainMenu,
    Playing,
    Research,
    SeedShip,
    Interplanetary,
    Settings,
    /// A ship is on its way. The world is not being simulated while it plays.
    Launch,
    /// The system is spent. Reachable once and then only by choice.
    CampaignComplete,
}

/// Main game state container
pub struct Game {
    phase: GamePhase,
    campaign: Campaign,
    research_tree: ResearchTree,
    research_state: ResearchState,
    settings: GameSettings,
    debug_overlay: DebugOverlay,
    has_save: bool,
    /// The ending has been shown once; seeing it again is the player's choice.
    ending_seen: bool,
    /// The launch being played out, if one is.
    launch: Option<LaunchSequence>,
    /// Staging a still frame for the screenshot harness. Anything that runs on
    /// real time has to hold where the scene put it, or the frame that gets
    /// written is wherever the capture's frame rate happened to carry it.
    capture_still: bool,
    textures: GameTextures,
    config: data::GameConfig,
    ui_theme: data::UiTheme,
}

const SAVE_PATH: &str = "save.json";
const RESEARCH_RATE: f32 = 5.0; // data per second

impl Game {
    pub async fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let config = load_game_config();
        #[cfg(target_arch = "wasm32")]
        let config = load_game_config().await;

        #[cfg(not(target_arch = "wasm32"))]
        let ui_theme = load_ui_theme();
        #[cfg(target_arch = "wasm32")]
        let ui_theme = load_ui_theme().await;

        #[cfg(not(target_arch = "wasm32"))]
        let game_data = load_game_data();
        #[cfg(target_arch = "wasm32")]
        let game_data = load_game_data().await;

        set_game_data(game_data);

        // Settings survive between sessions and are applied before the first
        // frame, so the player's text scale and fullscreen choice are already
        // in effect rather than snapping in when they visit the menu.
        let mut settings = GameSettings::load(GAME_NAME);
        settings.sanitize();
        apply_display_settings(&settings, None);

        Self {
            phase: GamePhase::MainMenu,
            campaign: Campaign::new(config.clone(), 42),
            research_tree: ResearchTree::default(),
            research_state: ResearchState::default(),
            settings,
            debug_overlay: DebugOverlay::new(),
            has_save: false,
            ending_seen: false,
            launch: None,
            capture_still: false,
            textures: GameTextures::load().await,
            config,
            ui_theme,
        }
    }

    /// Check if mass driver technology is researched
    fn has_mass_driver(&self) -> bool {
        self.research_state.is_unlocked("mass_driver")
    }

    pub fn update(&mut self) {
        self.debug_overlay.record_frame(get_frame_time());
        self.debug_overlay.visible = self.settings.show_fps;

        match self.phase {
            GamePhase::MainMenu => match render_main_menu(self.has_save) {
                MenuAction::NewGame => {
                    self.campaign =
                        Campaign::new(self.config.clone(), macroquad_toolkit::rng::random_u64());
                    self.research_state = ResearchState::default();
                    self.sync_research_to_planet();
                    self.sync_building_unlocks();
                    self.phase = GamePhase::Playing;
                }
                MenuAction::Continue => {
                    self.phase = GamePhase::Playing;
                }
                MenuAction::Load => {
                    if let Ok((campaign, source)) = load_from_file(SAVE_PATH) {
                        self.campaign = campaign;
                        // A save written before research was campaign-wide
                        // keeps it on the planet; take it from there once.
                        self.campaign.adopt_planet_research();
                        self.sync_research_from_planet();
                        self.phase = GamePhase::Playing;
                        self.has_save = true;
                        self.sync_building_unlocks();
                        // The player is owed the truth about which copy this is.
                        self.campaign.current_mut().restored_from_backup =
                            source == state::LoadSource::Backup;
                    }
                }
                MenuAction::Save => {
                    self.save_campaign();
                }
                MenuAction::Settings => {
                    self.phase = GamePhase::Settings;
                }
                MenuAction::Quit => {}
                MenuAction::None => {}
            },
            GamePhase::Settings => {
                let before = self.settings.clone();
                let action = render_settings_menu(&mut self.settings);
                if self.settings != before {
                    // Applied as it changes so the player can see what a text
                    // scale actually does, and written down so it is still
                    // true next session.
                    self.settings.sanitize();
                    apply_display_settings(&self.settings, Some(&before));
                    let _ = self.settings.save(GAME_NAME);
                }
                if action == SettingsAction::Back {
                    self.phase = GamePhase::MainMenu;
                }
            }
            GamePhase::Playing => {
                self.advance_simulation();

                let (planet, directive) = self.campaign.current_and_directive();
                match render_planetary_view(planet, &self.textures, directive, &self.ui_theme) {
                    PlanetaryAction::OpenResearch => {
                        self.phase = GamePhase::Research;
                    }
                    PlanetaryAction::OpenSeedShip => {
                        self.phase = GamePhase::SeedShip;
                    }
                    PlanetaryAction::OpenInterplanetary => {
                        self.phase = GamePhase::Interplanetary;
                    }
                    PlanetaryAction::OpenMenu => {
                        // Leaving the world is a good moment to write it down,
                        // and the menu used to only *claim* a save existed.
                        self.save_campaign();
                        self.phase = GamePhase::MainMenu;
                    }
                    PlanetaryAction::None => {}
                }
            }
            GamePhase::Research => {
                self.advance_simulation();
                match render_research_view(
                    &self.research_state,
                    &self.research_tree,
                    self.campaign.current().resources.data,
                    self.campaign.current().research_lock_timer > 0.0,
                ) {
                    ResearchAction::Close => {
                        self.phase = GamePhase::Playing;
                    }
                    ResearchAction::StartResearch(tech_id) => {
                        let _ = self.research_state.start_research(
                            &tech_id,
                            &self.research_tree,
                            self.campaign.current().resources.data,
                        );
                    }
                    ResearchAction::None => {}
                }
            }
            GamePhase::SeedShip => {
                self.advance_simulation();
                match render_seed_ship_view(self.campaign.current()) {
                    SeedShipAction::Close => {
                        self.phase = GamePhase::Playing;
                    }
                    SeedShipAction::ToggleCommitment => {
                        self.campaign.current_mut().toggle_seed_ship_commitment();
                    }
                    SeedShipAction::None => {}
                }
            }
            GamePhase::CampaignComplete => match render_campaign_complete_view(&self.campaign) {
                CampaignCompleteAction::KeepGoing => {
                    self.phase = GamePhase::Playing;
                }
                CampaignCompleteAction::Close => {
                    self.save_campaign();
                    self.phase = GamePhase::MainMenu;
                }
                CampaignCompleteAction::None => {}
            },
            GamePhase::Interplanetary => {
                let stockpiles: [Option<f32>; state::PLANET_COUNT] =
                    std::array::from_fn(|index| self.campaign.stockpile(index));
                match render_interplanetary_view(
                    self.campaign.current_index(),
                    self.has_mass_driver(),
                    self.campaign.current().seed_ship.is_ready_to_launch(),
                    &self.campaign.colonized_flags(),
                    &stockpiles,
                ) {
                    InterplanetaryAction::Close => {
                        self.phase = GamePhase::Playing;
                    }
                    InterplanetaryAction::SelectPlanet(index) => {
                        // The world being left keeps everything it had.
                        if self.campaign.travel_to(index) {
                            // The arriving world needs the campaign's research.
                            self.sync_research_to_planet();
                            self.sync_building_unlocks();
                            self.save_campaign();
                            self.phase = GamePhase::Playing;
                        }
                    }
                    InterplanetaryAction::LaunchSeedShip(index) => {
                        // The ship is spent carrying the swarm to a new world.
                        let origin = self.campaign.current_index();
                        if self.has_mass_driver() && self.campaign.launch_seed_ship(index) {
                            self.sync_research_to_planet();
                            self.sync_building_unlocks();
                            self.save_campaign();
                            // The campaign has already moved; what follows is
                            // only the telling of it. The vignette delivers the
                            // arrival line itself, so the top-bar notice that
                            // free travel uses would only repeat it.
                            self.campaign.current_mut().arrival_notice_timer = 0.0;
                            self.launch = Some(LaunchSequence::new(origin, index));
                            self.phase = GamePhase::Launch;
                        }
                    }
                    InterplanetaryAction::None => {}
                }
            }
            GamePhase::Launch => {
                let arrival_line = self.campaign.current().arrival_line();
                let Some(sequence) = self.launch.as_mut() else {
                    self.phase = GamePhase::Playing;
                    return;
                };
                if !self.capture_still {
                    sequence.advance(get_frame_time());
                }
                let action = render_launch_view(sequence, arrival_line);
                if action == LaunchAction::Skip {
                    sequence.skip();
                }
                if sequence.is_finished() {
                    self.launch = None;
                    self.phase = GamePhase::Playing;
                }
            }
        }

        self.debug_overlay.draw(&[]);
    }

    /// Advance the world by whole simulation ticks. Research and directives run
    /// on exactly the time the planet simulated, so nothing drifts apart when
    /// the frame rate moves or a catch-up backlog is dropped.
    fn advance_simulation(&mut self) {
        let ticks = self.campaign.current_mut().advance(get_frame_time(), true);
        if ticks == 0 {
            return;
        }
        let simulated = ticks as f32 * state::TICK_SECONDS;
        self.update_research(simulated);
        self.campaign.update_directive(simulated);
        // The worlds nobody is looking at keep working.
        self.campaign.update_background(simulated);
        self.check_campaign_complete();
        if self
            .campaign
            .due_for_autosave(self.settings.autosave_interval)
        {
            self.save_campaign();
        }
    }

    /// Show the ending the first time the system runs out of worlds.
    ///
    /// Only once: after that the player can go back to it from the map, but
    /// the game does not keep interrupting a finished campaign they have
    /// chosen to carry on playing.
    fn check_campaign_complete(&mut self) {
        if self.ending_seen || !self.campaign.is_complete() {
            return;
        }
        self.ending_seen = true;
        self.campaign
            .current_mut()
            .announce_achievement("system_consumed");
        self.save_campaign();
        self.phase = GamePhase::CampaignComplete;
    }

    /// Write the campaign down, and say so on screen either way.
    ///
    /// A silent failed save is worse than no autosave at all: the player would
    /// carry on believing their world was safe.
    fn save_campaign(&mut self) {
        match save_to_file(&mut self.campaign, SAVE_PATH) {
            Ok(()) => {
                self.has_save = true;
                self.campaign.mark_saved();
                self.campaign.current_mut().save_failed = false;
            }
            Err(_) => {
                self.campaign.current_mut().save_failed = true;
            }
        }
    }

    fn update_research(&mut self, delta_time: f32) {
        let Some(current_id) = self.research_state.current_research.clone() else {
            self.sync_building_unlocks();
            self.sync_research_to_planet();
            return;
        };
        if self.campaign.current().research_lock_timer > 0.0 {
            return;
        }
        let Some(node) = self.research_tree.get_node(&current_id) else {
            self.research_state.current_research = None;
            self.research_state.research_progress = 0.0;
            self.sync_research_to_planet();
            return;
        };

        let remaining = (node.data_cost - self.research_state.research_progress).max(0.0);
        if remaining <= 0.0 {
            let name = node.name.clone();
            self.research_state.complete_research();
            self.announce_research(&name);
            self.sync_building_unlocks();
            self.sync_research_to_planet();
            return;
        }

        let available = self.campaign.current().resources.data;
        if available <= 0.0 {
            return;
        }

        let rate = self
            .campaign
            .current()
            .stats
            .apply(engine::StatId::ResearchRate, RESEARCH_RATE);
        let spend = (rate * delta_time).min(available).min(remaining);
        self.campaign.current_mut().resources.data -= spend;
        self.research_state.research_progress += spend;

        if self.research_state.research_progress >= node.data_cost {
            let name = node.name.clone();
            self.research_state.complete_research();
            self.announce_research(&name);
        }

        self.sync_building_unlocks();
        self.sync_research_to_planet();
    }

    /// Finished research used to land with no more sign than a node changing
    /// colour on a screen the player was probably not looking at.
    fn announce_research(&mut self, name: &str) {
        self.campaign
            .current_mut()
            .notifications
            .success(format!("Research complete: {}", name));
    }

    /// Take the campaign's research into the research screen's state.
    fn sync_research_from_planet(&mut self) {
        self.research_state.unlocked = self.campaign.research.unlocked_techs.clone();
        for tech in &data::game_data().research.starting_unlocked {
            if !self.research_state.unlocked.contains(tech) {
                self.research_state.unlocked.push(tech.clone());
            }
        }
        self.research_state.current_research = self.campaign.research.current_research.clone();
        self.research_state.research_progress = self.campaign.research.research_progress;
    }

    /// Write the research screen's state back to the campaign, and push it at
    /// every world the campaign holds.
    fn sync_research_to_planet(&mut self) {
        self.campaign.research.unlocked_techs = self.research_state.unlocked.clone();
        self.campaign.research.current_research = self.research_state.current_research.clone();
        self.campaign.research.research_progress = self.research_state.research_progress;
        self.campaign.sync_research();
    }

    /// Announce anything that has just become available here.
    ///
    /// The unlocking itself is the campaign's job now, across every world;
    /// this only notices what changed on the world the player is looking at,
    /// because that is who the toast is for.
    fn sync_building_unlocks(&mut self) {
        let before: Vec<engine::BuildingType> = data::game_data()
            .buildings
            .iter()
            .filter_map(|def| engine::BuildingType::from_id(&def.id))
            .filter(|kind| self.campaign.current().is_building_researched(*kind))
            .collect();

        self.campaign.sync_research();

        for def in &data::game_data().buildings {
            let Some(building_type) = engine::BuildingType::from_id(&def.id) else {
                continue;
            };
            if def.start_unlocked || before.contains(&building_type) {
                continue;
            }
            let planet = self.campaign.current_mut();
            // Only the moment it opens up is worth a toast, not every frame it
            // stays open, and not the ones this world refuses anyway.
            if planet.is_building_researched(building_type)
                && !planet.is_building_banned(building_type)
            {
                let name = def.name.clone();
                planet.notifications.info(format!("Available: {}", name));
            }
        }
    }

    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.capture_still = true;
        match scene {
            "mainmenu" => self.phase = GamePhase::MainMenu,
            "research" => self.phase = GamePhase::Research,
            "logistics" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
            }
            "seedship" => {
                self.phase = GamePhase::SeedShip;
                self.seed_logistics_scene();
                self.campaign.current_mut().resources.alloy = 80.0;
                // Mid-build, with the swarm diverting production into the yard.
                let planet = self.campaign.current_mut();
                planet.config.resources.base_mineral_cap = 100_000.0;
                planet.resources.minerals = 400.0;
                planet.resources.data = 120.0;
                planet.toggle_seed_ship_commitment();
                for _ in 0..20 {
                    planet.update_seed_ship(1.0);
                }
            }
            // The two beats of a launch worth looking at in a still frame: the
            // ship clearing the world it was built on, and the one it reaches.
            "launch" | "arrival" => {
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                let mut sequence = LaunchSequence::new(0, 1);
                sequence.advance(if scene == "launch" {
                    LaunchSequence::beat_start(state::LaunchBeat::Ascent) + 1.2
                } else {
                    LaunchSequence::beat_start(state::LaunchBeat::Arrival) + 1.5
                });
                self.launch = Some(sequence);
                self.phase = GamePhase::Launch;
            }
            "venus" => {
                self.phase = GamePhase::Playing;
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                self.research_state
                    .unlocked
                    .push("ceramic_plating".to_string());
                self.research_state
                    .unlocked
                    .push("heater_nodes".to_string());
                let planet = self.campaign.current_mut();
                // Everything researched, so the palette shows what this world
                // refuses rather than what the swarm has not reached yet.
                for def in &data::game_data().buildings {
                    if let Some(building_type) = engine::BuildingType::from_id(&def.id) {
                        planet.unlock_building(building_type);
                    }
                }
                if let Some(core) = planet.grid.find_core() {
                    planet.grid.reveal_around(core, 12);
                }
            }
            "upkeep" => {
                self.phase = GamePhase::Playing;
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                self.research_state
                    .unlocked
                    .push("ceramic_plating".to_string());
                let planet = self.campaign.current_mut();
                planet.resources.minerals = 10_000.0;
                planet.resources.energy = 10_000.0;
                planet.config.resources.max_energy = 10_000.0;
                for def in &data::game_data().buildings {
                    if let Some(building_type) = engine::BuildingType::from_id(&def.id) {
                        planet.unlock_building(building_type);
                    }
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
            "ending" => {
                self.phase = GamePhase::CampaignComplete;
                // Play the campaign out the way it is actually played: every
                // world reached by building a ship and riding it there, so the
                // numbers on the ending screen are real.
                // The later stages are gated on research, so the scene has to
                // have done it, the same as a player would.
                for stage in &data::game_data().seed_ship.stages {
                    if let Some(tech) = stage.requires.as_deref() {
                        self.research_state.unlocked.push(tech.to_string());
                    }
                }
                self.sync_research_to_planet();

                let build_ship = |campaign: &mut Campaign| {
                    let planet = campaign.current_mut();
                    planet.config.resources.base_mineral_cap = 1_000_000.0;
                    planet.resources.minerals = 100_000.0;
                    planet.resources.data = 100_000.0;
                    planet.resources.biomass = 100_000.0;
                    planet.resources.alloy = 100_000.0;
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
                    let target =
                        (0..state::PLANET_COUNT).find(|index| !self.campaign.is_colonized(*index));
                    match target {
                        Some(index) => {
                            self.campaign.launch_seed_ship(index);
                        }
                        // Nowhere left: the last ship is the ending.
                        None => break,
                    }
                }
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
            "interplanetary" => {
                self.phase = GamePhase::Interplanetary;
                self.research_state.unlocked.push("mass_driver".to_string());
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
                planet.toggle_seed_ship_commitment();
                for _ in 0..2_000 {
                    planet.update_seed_ship(1.0);
                }
            }
            _ => {
                // Default: jump straight into gameplay on the starting planet.
                self.phase = GamePhase::Playing;
            }
        }
    }

    /// A working conduit run with a drill on the end of it, so drone routing
    /// can be eyeballed without playing up to it.
    fn seed_logistics_scene(&mut self) {
        use engine::{BuildingType, GridPos};

        let state = self.campaign.current_mut();
        let Some(core) = state.grid.find_core() else {
            return;
        };
        state.grid.reveal_around(core, 12);
        state.resources.minerals = 500.0;
        state.resources.energy = 500.0;
        state.config.resources.max_energy = 500.0;
        state.unlock_building(BuildingType::Conduit);
        state.unlock_building(BuildingType::PowerNode);

        // An L-shaped run: five tiles east, then four north, drill on the end.
        let mut run: Vec<GridPos> = (1..=5).map(|x| GridPos::new(core.x + x, core.y)).collect();
        run.extend((1..=4).map(|y| GridPos::new(core.x + 5, core.y - y)));

        for (index, pos) in run.iter().enumerate() {
            if let Some(tile) = state.grid.get_mut(*pos) {
                tile.terrain = engine::TerrainType::Empty;
                tile.building = None;
            }
            let piece = if index == 4 {
                BuildingType::PowerNode
            } else {
                BuildingType::Conduit
            };
            state.select_building(piece);
            state.try_place_building(*pos);
        }

        let drill = GridPos::new(core.x + 5, core.y - 5);
        if let Some(tile) = state.grid.get_mut(drill) {
            tile.terrain = engine::TerrainType::Empty;
            tile.building = None;
        }
        state.select_building(BuildingType::Drill);
        state.try_place_building(drill);
        state.grid.update_power_grid();
    }
}

/// Push display settings at the window, touching only what actually changed.
///
/// `GameSettings::apply_display` sets fullscreen unconditionally, and asking
/// miniquad for windowed mode when it is already windowed re-applies the window
/// style: on Windows that shrinks the client area, which cost the bottom bar
/// forty pixels the first time this was wired up.
fn apply_display_settings(settings: &GameSettings, previous: Option<&GameSettings>) {
    let scale_changed = previous.is_none_or(|old| old.ui_text_scale != settings.ui_text_scale);
    if scale_changed {
        macroquad_toolkit::ui::set_ui_text_scale(settings.ui_text_scale);
    }

    let fullscreen_changed = match previous {
        Some(old) => old.fullscreen != settings.fullscreen,
        // At startup there is nothing to undo, so only ask for fullscreen if
        // that is what the player wants.
        None => settings.fullscreen,
    };
    if fullscreen_changed {
        set_fullscreen(settings.fullscreen);
    }
}

fn window_conf() -> Conf {
    capture::capture_window_conf("NANITE_SWARM", "Nanite Swarm", 1280, 720)
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    // Screenshot harness: when NANITE_SWARM_CAPTURE_PATH is set, seed a scene,
    // simulate deterministic frames, write a PNG, and exit. Each render_*
    // screen function clears its own background, so there is nothing extra
    // to move into the closure.
    if let Some(config) = capture::CaptureConfig::from_env("NANITE_SWARM") {
        game.begin_capture_scene(&config.scene);
        capture::run_capture(&config, |_dt| {
            game.update();
        })
        .await;
        return;
    }

    loop {
        game.update();
        next_frame().await;
    }
}
