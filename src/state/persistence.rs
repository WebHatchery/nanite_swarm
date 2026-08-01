//! Save/load functionality

use super::campaign::Campaign;
use super::PlanetState;
use macroquad::miniquad;
use macroquad_toolkit::persistence::{load_string_key, save_string_key};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::engine::{BuildingType, GridPos};

    fn campaign() -> Campaign {
        Campaign::new(GameConfig::default(), 7)
    }

    /// A save store that lives in memory, so the rotation rules can be tested
    /// without going near a real save.
    #[derive(Default)]
    struct MapStore {
        entries: std::collections::HashMap<String, String>,
        fail_writes_to: Option<String>,
    }

    impl SaveStore for MapStore {
        fn read(&self, key: &str) -> Option<String> {
            self.entries.get(key).cloned()
        }

        fn write(&mut self, key: &str, content: &str) -> Result<(), String> {
            if self.fail_writes_to.as_deref() == Some(key) {
                return Err("disk is on fire".to_string());
            }
            self.entries.insert(key.to_string(), content.to_string());
            Ok(())
        }
    }

    #[test]
    fn the_first_save_writes_no_backup_because_there_is_nothing_to_keep() {
        let mut store = MapStore::default();
        let mut campaign = campaign();
        save_campaign(&mut store, "save", &mut campaign).unwrap();

        assert!(store.read("save").is_some());
        assert!(store.read("save_backup").is_none());
    }

    #[test]
    fn saving_keeps_the_previous_save_as_the_backup() {
        let mut store = MapStore::default();
        let mut first = campaign();
        first.current_mut().resources.biomass = 11.0;
        save_campaign(&mut store, "save", &mut first).unwrap();

        let mut second = campaign();
        second.current_mut().resources.biomass = 22.0;
        save_campaign(&mut store, "save", &mut second).unwrap();

        let (current, source) = load_campaign(&store, "save").unwrap();
        assert_eq!(source, LoadSource::Primary);
        assert_eq!(current.current().resources.biomass, 22.0);

        let backup = load_from_json(&store.read("save_backup").unwrap()).unwrap();
        assert_eq!(backup.current().resources.biomass, 11.0);
    }

    #[test]
    fn a_corrupt_save_is_recovered_from_the_backup() {
        let mut store = MapStore::default();
        let mut good = campaign();
        good.current_mut().resources.biomass = 33.0;
        save_campaign(&mut store, "save", &mut good).unwrap();
        // Second save rotates the good one into the backup...
        let mut newer = campaign();
        save_campaign(&mut store, "save", &mut newer).unwrap();
        // ...and then the main save goes bad.
        store.write("save", "{ this is not a save }").unwrap();

        let (recovered, source) = load_campaign(&store, "save").unwrap();
        assert_eq!(source, LoadSource::Backup);
        assert_eq!(recovered.current().resources.biomass, 33.0);
    }

    #[test]
    fn a_missing_save_with_a_good_backup_still_loads() {
        let mut store = MapStore::default();
        let mut good = campaign();
        good.current_mut().resources.biomass = 44.0;
        save_campaign(&mut store, "save", &mut good).unwrap();
        let mut newer = campaign();
        save_campaign(&mut store, "save", &mut newer).unwrap();
        store.entries.remove("save");

        let (recovered, source) = load_campaign(&store, "save").unwrap();
        assert_eq!(source, LoadSource::Backup);
        assert_eq!(recovered.current().resources.biomass, 44.0);
    }

    #[test]
    fn two_bad_copies_is_an_error_rather_than_a_silent_new_game() {
        let mut store = MapStore::default();
        store.write("save", "rubbish").unwrap();
        store.write("save_backup", "also rubbish").unwrap();
        assert!(load_campaign(&store, "save").is_err());
    }

    #[test]
    fn a_failed_rotation_does_not_quietly_overwrite_the_good_save() {
        let mut store = MapStore::default();
        let mut first = campaign();
        first.current_mut().resources.biomass = 55.0;
        save_campaign(&mut store, "save", &mut first).unwrap();

        store.fail_writes_to = Some("save_backup".to_string());
        let mut second = campaign();
        second.current_mut().resources.biomass = 66.0;
        assert!(save_campaign(&mut store, "save", &mut second).is_err());

        // The old save is still there and still readable.
        let (kept, _) = load_campaign(&store, "save").unwrap();
        assert_eq!(kept.current().resources.biomass, 55.0);
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
