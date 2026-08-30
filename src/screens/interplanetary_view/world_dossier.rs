//! World dossier details shared by the desktop map and portrait map.

use crate::engine::ResourceType;
use crate::state::{cargo_name, ExportOrder, UpkeepAlertCause, WorldAlert, WorldTelemetry};
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

const TELEMETRY_LINE_HEIGHT: f32 = 16.0;
const TELEMETRY_BOTTOM_GAP: f32 = 6.0;

pub(super) fn world_telemetry_row_y(y: f32, row: usize) -> f32 {
    y + row as f32 * TELEMETRY_LINE_HEIGHT
}

pub(super) fn world_telemetry_height() -> f32 {
    TELEMETRY_LINE_HEIGHT * 4.0 + TELEMETRY_BOTTOM_GAP
}

/// The standing freight order for a world, short enough to share its dossier
/// with the live telemetry and construction rules.
pub(super) fn shipping_summary(
    data: &crate::data::GameData,
    order: Option<ExportOrder>,
    rate_per_minute: f32,
) -> Option<String> {
    let order = order?;
    Some(format!(
        "Shipping: {} -> {} @ {:.0}/min",
        cargo_name(order.cargo),
        data.planet(order.target).name,
        rate_per_minute.max(0.0)
    ))
}

pub(super) fn shipping_status_line(status: &str) -> String {
    format!("Route status: {status}")
}

pub(super) fn compact_shipping_status(target: &str, status: &str) -> String {
    format!("Route: {target} {status}")
}

pub(super) fn storage_resource_label(resource: Option<ResourceType>) -> Option<&'static str> {
    match resource {
        Some(ResourceType::Minerals) => Some("ORE"),
        Some(ResourceType::Data) => Some("DATA"),
        Some(ResourceType::Biomass) => Some("BIO"),
        Some(ResourceType::Alloy) => Some("ALLOY"),
        Some(ResourceType::Components) => Some("PARTS"),
        Some(ResourceType::Energy) | None => None,
    }
}

pub(super) fn maintenance_cause_label(cause: Option<UpkeepAlertCause>) -> Option<&'static str> {
    cause.map(UpkeepAlertCause::label)
}

pub(super) fn draw_world_telemetry(telemetry: &WorldTelemetry, x: f32, y: f32) {
    let storage_suffix = storage_resource_label(telemetry.storage_full_resource)
        .map(|label| format!("  FULL {label}"))
        .unwrap_or_default();
    draw_ui_text(
        &format!(
            "Live: O +{:.1}/s  A +{:.1}/s  Parts +{:.1}/s",
            telemetry.ore_rate, telemetry.alloy_rate, telemetry.components_rate
        ),
        x,
        y,
        9.0,
        Colors::TEXT,
    );
    draw_ui_text(
        &format!(
            "Data +{:.2}/s  Biomass +{:.1}/s",
            telemetry.data_rate, telemetry.biomass_rate
        ),
        x,
        world_telemetry_row_y(y, 1),
        9.0,
        Colors::TEXT,
    );
    draw_ui_text(
        &format!(
            "Stock M {:.0}  D {:.0}  B {:.0}  A {:.0}  P {:.0}{}",
            telemetry.minerals,
            telemetry.data,
            telemetry.biomass,
            telemetry.alloy,
            telemetry.components,
            storage_suffix,
        ),
        x,
        world_telemetry_row_y(y, 2),
        9.0,
        Colors::TEXT,
    );
    let health = if telemetry.alerts == 0 {
        (WorldAlert::Nominal.label().to_string(), Colors::SUCCESS)
    } else {
        let detail = if telemetry.alert_kind == WorldAlert::Maintenance {
            maintenance_cause_label(telemetry.maintenance_cause)
                .map(|cause| format!(" {cause}"))
                .unwrap_or_default()
        } else {
            String::new()
        };
        (
            format!(
                "{}{} x{}",
                telemetry.alert_kind.label(),
                detail,
                telemetry.alerts
            ),
            Colors::ERROR,
        )
    };
    draw_ui_text(
        &format!(
            "Grid {:+.1}/s  DR {}  S {}  {}",
            telemetry.power_balance, telemetry.drones, telemetry.structures, health.0
        ),
        x,
        world_telemetry_row_y(y, 3),
        9.0,
        health.1,
    );
}

pub(super) fn world_alert_code(telemetry: Option<WorldTelemetry>) -> String {
    let Some(telemetry) = telemetry else {
        return "--".to_string();
    };
    let code = match telemetry.alert_kind {
        WorldAlert::Nominal => "OK".to_string(),
        WorldAlert::Collapse => "COLL".to_string(),
        WorldAlert::Power => "PWR".to_string(),
        WorldAlert::Hibernation => "SLEEP".to_string(),
        WorldAlert::Stalled => "STALL".to_string(),
        WorldAlert::Starved => "INPUT".to_string(),
        WorldAlert::Blocked => "FULL".to_string(),
        WorldAlert::Storage => storage_resource_label(telemetry.storage_full_resource)
            .map(|label| format!("STORE {label}"))
            .unwrap_or_else(|| "STORE".to_string()),
        WorldAlert::Maintenance => maintenance_cause_label(telemetry.maintenance_cause)
            .map(|cause| format!("MAINT {cause}"))
            .unwrap_or_else(|| "MAINT".to_string()),
    };
    if telemetry.alerts > 1 {
        format!("{} x{}", code, telemetry.alerts)
    } else {
        code
    }
}

pub(super) fn world_alert_color(telemetry: Option<WorldTelemetry>) -> Color {
    if telemetry.is_some_and(|telemetry| telemetry.alerts > 0) {
        Colors::ERROR
    } else if telemetry.is_some() {
        Colors::SUCCESS
    } else {
        Colors::TEXT_DIM
    }
}

pub(super) fn hazard_summary(planet: &crate::data::PlanetDef) -> String {
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

pub(super) fn hazard_color(planet: &crate::data::PlanetDef) -> Color {
    if planet.hazards.any() || !planet.hazard_fields.is_empty() {
        Colors::WARNING
    } else {
        Colors::SUCCESS
    }
}

pub(super) fn world_rule_lines(data: &crate::data::GameData, index: usize) -> Vec<String> {
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

fn world_power_source<'a>(
    data: &'a crate::data::GameData,
    planet: &crate::data::PlanetDef,
) -> Option<&'a str> {
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

fn building_name<'a>(data: &'a crate::data::GameData, id: &'a str) -> &'a str {
    data.buildings
        .iter()
        .find(|building| building.id == id)
        .map(|building| building.name.as_str())
        .unwrap_or(id)
}

#[cfg(test)]
mod tests;
