//! The minimal world dossier for very short phone viewports.

use crate::screens::interplanetary_view::world_dossier::{
    compact_shipping_status, core_stage_summary, hazard_color, hazard_summary, world_rule_lines,
};
use crate::ui::{color_from_rgba, Colors};
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

use super::{compact_stockpile_summary, MobileWorldInfo};

pub(super) fn draw(info: &MobileWorldInfo<'_, '_>) {
    let text_w = info.text_w;
    draw_ui_text(
        &truncate_text_to_width(&info.planet.name, text_w, 14.0),
        info.x + 12.0,
        info.y + 18.0,
        14.0,
        color_from_rgba(&info.planet.color),
    );
    draw_ui_text(
        &truncate_text_to_width(&info.status_line, text_w, 8.0),
        info.x + 12.0,
        info.y + 34.0,
        8.0,
        info.status_color,
    );
    draw_ui_text(
        &truncate_text_to_width(
            &format!("Hazard: {}", hazard_summary(info.planet)),
            text_w,
            7.0,
        ),
        info.x + 12.0,
        info.y + 47.0,
        7.0,
        hazard_color(info.planet),
    );

    let compact_detail = if let Some(telemetry) = info.telemetry {
        let stock = compact_stockpile_summary(&telemetry);
        if let (Some(order), Some(status)) = (info.order, info.route_status) {
            format!(
                "{stock} | {}",
                compact_shipping_status(&info.view.data.planet(order.target).name, status)
            )
        } else {
            format!(
                "{} | {stock}",
                core_stage_summary(info.view.data, &telemetry)
            )
        }
    } else if let (Some(order), Some(status)) = (info.order, info.route_status) {
        compact_shipping_status(&info.view.data.planet(order.target).name, status)
    } else {
        world_rule_lines(info.view.data, info.index)
            .first()
            .map(|rule| format!("Rule: {rule}"))
            .unwrap_or_else(|| "Rule: no special build rules".to_string())
    };
    draw_ui_text(
        &truncate_text_to_width(&compact_detail, text_w, 7.0),
        info.x + 12.0,
        info.y + 59.0,
        7.0,
        Colors::TEXT_DIM,
    );

    let action = info.action(false);
    draw_ui_text(
        &truncate_text_to_width(&action, text_w, 8.0),
        info.x + 12.0,
        info.y + info.panel_h - 8.0,
        8.0,
        info.action_color(),
    );
}
