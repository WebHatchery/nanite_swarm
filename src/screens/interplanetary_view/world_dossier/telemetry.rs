use crate::data::GameData;
use crate::engine::ResourceType;
use crate::state::{
    CoreRequirementProgress, Milestone, UpkeepAlertCause, WorldAlert, WorldTelemetry,
};
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

const TELEMETRY_LINE_HEIGHT: f32 = 16.0;
const TELEMETRY_BOTTOM_GAP: f32 = 6.0;

pub(crate) fn world_telemetry_row_y(y: f32, row: usize) -> f32 {
    y + row as f32 * TELEMETRY_LINE_HEIGHT
}

pub(crate) fn world_telemetry_height() -> f32 {
    TELEMETRY_LINE_HEIGHT * 4.0 + TELEMETRY_BOTTOM_GAP
}

pub(crate) fn storage_resource_label(resource: Option<ResourceType>) -> Option<&'static str> {
    match resource {
        Some(ResourceType::Minerals) => Some("ORE"),
        Some(ResourceType::Data) => Some("DATA"),
        Some(ResourceType::Biomass) => Some("BIO"),
        Some(ResourceType::Alloy) => Some("ALLOY"),
        Some(ResourceType::Components) => Some("PARTS"),
        Some(ResourceType::Energy) | None => None,
    }
}

pub(crate) fn maintenance_cause_label(cause: Option<UpkeepAlertCause>) -> Option<&'static str> {
    cause.map(UpkeepAlertCause::label)
}

pub(crate) fn core_stage_summary(data: &GameData, telemetry: &WorldTelemetry) -> String {
    let next_stage = data
        .core_stages
        .get(telemetry.core_stage)
        .map(|stage| stage.name.split_whitespace().last().unwrap_or(&stage.name));
    let stage = match next_stage {
        Some(next) => format!(
            "Core {}/{} > {}",
            telemetry.core_stage, telemetry.core_stage_count, next
        ),
        None => format!(
            "Core {}/{}",
            telemetry.core_stage, telemetry.core_stage_count
        ),
    };
    telemetry
        .core_next_requirement
        .map(|requirement| format!("{stage} / {}", core_requirement_summary(requirement)))
        .unwrap_or(stage)
}

fn core_requirement_summary(progress: CoreRequirementProgress) -> String {
    format!(
        "{}{}/{}",
        core_requirement_label(progress.milestone),
        compact_requirement_amount(progress.current),
        compact_requirement_amount(progress.target)
    )
}

fn core_requirement_label(milestone: Milestone) -> &'static str {
    match milestone {
        Milestone::Structures => "STR",
        Milestone::NetworkTiles => "NET",
        Milestone::Technologies => "TECH",
        Milestone::SeedShipStages => "SHIP",
        _ => milestone.id(),
    }
}

fn compact_requirement_amount(amount: f32) -> String {
    if amount.fract().abs() > 0.05 {
        format!("{amount:.1}")
    } else {
        format!("{amount:.0}")
    }
}

pub(crate) fn draw_world_telemetry(data: &GameData, telemetry: &WorldTelemetry, x: f32, y: f32) {
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
            "Stock M {:.0}/{:.0}  D {:.0}  B {:.0}  A {:.0}  P {:.0}{}",
            telemetry.minerals,
            telemetry.mineral_capacity,
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
            "{}  Grid {:+.1}/s  DR {}  S {}  {}",
            core_stage_summary(data, telemetry),
            telemetry.power_balance,
            telemetry.drones,
            telemetry.structures,
            health.0
        ),
        x,
        world_telemetry_row_y(y, 3),
        9.0,
        health.1,
    );
}

pub(crate) fn world_alert_code(telemetry: Option<WorldTelemetry>) -> String {
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

pub(crate) fn world_alert_color(telemetry: Option<WorldTelemetry>) -> Color {
    if telemetry.is_some_and(|telemetry| telemetry.alerts > 0) {
        Colors::ERROR
    } else if telemetry.is_some() {
        Colors::SUCCESS
    } else {
        Colors::TEXT_DIM
    }
}
