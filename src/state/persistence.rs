//! Save/load functionality

use super::campaign::Campaign;
use super::PlanetState;
use macroquad::miniquad;
use macroquad_toolkit::persistence::{json_key_exists, load_string_key, save_string_key};
use serde::{Deserialize, Serialize};
use std::io;

/// The key namespace every save and the settings file live under.
pub const GAME_NAME: &str = "nanite_swarm";

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

/// Where saves are kept. The game uses the toolkit's key store; tests use a
/// map, so the rotation and recovery rules can be exercised without touching a
/// player's actual save.
pub trait SaveStore {
    fn read(&self, key: &str) -> Option<String>;
    fn write(&mut self, key: &str, content: &str) -> Result<(), String>;
}

struct KeyStore;

impl SaveStore for KeyStore {
    fn read(&self, key: &str) -> Option<String> {
        load_string_key(GAME_NAME, key).ok()
    }

    fn write(&mut self, key: &str, content: &str) -> Result<(), String> {
        save_string_key(GAME_NAME, key, content)
    }
}

/// Which copy a campaign came back from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadSource {
    Primary,
    Backup,
}

fn backup_key(key: &str) -> String {
    format!("{}_backup", key)
}

/// Write the campaign, keeping the previous save as a backup first.
///
/// Autosave overwrites the same slot every minute, so without this one bad
/// write - a bug, a half-full disk, a shape the loader cannot read - would take
/// the whole campaign with it. The backup is always the last save that was
/// good enough to have been written.
fn save_campaign(
    store: &mut dyn SaveStore,
    key: &str,
    campaign: &mut Campaign,
) -> Result<(), String> {
    let json = save_to_json(campaign).map_err(|error| error.to_string())?;
    if let Some(previous) = store.read(key) {
        // A failed rotation is not worth losing the new save over, but it does
        // mean the backup is stale, so it is not silently ignored either.
        store.write(&backup_key(key), &previous)?;
    }
    store.write(key, &json)
}

/// Read the campaign, falling back to the backup if the main save will not
/// parse. Says which one it came from so the player can be told.
fn load_campaign(store: &dyn SaveStore, key: &str) -> Result<(Campaign, LoadSource), String> {
    let primary_error = match store.read(key) {
        Some(json) => match load_from_json(&json) {
            Ok(campaign) => return Ok((campaign, LoadSource::Primary)),
            Err(error) => error.to_string(),
        },
        None => "no save found".to_string(),
    };

    match store.read(&backup_key(key)) {
        Some(json) => match load_from_json(&json) {
            Ok(campaign) => Ok((campaign, LoadSource::Backup)),
            Err(_) => Err(primary_error),
        },
        None => Err(primary_error),
    }
}

pub fn save_to_file(campaign: &mut Campaign, path: &str) -> Result<(), io::Error> {
    save_campaign(&mut KeyStore, path, campaign).map_err(io::Error::other)
}

pub fn load_from_file(path: &str) -> Result<(Campaign, LoadSource), io::Error> {
    load_campaign(&KeyStore, path).map_err(io::Error::other)
}

/// Whether either recoverable copy of a save exists.
pub fn save_exists(path: &str) -> bool {
    json_key_exists(GAME_NAME, path) || json_key_exists(GAME_NAME, &backup_key(path))
}

#[cfg(test)]
mod tests;
