//! Save schema migrations. Each supported version gets a named runner so a
//! future format change does not turn loading into an archaeology project.

use serde_json::{Map, Value};

use super::campaign::Campaign;
use super::game_state::PlanetState;
use super::persistence::SaveGame;

pub const SUPPORTED_SAVE_VERSIONS: &[u32] = &[0, 1, 2, 3, 4, 5];

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
    // Keep the parsed value wide until matching. Casting a future or corrupt
    // u64 version to u32 could wrap it back onto a supported schema number.
    let migrated = match version {
        1 => migrate_v1(value),
        2 => migrate_v2(value),
        3 => migrate_v3(value),
        4 => migrate_v4(value),
        5 => Ok(value),
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
    insert_default(campaign, "ending_seen", Value::Bool(false));
    object.insert(
        "version".to_string(),
        Value::from(super::persistence::SAVE_VERSION),
    );
    Ok(value)
}

fn migrate_v2(mut value: Value) -> Result<Value, String> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| "version 2 save root is not an object".to_string())?;
    let campaign = object
        .get_mut("campaign")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "version 2 save has no campaign object".to_string())?;
    insert_default(campaign, "ending_seen", Value::Bool(false));
    object.insert(
        "version".to_string(),
        Value::from(super::persistence::SAVE_VERSION),
    );
    Ok(value)
}

fn migrate_v3(mut value: Value) -> Result<Value, String> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| "version 3 save root is not an object".to_string())?;
    // `parked_progress` is optional on the new research structs, so old
    // campaigns need no synthetic entries; bump the envelope so a later save
    // records the new schema explicitly.
    object.insert(
        "version".to_string(),
        Value::from(super::persistence::SAVE_VERSION),
    );
    Ok(value)
}

fn migrate_v4(mut value: Value) -> Result<Value, String> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| "version 4 save root is not an object".to_string())?;
    // Resources gained a durable lifetime-consumption ledger. Its serde
    // default keeps the old live pools intact while recording future sinks.
    object.insert(
        "version".to_string(),
        Value::from(super::persistence::SAVE_VERSION),
    );
    Ok(value)
}

fn insert_default(object: &mut Map<String, Value>, key: &str, value: Value) {
    object.entry(key.to_string()).or_insert(value);
}
