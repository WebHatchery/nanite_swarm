//! Data types and JSON loading
//!
//! This module contains all data structures and configuration loading.

#![allow(unused)]

mod defs;
mod game_config;
mod loader;
mod ui_theme;

pub use defs::*;
pub use game_config::*;
pub use loader::*;
pub use ui_theme::*;

use std::path::PathBuf;
use std::sync::OnceLock;

static GAME_DATA: OnceLock<GameData> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game_config() -> GameConfig {
    load_game_config_source()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_ui_theme() -> UiTheme {
    load_ui_theme_source()
}

#[cfg(target_arch = "wasm32")]
pub async fn load_game_config() -> GameConfig {
    load_game_config_source()
}

#[cfg(target_arch = "wasm32")]
pub async fn load_ui_theme() -> UiTheme {
    load_ui_theme_source()
}

fn load_game_config_source() -> GameConfig {
    macroquad_toolkit::data_loader::load_json_with_fallback_sync(
        "assets/game_config.json",
        &[PathBuf::from("assets/game_config.json")],
        macroquad_toolkit::include_json_str!("../../assets/game_config.json"),
    )
    .unwrap_or_else(|_| GameConfig::default())
}

fn load_ui_theme_source() -> UiTheme {
    macroquad_toolkit::data_loader::load_json_with_fallback_sync(
        "assets/ui_theme.json",
        &[PathBuf::from("assets/ui_theme.json")],
        macroquad_toolkit::include_json_str!("../../assets/ui_theme.json"),
    )
    .unwrap_or_else(|_| UiTheme::default())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game_data() -> GameData {
    GameData::load()
}

#[cfg(target_arch = "wasm32")]
pub async fn load_game_data() -> GameData {
    GameData::load()
}

pub fn set_game_data(data: GameData) {
    let _ = GAME_DATA.set(data);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn game_data() -> &'static GameData {
    GAME_DATA.get_or_init(GameData::load)
}

#[cfg(target_arch = "wasm32")]
pub fn game_data() -> &'static GameData {
    GAME_DATA.get().expect("Game data not loaded yet")
}
