//! World dossier details shared by the desktop map and portrait map.

mod rules;
mod shipping;
mod telemetry;

pub(super) use rules::{hazard_color, hazard_summary, world_rule_lines};
pub(super) use shipping::{compact_shipping_status, shipping_status_line, shipping_summary};
pub(super) use telemetry::{
    core_stage_summary, draw_world_telemetry, storage_resource_label, world_alert_code,
    world_alert_color, world_telemetry_height,
};

#[cfg(test)]
pub(super) use telemetry::{maintenance_cause_label, world_telemetry_row_y};

#[cfg(test)]
mod tests;
