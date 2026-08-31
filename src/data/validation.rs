//! Checks that the shipped definition files agree with one another.
//!
//! The engine, directives, and state modules validate the vocabularies they
//! own. This module owns the schema-level work that belongs to the data
//! package: unique identifiers, cross-file references, and asset-shaped
//! values such as map weights and feature bounds.

use std::collections::HashSet;

use super::GameData;

mod planets;

use planets::validate_planets;

fn duplicate_id<'a>(source: &str, mut ids: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut seen = HashSet::new();
    ids.find_map(|id| {
        (!seen.insert(id)).then(|| format!("{source} contains duplicate identifier \"{id}\""))
    })
}

/// Validate references between definition files before runtime lookups use
/// them. Errors retain the source asset and identifier so a bad package is
/// actionable on desktop and in the browser console.
pub(crate) fn validate_game_data(data: &GameData) -> Result<(), String> {
    validate_unique_ids(data)?;

    let building_ids: HashSet<_> = data.buildings.iter().map(|def| def.id.as_str()).collect();
    let research_ids: HashSet<_> = data
        .research
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    let terrain_ids: HashSet<_> = data.terrain.iter().map(|def| def.id.as_str()).collect();

    validate_building_unlocks(data, &research_ids)?;
    validate_research(data, &research_ids)?;
    validate_terrain(data, &terrain_ids)?;
    validate_planets(data, &building_ids, &research_ids)?;
    validate_core_sprites(data)?;
    validate_seed_ship(data, &research_ids)?;
    validate_tutorial(data, &research_ids)
}

fn validate_unique_ids(data: &GameData) -> Result<(), String> {
    let collections: [(&str, Vec<&str>); 8] = [
        (
            "assets/buildings.json",
            data.buildings.iter().map(|def| def.id.as_str()).collect(),
        ),
        (
            "assets/terrain.json",
            data.terrain.iter().map(|def| def.id.as_str()).collect(),
        ),
        (
            "assets/research.json",
            data.research
                .nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect(),
        ),
        (
            "assets/planets.json",
            data.planets
                .iter()
                .map(|planet| planet.id.as_str())
                .collect(),
        ),
        (
            "assets/directives.json",
            data.directives
                .directives
                .iter()
                .map(|def| def.id.as_str())
                .collect(),
        ),
        (
            "assets/achievements.json",
            data.achievements
                .iter()
                .map(|def| def.id.as_str())
                .collect(),
        ),
        (
            "assets/core_stages.json",
            data.core_stages
                .iter()
                .map(|stage| stage.id.as_str())
                .collect(),
        ),
        (
            "assets/seed_ship.json",
            data.seed_ship
                .stages
                .iter()
                .map(|stage| stage.id.as_str())
                .collect(),
        ),
    ];
    for (source, ids) in collections {
        if let Some(error) = duplicate_id(source, ids.into_iter()) {
            return Err(error);
        }
    }
    Ok(())
}

fn validate_building_unlocks(data: &GameData, research_ids: &HashSet<&str>) -> Result<(), String> {
    for def in &data.buildings {
        if let Some(required) = &def.unlocked_by {
            if !research_ids.contains(required.as_str()) {
                return Err(format!(
                    "assets/buildings.json building \"{}\" references missing unlock \"{}\"",
                    def.id, required
                ));
            }
        }
    }
    Ok(())
}

fn validate_research(data: &GameData, research_ids: &HashSet<&str>) -> Result<(), String> {
    for node in &data.research.nodes {
        for prerequisite in &node.prerequisites {
            if !research_ids.contains(prerequisite.as_str()) {
                return Err(format!(
                    "assets/research.json node \"{}\" references missing prerequisite \"{}\"",
                    node.id, prerequisite
                ));
            }
        }
        if let Some(condition) = &node.planet_condition {
            if !["acid", "cold", "water", "mountain"].contains(&condition.as_str()) {
                return Err(format!(
                    "assets/research.json node \"{}\" has unknown planet condition \"{}\"",
                    node.id, condition
                ));
            }
        }
    }
    for unlocked in &data.research.starting_unlocked {
        if !research_ids.contains(unlocked.as_str()) {
            return Err(format!(
                "assets/research.json starting unlock references missing node \"{}\"",
                unlocked
            ));
        }
    }
    Ok(())
}

fn validate_terrain(data: &GameData, terrain_ids: &HashSet<&str>) -> Result<(), String> {
    for terrain in &data.terrain {
        if !terrain_ids.contains(terrain.harvested_to.as_str()) {
            return Err(format!(
                "assets/terrain.json terrain \"{}\" references missing harvested_to \"{}\"",
                terrain.id, terrain.harvested_to
            ));
        }
    }
    Ok(())
}

fn validate_core_sprites(data: &GameData) -> Result<(), String> {
    let valid_sprites = [
        "core_stage_1a",
        "core_stage_1b",
        "core_stage_1c",
        "core_stage_2a",
        "core_stage_2b",
        "core_stage_3a",
        "core_stage_3b",
        "core_stage_4a",
        "core_stage_4b",
    ];
    for stage in &data.core_stages {
        if stage
            .sprite
            .as_deref()
            .is_some_and(|sprite| !valid_sprites.contains(&sprite))
        {
            return Err(format!(
                "assets/core_stages.json stage \"{}\" references missing sprite",
                stage.id
            ));
        }
    }
    Ok(())
}

fn validate_seed_ship(data: &GameData, research_ids: &HashSet<&str>) -> Result<(), String> {
    for stage in &data.seed_ship.stages {
        if let Some(required) = &stage.requires {
            if !research_ids.contains(required.as_str()) {
                return Err(format!(
                    "assets/seed_ship.json stage \"{}\" references missing research \"{}\"",
                    stage.id, required
                ));
            }
        }
    }
    Ok(())
}

fn validate_tutorial(data: &GameData, research_ids: &HashSet<&str>) -> Result<(), String> {
    for step in &data.tutorial {
        if step.goal.kind == "research" && !research_ids.contains(step.goal.target.as_str()) {
            return Err(format!(
                "assets/tutorial.json step \"{}\" references missing research \"{}\"",
                step.id, step.goal.target
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
