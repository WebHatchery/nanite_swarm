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

#[cfg(target_arch = "wasm32")]
use macroquad::prelude::*;
use std::fs;
use std::sync::OnceLock;

static GAME_DATA: OnceLock<GameData> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game_config() -> GameConfig {
    let json = fs::read_to_string("assets/game_config.json").unwrap_or_else(|_| {
        macroquad_toolkit::include_json_str!("../../assets/game_config.json").to_string()
    });
    load_json(&json).unwrap_or_else(|_| GameConfig::default())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_ui_theme() -> UiTheme {
    let json = fs::read_to_string("assets/ui_theme.json").unwrap_or_else(|_| {
        macroquad_toolkit::include_json_str!("../../assets/ui_theme.json").to_string()
    });
    load_json(&json).unwrap_or_else(|_| UiTheme::default())
}

#[cfg(target_arch = "wasm32")]
pub async fn load_game_config() -> GameConfig {
    let json = load_string("assets/game_config.json")
        .await
        .unwrap_or_else(|_| {
            macroquad_toolkit::include_json_str!("../../assets/game_config.json").to_string()
        });
    load_json(&json).unwrap_or_else(|_| GameConfig::default())
}

#[cfg(target_arch = "wasm32")]
pub async fn load_ui_theme() -> UiTheme {
    let json = load_string("assets/ui_theme.json")
        .await
        .unwrap_or_else(|_| {
            macroquad_toolkit::include_json_str!("../../assets/ui_theme.json").to_string()
        });
    load_json(&json).unwrap_or_else(|_| UiTheme::default())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game_data() -> GameData {
    GameData::load()
}

#[cfg(target_arch = "wasm32")]
pub async fn load_game_data() -> GameData {
    GameData::load_async().await
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
