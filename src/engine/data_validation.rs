//! Checks that data declarations can be understood by the simulation engine.

use crate::data::GameData;

use super::{parse_modifier, BuildingType, ResourceType, StatId};

#[cfg(test)]
mod tests;

pub(crate) fn validate_game_data(data: &GameData) -> Result<(), String> {
    validate_buildings(data)?;
    validate_research(data)?;
    validate_seed_ship(data)?;
    validate_core_stages(data)
}

fn validate_buildings(data: &GameData) -> Result<(), String> {
    for def in &data.buildings {
        if BuildingType::from_id(&def.id).is_none() {
            return Err(format!(
                "assets/buildings.json building \"{}\" has no engine type",
                def.id
            ));
        }
        if !is_nonnegative_finite(def.cost.minerals) || !is_nonnegative_finite(def.cost.energy) {
            return Err(format!(
                "assets/buildings.json building \"{}\" has invalid construction cost",
                def.id
            ));
        }
        if !is_nonnegative_finite(def.power_generation)
            || !is_nonnegative_finite(def.power_consumption)
        {
            return Err(format!(
                "assets/buildings.json building \"{}\" has invalid power values",
                def.id
            ));
        }
        if let Some(condition) = &def.planet_condition {
            if !["acid", "cold", "water", "mountain"].contains(&condition.as_str()) {
                return Err(format!(
                    "assets/buildings.json building \"{}\" has unknown planet condition \"{}\"",
                    def.id, condition
                ));
            }
        }
        for (resource, value) in def.recipe.inputs.iter().chain(def.recipe.outputs.iter()) {
            if ResourceType::from_id(resource).is_none() {
                return Err(format!(
                    "assets/buildings.json building \"{}\" recipe uses unknown resource \"{}\"",
                    def.id, resource
                ));
            }
            if !value.is_finite() || *value < 0.0 {
                return Err(format!(
                    "assets/buildings.json building \"{}\" recipe has invalid rate for \"{}\"",
                    def.id, resource
                ));
            }
        }
        for carried in def.recipe.carried_ids() {
            if !def.recipe.inputs.contains_key(carried) {
                return Err(format!(
                    "assets/buildings.json building \"{}\" carries missing input \"{}\"",
                    def.id, carried
                ));
            }
            if ResourceType::from_id(carried).is_none_or(|resource| !resource.is_physical()) {
                return Err(format!(
                    "assets/buildings.json building \"{}\" carries non-physical input \"{}\"",
                    def.id, carried
                ));
            }
        }
    }
    Ok(())
}

fn validate_research(data: &GameData) -> Result<(), String> {
    if !data.research.stats.is_empty() {
        for entry in &data.research.stats {
            if StatId::from_id(&entry.id).is_none() {
                return Err(format!("research stats: unknown stat \"{}\"", entry.id));
            }
        }
        for stat in StatId::ALL {
            if !data
                .research
                .stats
                .iter()
                .any(|entry| entry.id == stat.id())
            {
                return Err(format!("research stats: no label for \"{}\"", stat.id()));
            }
        }
    }
    for node in &data.research.nodes {
        if !is_nonnegative_finite(node.data_cost) {
            return Err(format!(
                "assets/research.json node \"{}\" has an invalid data cost",
                node.id
            ));
        }
        if !node.position.0.is_finite() || !node.position.1.is_finite() {
            return Err(format!(
                "assets/research.json node \"{}\" has invalid map position",
                node.id
            ));
        }
        for modifier in &node.modifiers {
            parse_modifier(modifier)
                .map_err(|problem| format!("research node \"{}\": {problem}", node.id))?;
        }
    }
    Ok(())
}

fn validate_seed_ship(data: &GameData) -> Result<(), String> {
    let intake = data.seed_ship.intake_per_second;
    if [
        intake.minerals,
        intake.data,
        intake.biomass,
        intake.alloy,
        intake.components,
    ]
    .into_iter()
    .any(|value| !is_nonnegative_finite(value))
    {
        return Err("assets/seed_ship.json has invalid intake rates".to_string());
    }
    for stage in &data.seed_ship.stages {
        let cost = stage.cost;
        if [
            cost.minerals,
            cost.data,
            cost.biomass,
            cost.alloy,
            cost.components,
        ]
        .into_iter()
        .any(|value| !is_nonnegative_finite(value))
        {
            return Err(format!(
                "assets/seed_ship.json stage \"{}\" has an invalid cost",
                stage.id
            ));
        }
        for modifier in &stage.modifiers {
            parse_modifier(modifier).map_err(|problem| {
                format!("assets/seed_ship.json stage \"{}\": {problem}", stage.id)
            })?;
        }
    }
    Ok(())
}

fn validate_core_stages(data: &GameData) -> Result<(), String> {
    for stage in &data.core_stages {
        for modifier in &stage.modifiers {
            parse_modifier(modifier).map_err(|problem| {
                format!("assets/core_stages.json stage \"{}\": {problem}", stage.id)
            })?;
        }
    }
    Ok(())
}

fn is_nonnegative_finite(value: f32) -> bool {
    value.is_finite() && value >= 0.0
}
