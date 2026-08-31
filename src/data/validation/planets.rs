//! Validation for planet geometry, hazards, features, and restrictions.

use std::collections::HashSet;

use crate::data::{GameData, PlanetDef};

pub(super) fn validate_planets(
    data: &GameData,
    building_ids: &HashSet<&str>,
    research_ids: &HashSet<&str>,
) -> Result<(), String> {
    for planet in &data.planets {
        validate_planet_dimensions(planet)?;
        if !planet.orbit_radius.is_finite()
            || planet.orbit_radius <= 0.0
            || !planet.size.is_finite()
            || planet.size <= 0.0
            || planet
                .color
                .iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(format!(
                "assets/planets.json planet \"{}\" has invalid map presentation values",
                planet.id
            ));
        }
        if !planet.hazards.acid_rain.is_finite()
            || planet.hazards.acid_rain < 0.0
            || !planet.hazards.freeze.is_finite()
            || planet.hazards.freeze < 0.0
        {
            return Err(format!(
                "assets/planets.json planet \"{}\" has invalid hazard strength",
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
        validate_planet_constraints(planet)?;
        for feature in &planet.features {
            if !feature.width.is_finite()
                || !feature.height.is_finite()
                || feature.width <= 0.0
                || feature.height <= 0.0
                || feature.width > 1.0
                || feature.height > 1.0
            {
                return Err(format!(
                    "assets/planets.json planet \"{}\" feature \"{}\" has invalid bounds",
                    planet.id, feature.id
                ));
            }
            if feature
                .ore_multiplier
                .is_some_and(|multiplier| !multiplier.is_finite() || multiplier < 1.0)
            {
                return Err(format!(
                    "assets/planets.json planet \"{}\" feature \"{}\" has invalid ore multiplier",
                    planet.id, feature.id
                ));
            }
            if feature
                .acid_multiplier
                .is_some_and(|multiplier| !multiplier.is_finite() || multiplier < 1.0)
            {
                return Err(format!(
                    "assets/planets.json planet \"{}\" feature \"{}\" has invalid acid multiplier",
                    planet.id, feature.id
                ));
            }
            if feature
                .freeze_multiplier
                .is_some_and(|multiplier| !multiplier.is_finite() || multiplier < 1.0)
            {
                return Err(format!(
                    "assets/planets.json planet \"{}\" feature \"{}\" has invalid freeze multiplier",
                    planet.id, feature.id
                ));
            }
            if feature
                .cooling_multiplier
                .is_some_and(|multiplier| !multiplier.is_finite() || multiplier < 1.0)
            {
                return Err(format!(
                    "assets/planets.json planet \"{}\" feature \"{}\" has invalid cooling multiplier",
                    planet.id, feature.id
                ));
            }
            if feature
                .harvest_multiplier
                .is_some_and(|multiplier| !multiplier.is_finite() || multiplier < 1.0)
            {
                return Err(format!(
                    "assets/planets.json planet \"{}\" feature \"{}\" has invalid harvest multiplier",
                    planet.id, feature.id
                ));
            }
        }
        if let Some(error) = super::duplicate_id(
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
                || field
                    .center
                    .iter()
                    .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            {
                return Err(format!(
                    "assets/planets.json planet \"{}\" field \"{}\" has invalid hazard definition",
                    planet.id, field.id
                ));
            }
        }
        if let Some(error) = super::duplicate_id(
            &format!("assets/planets.json planet \"{}\" hazard fields", planet.id),
            planet.hazard_fields.iter().map(|field| field.id.as_str()),
        ) {
            return Err(error);
        }
    }
    Ok(())
}

fn validate_planet_dimensions(planet: &PlanetDef) -> Result<(), String> {
    if planet.width == 0 || planet.height == 0 {
        return Err(format!(
            "assets/planets.json planet \"{}\" has an empty map",
            planet.id
        ));
    }
    let weights = [
        planet.terrain.mountain,
        planet.terrain.forest,
        planet.terrain.water,
        planet.terrain.void,
    ];
    let terrain_sum = weights.into_iter().sum::<f32>();
    if weights
        .into_iter()
        .any(|weight| !weight.is_finite() || weight < 0.0)
        || !terrain_sum.is_finite()
        || !(0.0..=1.0).contains(&terrain_sum)
    {
        return Err(format!(
            "assets/planets.json planet \"{}\" has invalid terrain weights",
            planet.id
        ));
    }
    Ok(())
}

fn validate_planet_constraints(planet: &PlanetDef) -> Result<(), String> {
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
    Ok(())
}
