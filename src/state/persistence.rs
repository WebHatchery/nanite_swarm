//! Save/load functionality

use super::campaign::Campaign;
use super::PlanetState;
use macroquad::miniquad;
use macroquad_toolkit::persistence::{
    delete_json_key, json_key_exists, load_string_key, save_string_key,
};
use serde::{Deserialize, Serialize};
use std::io;

/// The key namespace every save and the settings file live under.
pub const GAME_NAME: &str = "nanite_swarm";

/// Bumped whenever the shape below changes. Version 0 is the unversioned save
/// that held a single bare `PlanetState`.
pub(crate) const SAVE_VERSION: u32 = 5;

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
    campaign.stamp_saved_at(unix_seconds_now());
    let save = SaveGame {
        version: SAVE_VERSION,
        campaign: campaign.clone(),
    };
    serde_json::to_string_pretty(&save)
}

/// Deserialize a campaign, accepting saves written before the campaign existed.
pub fn load_from_json(json: &str) -> Result<Campaign, serde_json::Error> {
    let mut campaign = crate::state::migrations::migrate(json)?.campaign;
    campaign.validate_loaded_state().map_err(|message| {
        serde_json::Error::io(io::Error::new(io::ErrorKind::InvalidData, message))
    })?;
    campaign.normalize_tutorial_scope();

    // Older saves kept research only on each world. Restore that campaign
    // mirror before offline work is assigned to the active job.
    campaign.adopt_planet_research();
    // The build catalog is derived from the campaign research and each
    // world's bans. Rebuild it even when a current-format save already has a
    // non-empty campaign mirror, so stale or hand-edited unlock lists cannot
    // survive loading.
    campaign.sync_research();
    let now = unix_seconds_now();
    let mut shipment_elapsed: f32 = 0.0;
    let mut offline_worlds = Vec::new();
    let current_index = campaign.current_index();
    let mut foreground_simulated = 0.0;
    for (index, slot) in campaign.planets.iter_mut().enumerate() {
        let Some(planet) = slot.as_mut() else {
            continue;
        };
        planet
            .achievements
            .sync_definitions(super::game_state::achievement_definitions(planet.data()));
        // The stat sheet is derived, not saved: rebuild it before the
        // aggregated offline calculation runs.
        planet.refresh_stats();
        if planet.last_saved_unix > 0 {
            let delta = now - planet.last_saved_unix;
            let offline_seconds = if delta >= 0 {
                delta as f32
            } else {
                planet.config.offline.fallback_delta_seconds
            };
            shipment_elapsed = shipment_elapsed
                .max(offline_seconds.min(planet.config.offline.max_elapsed_seconds.max(0.0)));
            planet.apply_offline_progress(offline_seconds);
            planet.last_offline_report.tamper_guarded = delta < 0;
            offline_worlds.push(index);
            if index == current_index {
                foreground_simulated = planet.last_offline_simulated;
            }
        }
        planet.last_saved_unix = now;
    }

    // Move pods that were already queued or in flight when the save was
    // written. Offline exports are enqueued in the next pass so a pod made at
    // the end of this gap does not spend the whole gap travelling.
    if shipment_elapsed > 0.0 {
        campaign.update_shipments(shipment_elapsed);
    }
    for index in offline_worlds {
        let Some(planet) = campaign.planet_mut(index) else {
            continue;
        };
        let freight_pods = planet.apply_offline_export_progress();
        planet.last_offline_report.freight_pods_launched = freight_pods;
    }
    campaign.update_shipments(0.0);

    if foreground_simulated > 0.0 {
        let (research_spent, research_completed) =
            campaign.apply_offline_research(foreground_simulated);
        let stages_completed = campaign
            .current_mut()
            .apply_offline_seed_ship_progress(foreground_simulated);
        let directives_completed = campaign.apply_offline_directive_progress(foreground_simulated);
        let report = &mut campaign.current_mut().last_offline_report;
        report.research_data_spent = research_spent;
        report.research_completed = research_completed;
        report.seed_ship_stages_completed = stages_completed;
        report.directives_completed = directives_completed;

        // Research and Seed Ship stages are campaign-side offline work, so
        // refresh the foreground world's records after both have been applied.
        // This catches Core stages and achievements whose final requirement
        // was completed by the post-world catch-up passes above.
        let planet = campaign.current_mut();
        planet.update_achievements();
        planet.update_core_stage();
    }

    // Older saves kept histories beside each world. Fold those into the new
    // campaign stream during migration so Records does not appear empty.
    campaign.sync_notification_history();

    Ok(campaign)
}

/// A migrated save has no campaign seed of its own; give it a stable one so
/// the worlds it goes on to colonize are at least reproducible from here.
pub(super) fn planet_seed_fallback() -> u64 {
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

fn backup_key(key: &str, generation: usize) -> String {
    if generation == 1 {
        format!("{}_backup", key)
    } else {
        format!("{}_backup_{}", key, generation)
    }
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
        for generation in (2..=3).rev() {
            if let Some(older) = store.read(&backup_key(key, generation - 1)) {
                store.write(&backup_key(key, generation), &older)?;
            }
        }
        store.write(&backup_key(key, 1), &previous)?;
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

    for generation in 1..=3 {
        let Some(json) = store.read(&backup_key(key, generation)) else {
            continue;
        };
        if let Ok(mut campaign) = load_from_json(&json) {
            campaign.current_mut().restored_from_backup_generation = generation as u8;
            return Ok((campaign, LoadSource::Backup));
        }
    }
    Err(primary_error)
}

pub fn save_to_file(campaign: &mut Campaign, path: &str) -> Result<(), io::Error> {
    save_campaign(&mut KeyStore, path, campaign).map_err(io::Error::other)
}

pub fn load_from_file(path: &str) -> Result<(Campaign, LoadSource), io::Error> {
    load_campaign(&KeyStore, path).map_err(io::Error::other)
}

/// Whether either recoverable copy of a save exists.
pub fn save_exists(path: &str) -> bool {
    json_key_exists(GAME_NAME, path)
        || (1..=3).any(|generation| json_key_exists(GAME_NAME, &backup_key(path, generation)))
}

/// Delete one visible campaign slot, including all recovery generations.
pub fn delete_save(path: &str) -> Result<(), String> {
    delete_json_key(GAME_NAME, path)?;
    for generation in 1..=3 {
        delete_json_key(GAME_NAME, &backup_key(path, generation))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
