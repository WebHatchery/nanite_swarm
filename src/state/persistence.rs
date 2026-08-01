//! Save/load functionality

use super::PlanetState;
use macroquad::miniquad;
use macroquad_toolkit::persistence::{load_string_key, save_string_key};
use std::io;

const GAME_NAME: &str = "nanite_swarm";

fn unix_seconds_now() -> i64 {
    (miniquad::date::now() as i64).max(0)
}

/// Serialize game state to JSON string
pub fn save_to_json(state: &mut PlanetState) -> Result<String, serde_json::Error> {
    state.last_saved_unix = unix_seconds_now();
    serde_json::to_string_pretty(state)
}

/// Deserialize game state from JSON string
pub fn load_from_json(json: &str) -> Result<PlanetState, serde_json::Error> {
    let mut state: PlanetState = serde_json::from_str(json)?;
    state
        .achievements
        .sync_definitions(super::game_state::achievement_definitions());
    // The stat sheet is derived, not saved: rebuild it before the offline
    // catch-up runs, or the save loads with every research effect switched off.
    state.refresh_stats();
    let now = unix_seconds_now();
    if state.last_saved_unix > 0 && now > state.last_saved_unix {
        let offline_seconds = (now - state.last_saved_unix) as f32;
        state.apply_offline_progress(offline_seconds);
    }
    state.last_saved_unix = now;
    Ok(state)
}

pub fn save_to_file(state: &mut PlanetState, path: &str) -> Result<(), io::Error> {
    let json = save_to_json(state).map_err(io::Error::other)?;
    save_string_key(GAME_NAME, path, &json).map_err(io::Error::other)
}

pub fn load_from_file(path: &str) -> Result<PlanetState, io::Error> {
    let json = load_string_key(GAME_NAME, path).map_err(io::Error::other)?;
    load_from_json(&json).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::engine::{BuildingType, GridPos};

    #[test]
    fn json_roundtrip_preserves_grid_and_resources() {
        let mut state = PlanetState::new("Roundtrip", 8, 8, 7, GameConfig::default());
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);

        let json = save_to_json(&mut state).unwrap();
        let loaded = load_from_json(&json).unwrap();

        assert_eq!(loaded.name, "Roundtrip");
        assert_eq!(loaded.grid.width, state.grid.width);
        assert_eq!(loaded.resources.minerals, state.resources.minerals);
        assert!(loaded.grid.get(pos).unwrap().building.is_some());
        assert_eq!(loaded.drones.total_count(), 1);
    }

    #[test]
    fn save_to_json_stamps_last_saved_unix() {
        let mut state = PlanetState {
            last_saved_unix: 0,
            ..Default::default()
        };
        save_to_json(&mut state).unwrap();
        assert!(state.last_saved_unix > 0);
    }

    #[test]
    fn load_from_json_applies_offline_progress_for_past_save() {
        let now = unix_seconds_now();
        let state = PlanetState {
            last_saved_unix: now - 120,
            ..Default::default()
        };
        let json = serde_json::to_string(&state).unwrap();

        let loaded = load_from_json(&json).unwrap();
        assert!(loaded.last_offline_seconds > 0.0);
        assert_eq!(loaded.last_saved_unix, now);
    }

    #[test]
    fn load_from_json_skips_offline_progress_for_fresh_save() {
        let state = PlanetState {
            last_saved_unix: unix_seconds_now(),
            ..Default::default()
        };
        let json = serde_json::to_string(&state).unwrap();

        let loaded = load_from_json(&json).unwrap();
        assert_eq!(loaded.last_offline_seconds, 0.0);
    }
}
