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
    render_interplanetary_view, render_main_menu, render_planetary_view, render_research_view,
    render_seed_ship_view, render_settings_menu, InterplanetaryAction, MenuAction, PlanetaryAction,
    ResearchAction, SeedShipAction, SettingsAction,
};
use state::{load_from_file, save_to_file, Campaign};

/// Game phases/screens
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamePhase {
    MainMenu,
    Playing,
    Research,
    SeedShip,
    Interplanetary,
    Settings,
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
        Self {
            phase: GamePhase::MainMenu,
            campaign: Campaign::new(config.clone(), 42),
            research_tree: ResearchTree::default(),
            research_state: ResearchState::default(),
            settings: GameSettings {
                music_volume: 0.6,
                sfx_volume: 0.7,
                ui_text_scale: 1.0,
                ..GameSettings::default()
            },
            debug_overlay: DebugOverlay::new(),
            has_save: false,
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
                    if let Ok(campaign) = load_from_file(SAVE_PATH) {
                        self.campaign = campaign;
                        self.sync_research_from_planet();
                        self.phase = GamePhase::Playing;
                        self.has_save = true;
                        self.sync_building_unlocks();
                    }
                }
                MenuAction::Save => {
                    let _ = save_to_file(&mut self.campaign, SAVE_PATH);
                    self.has_save = true;
                }
                MenuAction::Settings => {
                    self.phase = GamePhase::Settings;
                }
                MenuAction::Quit => {}
                MenuAction::None => {}
            },
            GamePhase::Settings => match render_settings_menu(&mut self.settings) {
                SettingsAction::Back => {
                    self.phase = GamePhase::MainMenu;
                }
                SettingsAction::None => {}
            },
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
                        self.phase = GamePhase::MainMenu;
                        self.has_save = true;
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
            GamePhase::Interplanetary => {
                match render_interplanetary_view(
                    self.campaign.current_index(),
                    self.has_mass_driver(),
                    self.campaign.current().seed_ship.is_ready_to_launch(),
                    &self.campaign.colonized_flags(),
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
                            self.phase = GamePhase::Playing;
                        }
                    }
                    InterplanetaryAction::LaunchSeedShip(index) => {
                        // The ship is spent carrying the swarm to a new world.
                        if self.has_mass_driver() && self.campaign.launch_seed_ship(index) {
                            self.sync_research_to_planet();
                            self.sync_building_unlocks();
                            self.phase = GamePhase::Playing;
                        }
                    }
                    InterplanetaryAction::None => {}
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
            self.research_state.complete_research();
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
            self.research_state.complete_research();
        }

        self.sync_building_unlocks();
        self.sync_research_to_planet();
    }

    fn sync_research_from_planet(&mut self) {
        self.research_state.unlocked = self.campaign.current().research.unlocked_techs.clone();
        for tech in &data::game_data().research.starting_unlocked {
            if !self.research_state.unlocked.contains(tech) {
                self.research_state.unlocked.push(tech.clone());
            }
        }
        let planet = self.campaign.current();
        self.research_state.current_research = planet.research.current_research.clone();
        self.research_state.research_progress = planet.research.research_progress;
    }

    fn sync_research_to_planet(&mut self) {
        let unlocked = self.research_state.unlocked.clone();
        let current_research = self.research_state.current_research.clone();
        let progress = self.research_state.research_progress;
        let planet = self.campaign.current_mut();
        planet.research.unlocked_techs = unlocked;
        planet.research.current_research = current_research;
        planet.research.research_progress = progress;
        // Unlocked techs just changed shape: rebuild what they do.
        planet.refresh_stats();
    }

    fn sync_building_unlocks(&mut self) {
        for def in &data::game_data().buildings {
            let Some(building_type) = engine::BuildingType::from_id(&def.id) else {
                continue;
            };
            let unlocked = def.start_unlocked
                || def
                    .unlocked_by
                    .as_deref()
                    .map(|tech| self.research_state.is_unlocked(tech))
                    .unwrap_or(false);
            if unlocked {
                self.campaign.current_mut().unlock_building(building_type);
            }
        }
    }

    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
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
                            drone.dispatch_to_core(core, route.clone(), 5.0);
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
