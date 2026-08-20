//! Nanite Swarm - A self-replicating AI simulation.
//!
//! The binary entrypoint owns application construction and the platform loop.
//! Frame orchestration lives in [`update`] so this file stays focused on
//! wiring the game's top-level dependencies together.

use macroquad::prelude::*;
use macroquad_toolkit::capture;
use macroquad_toolkit::debug::DebugOverlay;
use macroquad_toolkit::settings::GameSettings;

mod assets;
mod capture_scenes;
mod data;
mod directives;
mod display;
mod engine;
pub mod release;
mod research_runtime;
mod screens;
mod state;
mod ui;
#[path = "main/update.rs"]
mod update;

use assets::GameTextures;
use data::{load_game_config, load_game_data, load_ui_theme, set_game_data};
use engine::{ResearchState, ResearchTree};
use state::{Campaign, LaunchSequence, GAME_NAME};

/// Game phases/screens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamePhase {
    MainMenu,
    Playing,
    Research,
    SeedShip,
    Interplanetary,
    Records,
    Settings,
    /// A ship is on its way. The world is not being simulated while it plays.
    Launch,
    /// The system is spent. Reachable once and then only by choice.
    CampaignComplete,
}

/// Application coordinator and durable session state.
pub struct Game {
    phase: GamePhase,
    campaign: Campaign,
    research_tree: ResearchTree,
    research_state: ResearchState,
    settings: GameSettings,
    audio_mix: state::AudioMix,
    debug_overlay: DebugOverlay,
    has_save: bool,
    menu_notice: Option<String>,
    /// The ending has been shown once; seeing it again is the player's choice.
    ending_seen: bool,
    /// The launch being played out, if one is.
    launch: Option<LaunchSequence>,
    /// Staging a still frame for the screenshot harness.
    capture_still: bool,
    textures: GameTextures,
    config: data::GameConfig,
    ui_theme: data::UiTheme,
    research_viewport: screens::ResearchViewport,
    shipping_edit_world: usize,
    active_slot: usize,
}

const SAVE_PATH: &str = "save.json";
const SLOT_NAMES: [&str; 3] = ["slot_1", "slot_2", "slot_3"];

fn slot_name(index: usize) -> &'static str {
    SLOT_NAMES[index % SLOT_NAMES.len()]
}

fn save_path(index: usize) -> String {
    format!("{}_{}", SAVE_PATH, slot_name(index))
}

impl Game {
    pub async fn new() -> Self {
        // Rajdhani's dynamic glyph atlas can resize several times on the first
        // text-heavy gameplay frame. WebGL then tries to flush batches that
        // still refer to the deleted atlas textures. Macroquad's built-in font
        // has a stable, prebuilt atlas and avoids that first-frame corruption.
        macroquad_toolkit::ui::use_macroquad_default_ui_font();
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

        let mut settings = GameSettings::load(GAME_NAME);
        settings.sanitize();
        display::apply_display_settings(&settings, None);

        let mut game = Self {
            phase: GamePhase::MainMenu,
            campaign: Campaign::new(config.clone(), 42),
            research_tree: ResearchTree::default(),
            research_state: ResearchState::default(),
            settings,
            audio_mix: state::AudioMix::default(),
            debug_overlay: DebugOverlay::new(),
            has_save: state::save_exists(&save_path(0)),
            menu_notice: None,
            ending_seen: false,
            launch: None,
            capture_still: false,
            textures: GameTextures::load().await,
            config,
            ui_theme,
            research_viewport: screens::ResearchViewport::default(),
            shipping_edit_world: state::STARTING_PLANET,
            active_slot: 0,
        };
        game.campaign
            .apply_preferred_speed(game.settings.default_speed);
        game
    }
}

fn window_conf() -> Conf {
    capture::capture_window_conf("NANITE_SWARM", "Nanite Swarm", 1280, 720)
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    // Screenshot harness: when NANITE_SWARM_CAPTURE_PATH is set, seed a scene,
    // simulate deterministic frames, write a PNG, and exit.
    if let Some(configs) = capture::CaptureConfig::all_from_env("NANITE_SWARM") {
        for config in configs {
            game.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |_dt| {
                game.update();
                macroquad_toolkit::ui::end_frame_neighbours();
            })
            .await;
        }
        return;
    }

    loop {
        game.update();
        macroquad_toolkit::ui::end_frame_neighbours();
        next_frame().await;
    }
}
