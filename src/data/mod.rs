//! Data types and JSON loading
//!
//! This module contains all data structures and configuration loading.

mod defs;
mod game_config;
mod strings;
mod ui_theme;
mod validation;

pub use defs::*;
pub use game_config::*;
pub use strings::*;
pub use ui_theme::*;

pub(crate) use validation::validate_game_data;

use std::path::PathBuf;
use std::sync::Arc;

/// Immutable game definitions shared by one running session.
///
/// The context is deliberately owned by the session and cloned into durable
/// state as an `Arc`; it is never serialized. This lets tests and future
/// simultaneous sessions supply different data without a process-wide cache.
#[derive(Debug, Clone)]
pub struct GameDataContext(Arc<GameData>);

impl GameDataContext {
    pub fn new(data: GameData) -> Self {
        Self(Arc::new(data))
    }

    pub fn data(&self) -> &GameData {
        &self.0
    }
}

impl Default for GameDataContext {
    fn default() -> Self {
        Self::new(GameData::load())
    }
}

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

#[cfg(test)]
pub(crate) fn test_data() -> GameDataContext {
    GameDataContext::default()
}
