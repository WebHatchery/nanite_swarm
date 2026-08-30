//! JSON support for state maps keyed by grid coordinates.

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

/// JSON object keys have to be strings, while the simulation naturally uses a
/// coordinate tuple. Keep the on-disk form compact and deterministic without
/// changing the in-memory map used by the hot simulation paths.
pub(crate) fn serialize<S, V>(
    map: &HashMap<(i32, i32), V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Serialize,
{
    let entries: BTreeMap<String, &V> = map
        .iter()
        .map(|(&(x, y), value)| (format!("{x},{y}"), value))
        .collect();
    entries.serialize(serializer)
}

pub(crate) fn deserialize<'de, D, V>(deserializer: D) -> Result<HashMap<(i32, i32), V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    let entries = BTreeMap::<String, V>::deserialize(deserializer)?;
    entries
        .into_iter()
        .map(|(key, value)| parse_key::<D::Error>(&key).map(|position| (position, value)))
        .collect()
}

fn parse_key<E: Error>(key: &str) -> Result<(i32, i32), E> {
    let (x, y) = key
        .split_once(',')
        .ok_or_else(|| E::custom(format!("invalid grid coordinate key: {key}")))?;
    let x = x
        .parse::<i32>()
        .map_err(|_| E::custom(format!("invalid grid x coordinate: {x}")))?;
    let y = y
        .parse::<i32>()
        .map_err(|_| E::custom(format!("invalid grid y coordinate: {y}")))?;
    Ok((x, y))
}

#[cfg(test)]
mod tests;
