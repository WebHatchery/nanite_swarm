use std::collections::HashSet;

use crate::data::GameData;

fn duplicate_id<'a>(source: &str, mut ids: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut seen = HashSet::new();
    ids.find_map(|id| {
        (!seen.insert(id)).then(|| format!("{source} contains duplicate identifier \"{id}\""))
    })
}

/// Validate cross-file references once, with the source asset and identifier
/// in every failure. Runtime lookups remain compact because a malformed
/// package cannot reach them.
pub fn validate_game_data(data: &GameData) -> Result<(), String> {
    crate::engine::validate_game_data(data)?;
    crate::directives::validate_game_data(data)?;
    crate::state::validate_game_data(data)?;

    if let Some(error) = duplicate_id(
        "assets/buildings.json",
        data.buildings.iter().map(|def| def.id.as_str()),
    ) {
        return Err(error);
    }
    if let Some(error) = duplicate_id(
        "assets/terrain.json",
        data.terrain.iter().map(|def| def.id.as_str()),
    ) {
        return Err(error);
    }
    if let Some(error) = duplicate_id(
        "assets/research.json",
        data.research.nodes.iter().map(|node| node.id.as_str()),
    ) {
        return Err(error);
    }
    if let Some(error) = duplicate_id(
        "assets/planets.json",
        data.planets.iter().map(|planet| planet.id.as_str()),
    ) {
        return Err(error);
    }
    if let Some(error) = duplicate_id(
        "assets/directives.json",
        data.directives.directives.iter().map(|def| def.id.as_str()),
    ) {
        return Err(error);
    }
    if let Some(error) = duplicate_id(
        "assets/achievements.json",
        data.achievements.iter().map(|def| def.id.as_str()),
    ) {
        return Err(error);
    }
    if let Some(error) = duplicate_id(
        "assets/core_stages.json",
        data.core_stages.iter().map(|stage| stage.id.as_str()),
    ) {
        return Err(error);
    }
    if let Some(error) = duplicate_id(
        "assets/seed_ship.json",
        data.seed_ship.stages.iter().map(|stage| stage.id.as_str()),
    ) {
        return Err(error);
    }

    let building_ids: HashSet<_> = data.buildings.iter().map(|def| def.id.as_str()).collect();
    let research_ids: HashSet<_> = data
        .research
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    let terrain_ids: HashSet<_> = data.terrain.iter().map(|def| def.id.as_str()).collect();

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

    for terrain in &data.terrain {
        if !terrain_ids.contains(terrain.harvested_to.as_str()) {
            return Err(format!(
                "assets/terrain.json terrain \"{}\" references missing harvested_to \"{}\"",
                terrain.id, terrain.harvested_to
            ));
        }
    }

    for planet in &data.planets {
        if planet.width == 0 || planet.height == 0 {
            return Err(format!(
                "assets/planets.json planet \"{}\" has an empty map",
                planet.id
            ));
        }
        let terrain_sum = planet.terrain.mountain
            + planet.terrain.forest
            + planet.terrain.water
            + planet.terrain.void;
        if !terrain_sum.is_finite() || !(0.0..=1.0).contains(&terrain_sum) {
            return Err(format!(
                "assets/planets.json planet \"{}\" has invalid terrain weights",
                planet.id
            ));
        }
        for id in planet
            .banned_buildings
            .iter()
            .chain(planet.constraints.required_buildings.iter())
        {
            if !building_ids.contains(id.as_str()) {
                return Err(format!(
                    "assets/planets.json planet \"{}\" references missing building \"{}\"",
                    planet.id, id
                ));
            }
        }
        for id in &planet.constraints.required_research {
            if !research_ids.contains(id.as_str()) {
                return Err(format!(
                    "assets/planets.json planet \"{}\" references missing research \"{}\"",
                    planet.id, id
                ));
            }
        }
        if planet.pending_pod_cap == 0 {
            return Err(format!(
                "assets/planets.json planet \"{}\" has non-positive pending pod cap",
                planet.id
            ));
        }
        if !planet.constraints.minimum_power_generation.is_finite()
            || !planet.constraints.minimum_power_balance.is_finite()
            || planet.constraints.minimum_power_generation < 0.0
            || planet.constraints.minimum_power_balance < 0.0
        {
            return Err(format!(
                "assets/planets.json planet \"{}\" has invalid power constraints",
                planet.id
            ));
        }
        for feature in &planet.features {
            if feature.width <= 0.0
                || feature.height <= 0.0
                || feature.width > 1.0
                || feature.height > 1.0
                || feature.width + feature.height <= 0.0
            {
                return Err(format!(
                    "assets/planets.json planet \"{}\" feature \"{}\" has invalid bounds",
                    planet.id, feature.id
                ));
            }
        }
        if let Some(error) = duplicate_id(
            &format!("assets/planets.json planet \"{}\" features", planet.id),
            planet.features.iter().map(|feature| feature.id.as_str()),
        ) {
            return Err(error);
        }
        for field in &planet.hazard_fields {
            if !["acid", "cold"].contains(&field.hazard.as_str())
                || !field.radius.is_finite()
                || field.radius <= 0.0
                || !field.strength.is_finite()
                || field.strength < 0.0
                || field.center.iter().any(|value| !value.is_finite())
            {
                return Err(format!(
                    "assets/planets.json planet \"{}\" field \"{}\" has invalid hazard definition",
                    planet.id, field.id
                ));
            }
        }
    }

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
