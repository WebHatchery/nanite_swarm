use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerStrings {
    pub title: String,
    pub subtitle: String,
    pub briefing_title: String,
    pub briefing_lines: Vec<String>,
    pub briefing_tip: String,
    pub command_menu: String,
    pub new_game: String,
    pub r#continue: String,
    pub load: String,
    pub save: String,
    pub settings: String,
    pub delete_slot: String,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub quit: String,
}

static STRINGS: OnceLock<PlayerStrings> = OnceLock::new();

pub fn player_strings() -> &'static PlayerStrings {
    STRINGS.get_or_init(|| {
        let json = macroquad_toolkit::data_loader::load_text_with_fallback_sync(
            "assets/strings.json",
            &[PathBuf::from("assets/strings.json")],
            macroquad_toolkit::include_json_str!("../../assets/strings.json"),
        )
        .expect("player strings data source");
        macroquad_toolkit::data_loader::load_embedded_json(&json)
            .expect("assets/strings.json is valid")
    })
}
