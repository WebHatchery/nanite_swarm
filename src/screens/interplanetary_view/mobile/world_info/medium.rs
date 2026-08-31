//! The regular phone world dossier between short and tall layouts.

use crate::screens::interplanetary_view::shipping_panel::route_status_color;
use crate::screens::interplanetary_view::world_dossier::{
    compact_shipping_status, core_stage_summary, hazard_color, hazard_summary,
    storage_resource_label, world_rule_lines,
};
use crate::ui::{color_from_rgba, Colors};
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

use super::MobileWorldInfo;

pub(super) fn draw(info: &MobileWorldInfo<'_, '_>) {
    let text_w = info.text_w;
    draw_ui_text(
        &truncate_text_to_width(&info.planet.name, text_w, 16.0),
        info.x + 15.0,
        info.y + 25.0,
        16.0,
        color_from_rgba(&info.planet.color),
    );
    draw_ui_text(
        &truncate_text_to_width(&info.status_line, text_w, 10.0),
        info.x + 15.0,
        info.y + 44.0,
        10.0,
        info.status_color,
    );
    draw_ui_text(
        &truncate_text_to_width(
            &format!("Hazard: {}", hazard_summary(info.planet)),
            text_w,
            8.0,
        ),
        info.x + 15.0,
        info.y + 59.0,
        8.0,
        hazard_color(info.planet),
    );
    if let Some(telemetry) = info.telemetry {
        draw_ui_text(
            &truncate_text_to_width(
                &format!(
                    "{} | Stock M {:.0}/{:.0} D {:.0} B {:.0}{}",
                    core_stage_summary(info.view.data, &telemetry),
                    telemetry.minerals,
                    telemetry.mineral_capacity,
                    telemetry.data,
                    telemetry.biomass,
                    storage_resource_label(telemetry.storage_full_resource)
                        .map(|label| format!(" FULL {label}"))
                        .unwrap_or_default()
                ),
                text_w,
                8.0,
            ),
            info.x + 15.0,
            info.y + 73.0,
            8.0,
            Colors::TEXT,
        );
        draw_ui_text(
            &truncate_text_to_width(
                &format!(
                    "Refined A {:.0}  Parts {:.0}",
                    telemetry.alloy, telemetry.components
                ),
                text_w,
                8.0,
            ),
            info.x + 15.0,
            info.y + 86.0,
            8.0,
            Colors::TEXT,
        );
    }

    let compact_line_y = info.y + if info.telemetry.is_some() { 99.0 } else { 73.0 };
    if let (Some(order), Some(status)) = (info.order, info.route_status) {
        let route_line = compact_shipping_status(&info.view.data.planet(order.target).name, status);
        draw_ui_text(
            &truncate_text_to_width(&route_line, text_w, 8.0),
            info.x + 15.0,
            compact_line_y,
            8.0,
            route_status_color(status),
        );
    } else {
        let rules = world_rule_lines(info.view.data, info.index);
        let rule = rules
            .first()
            .map(|rule| format!("Rule: {rule}"))
            .unwrap_or_else(|| "Rule: no special build rules".to_string());
        let rule_color = if rules.len() == 1 && rules[0] == "No special build rules" {
            Colors::SUCCESS
        } else {
            Colors::WARNING
        };
        draw_ui_text(
            &truncate_text_to_width(&rule, text_w, 8.0),
            info.x + 15.0,
            compact_line_y,
            8.0,
            rule_color,
        );
    }

    let action = info.action(false);
    draw_ui_text(
        &truncate_text_to_width(&action, text_w, 9.0),
        info.x + 15.0,
        info.y
            + if info.telemetry.is_some() {
                115.0
            } else {
                89.0
            },
        9.0,
        info.action_color(),
    );
}
