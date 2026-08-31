use crate::data::{GameData, PlanetDef};
use crate::ui::Colors;
use macroquad::prelude::Color;

pub(crate) fn hazard_summary(planet: &PlanetDef) -> String {
    let mut hazards = Vec::new();
    if planet.hazards.acid_rain > 0.0
        || planet
            .hazard_fields
            .iter()
            .any(|field| field.hazard == "acid")
    {
        hazards.push("ACID RAIN");
    }
    if planet.hazards.freeze > 0.0
        || planet
            .hazard_fields
            .iter()
            .any(|field| field.hazard == "cold")
    {
        hazards.push("COLD");
    }
    if hazards.is_empty() {
        "NONE".to_string()
    } else {
        hazards.join(" / ")
    }
}

pub(crate) fn hazard_color(planet: &PlanetDef) -> Color {
    if planet.hazards.any() || !planet.hazard_fields.is_empty() {
        Colors::WARNING
    } else {
        Colors::SUCCESS
    }
}

pub(crate) fn world_rule_lines(data: &GameData, index: usize) -> Vec<String> {
    let planet = &data.planets[index];
    let constraints = &planet.constraints;
    let power_source = world_power_source(data, planet);
    let mut lines = Vec::new();
    let rule_count_without_bans = constraints.required_buildings.len()
        + constraints.required_research.len()
        + usize::from(constraints.minimum_power_generation > 0.0)
        + usize::from(constraints.minimum_power_balance > 0.0);
    if planet.banned_buildings.len() > 1
        && planet.banned_buildings.len() + rule_count_without_bans > 3
    {
        let unavailable = planet
            .banned_buildings
            .iter()
            .map(|id| building_name(data, id))
            .collect::<Vec<_>>()
            .join(" + ");
        lines.push(format!("Unavailable: {unavailable}"));
    } else {
        for id in &planet.banned_buildings {
            lines.push(format!("Unavailable: {}", building_name(data, id)));
        }
    }
    for id in &constraints.required_buildings {
        lines.push(format!("Needs {}", building_name(data, id)));
    }
    for id in &constraints.required_research {
        let name = data
            .research
            .nodes
            .iter()
            .find(|node| node.id == *id)
            .map(|node| node.name.as_str())
            .unwrap_or(id.as_str());
        lines.push(format!("Needs research: {}", name));
    }
    if constraints.minimum_power_generation > 0.0 {
        let source_suffix = power_source
            .map(|name| format!(" via {name}"))
            .unwrap_or_default();
        lines.push(format!(
            "Needs {:.0} power generation{source_suffix}",
            constraints.minimum_power_generation
        ));
    }
    if constraints.minimum_power_balance > 0.0 {
        lines.push(format!(
            "Needs {:.0} power surplus",
            constraints.minimum_power_balance
        ));
    }
    if lines.is_empty() {
        if let Some(source) = power_source {
            lines.push(format!("Power source: {source}"));
        } else {
            lines.push("No special build rules".to_string());
        }
    } else if constraints.minimum_power_generation <= 0.0 {
        if let Some(source) = power_source {
            lines.push(format!("Power source: {source}"));
        }
    }
    lines.truncate(3);
    lines
}

fn world_power_source<'a>(data: &'a GameData, planet: &PlanetDef) -> Option<&'a str> {
    let condition = if planet.hazards.acid_rain > 0.0
        || planet
            .hazard_fields
            .iter()
            .any(|field| field.hazard == "acid")
    {
        Some("acid")
    } else if planet.hazards.freeze > 0.0
        || planet
            .hazard_fields
            .iter()
            .any(|field| field.hazard == "cold")
    {
        Some("cold")
    } else {
        None
    };
    let condition = condition?;
    data.buildings
        .iter()
        .find(|building| {
            building.generates_power && building.planet_condition.as_deref() == Some(condition)
        })
        .map(|building| building.name.as_str())
}

fn building_name<'a>(data: &'a GameData, id: &'a str) -> &'a str {
    data.buildings
        .iter()
        .find(|building| building.id == id)
        .map(|building| building.name.as_str())
        .unwrap_or(id)
}
