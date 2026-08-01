//! Save/load functionality

use super::campaign::Campaign;
use super::PlanetState;
use macroquad::miniquad;
use macroquad_toolkit::persistence::{load_string_key, save_string_key};
use serde::{Deserialize, Serialize};
use std::io;

const GAME_NAME: &str = "nanite_swarm";

/// Bumped whenever the shape below changes. Version 0 is the unversioned save
/// that held a single bare `PlanetState`.
const SAVE_VERSION: u32 = 1;

/// What actually goes on disk.
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveGame {
    pub version: u32,
    pub campaign: Campaign,
}

fn unix_seconds_now() -> i64 {
    (miniquad::date::now() as i64).max(0)
}

/// Serialize the campaign to a JSON string
pub fn save_to_json(campaign: &mut Campaign) -> Result<String, serde_json::Error> {
    campaign.current_mut().last_saved_unix = unix_seconds_now();
    let save = SaveGame {
        version: SAVE_VERSION,
        campaign: campaign.clone(),
    };
    serde_json::to_string_pretty(&save)
}

/// Deserialize a campaign, accepting saves written before the campaign existed.
pub fn load_from_json(json: &str) -> Result<Campaign, serde_json::Error> {
    let mut campaign = match serde_json::from_str::<SaveGame>(json) {
        Ok(save) => save.campaign,
        Err(envelope_error) => {
            // Version 0: a single planet, no campaign around it.
            match serde_json::from_str::<PlanetState>(json) {
                Ok(planet) => Campaign::from_single_planet(planet, planet_seed_fallback()),
                Err(_) => return Err(envelope_error),
            }
        }
    };

    let planet = campaign.current_mut();
    planet
        .achievements
        .sync_definitions(super::game_state::achievement_definitions());
    // The stat sheet is derived, not saved: rebuild it before the offline
    // catch-up runs, or the save loads with every research effect switched off.
    planet.refresh_stats();

    let now = unix_seconds_now();
    if planet.last_saved_unix > 0 && now > planet.last_saved_unix {
        let offline_seconds = (now - planet.last_saved_unix) as f32;
        planet.apply_offline_progress(offline_seconds);
    }
    planet.last_saved_unix = now;

    Ok(campaign)
}

/// A migrated save has no campaign seed of its own; give it a stable one so
/// the worlds it goes on to colonize are at least reproducible from here.
fn planet_seed_fallback() -> u64 {
    0x5EED_0000_0000_0001
}

pub fn save_to_file(campaign: &mut Campaign, path: &str) -> Result<(), io::Error> {
    let json = save_to_json(campaign).map_err(io::Error::other)?;
    save_string_key(GAME_NAME, path, &json).map_err(io::Error::other)
}

pub fn load_from_file(path: &str) -> Result<Campaign, io::Error> {
    let json = load_string_key(GAME_NAME, path).map_err(io::Error::other)?;
    load_from_json(&json).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::engine::{BuildingType, GridPos};

    fn campaign() -> Campaign {
        Campaign::new(GameConfig::default(), 7)
    }

    #[test]
    fn json_roundtrip_preserves_grid_and_resources() {
        let mut campaign = campaign();
        let core = campaign.current().grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        campaign.current_mut().grid.reveal_around(pos, 1);
        campaign.current_mut().select_building(BuildingType::Drill);
        campaign.current_mut().try_place_building(pos);

        let json = save_to_json(&mut campaign).unwrap();
        let loaded = load_from_json(&json).unwrap();

        assert_eq!(loaded.current().name, "Mars");
        assert_eq!(loaded.current().grid.width, campaign.current().grid.width);
        assert!(loaded.current().grid.get(pos).unwrap().building.is_some());
        assert_eq!(loaded.current().drones.total_count(), 1);
    }

    #[test]
    fn json_roundtrip_preserves_every_colonized_world() {
        let mut campaign = campaign();
        campaign.colonize(0);
        campaign.travel_to(0);
        campaign.current_mut().resources.biomass = 42.0;
        campaign.travel_to(super::super::campaign::STARTING_PLANET);

        let json = save_to_json(&mut campaign).unwrap();
        let mut loaded = load_from_json(&json).unwrap();

        assert_eq!(loaded.colonized_flags(), [true, false, true, false, false]);
        assert_eq!(
            loaded.current_index(),
            super::super::campaign::STARTING_PLANET
        );
        assert!(loaded.travel_to(0));
        assert_eq!(loaded.current().resources.biomass, 42.0);
    }

    #[test]
    fn a_save_stamps_the_current_planet_with_the_time() {
        let mut campaign = campaign();
        campaign.current_mut().last_saved_unix = 0;
        save_to_json(&mut campaign).unwrap();
        assert!(campaign.current().last_saved_unix > 0);
    }

    #[test]
    fn the_written_save_carries_the_current_schema_version() {
        let mut campaign = campaign();
        let json = save_to_json(&mut campaign).unwrap();
        let save: SaveGame = serde_json::from_str(&json).unwrap();
        assert_eq!(save.version, SAVE_VERSION);
    }

    #[test]
    fn an_unversioned_single_planet_save_still_loads() {
        // Exactly what version 0 wrote: a bare PlanetState.
        let planet = PlanetState::new(2, 3, GameConfig::default());
        let json = serde_json::to_string(&planet).unwrap();

        let loaded = load_from_json(&json).unwrap();
        assert_eq!(loaded.current().name, "Mars");
        assert_eq!(
            loaded.current_index(),
            super::super::campaign::STARTING_PLANET
        );
        assert_eq!(loaded.colonized_flags(), [false, false, true, false, false]);
    }

    #[test]
    fn nonsense_json_is_an_error_not_a_fresh_campaign() {
        assert!(load_from_json("{\"not\": \"a save\"}").is_err());
    }

    #[test]
    fn load_from_json_applies_offline_progress_for_past_save() {
        let mut campaign = campaign();
        campaign.current_mut().last_saved_unix = unix_seconds_now() - 120;
        let save = SaveGame {
            version: SAVE_VERSION,
            campaign: campaign.clone(),
        };
        let json = serde_json::to_string(&save).unwrap();

        let loaded = load_from_json(&json).unwrap();
        assert!(loaded.current().last_offline_seconds > 0.0);
    }

    #[test]
    fn load_from_json_skips_offline_progress_for_fresh_save() {
        let mut campaign = campaign();
        let json = save_to_json(&mut campaign).unwrap();

        let loaded = load_from_json(&json).unwrap();
        assert_eq!(loaded.current().last_offline_seconds, 0.0);
    }
}
