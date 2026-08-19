//! Save schema migrations. Each supported version gets a named runner so a
//! future format change does not turn loading into an archaeology project.

use serde_json::{Map, Value};

use super::campaign::Campaign;
use super::game_state::PlanetState;
use super::persistence::SaveGame;

pub const SUPPORTED_SAVE_VERSIONS: &[u32] = &[0, 1, 2];

pub fn migrate(json: &str) -> Result<SaveGame, serde_json::Error> {
    let value: Value = serde_json::from_str(json)?;
    let Some(object) = value.as_object() else {
        return Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "save root is not an object",
        )));
    };
    let Some(version) = object.get("version").and_then(Value::as_u64) else {
        return migrate_v0(value);
    };
    let migrated = match version as u32 {
        1 => migrate_v1(value),
        2 => Ok(value),
        unsupported => Err(format!("unsupported save version {unsupported}")),
    }
    .map_err(|message| {
        serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message,
        ))
    })?;
    serde_json::from_value(migrated)
}

fn migrate_v0(value: Value) -> Result<SaveGame, serde_json::Error> {
    let planet: PlanetState = serde_json::from_value(value)?;
    Ok(SaveGame {
        version: super::persistence::SAVE_VERSION,
        campaign: Campaign::from_single_planet(planet, super::persistence::planet_seed_fallback()),
    })
}

fn migrate_v1(mut value: Value) -> Result<Value, String> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| "version 1 save root is not an object".to_string())?;
    let campaign = object
        .get_mut("campaign")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "version 1 save has no campaign object".to_string())?;
    insert_default(campaign, "slot_name", Value::String("slot_1".to_string()));
    insert_default(campaign, "directive_history", Value::Array(Vec::new()));
    insert_default(campaign, "toast_history", Value::Array(Vec::new()));
    insert_default(campaign, "shipments", Value::Array(Vec::new()));
    object.insert(
        "version".to_string(),
        Value::from(super::persistence::SAVE_VERSION),
    );
    Ok(value)
}

fn insert_default(object: &mut Map<String, Value>, key: &str, value: Value) {
    object.entry(key.to_string()).or_insert(value);
}
