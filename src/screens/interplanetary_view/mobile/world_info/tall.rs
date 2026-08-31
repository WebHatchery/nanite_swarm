//! The full world dossier for a comfortable phone viewport.

use crate::screens::interplanetary_view::shipping_panel::route_status_color;
use crate::screens::interplanetary_view::world_dossier::{
    core_stage_summary, hazard_color, hazard_summary, shipping_status_line, storage_resource_label,
    world_rule_lines,
};
use crate::ui::{color_from_rgba, Colors};
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

use super::MobileWorldInfo;

pub(super) fn draw(info: &MobileWorldInfo<'_, '_>) {
    let text_w = info.text_w;
    draw_ui_text(
        &truncate_text_to_width(&info.planet.name, text_w, 18.0),
        info.x + 15.0,
        info.y + 28.0,
        18.0,
        color_from_rgba(&info.planet.color),
    );
    draw_ui_text(
        &truncate_text_to_width(&info.planet.description, text_w, 10.0),
        info.x + 15.0,
        info.y + 48.0,
        10.0,
        Colors::TEXT,
    );
    draw_ui_text(
        &truncate_text_to_width(&info.planet.difficulty, text_w, 10.0),
        info.x + 15.0,
        info.y + 64.0,
        10.0,
        Colors::TEXT_DIM,
    );
    draw_ui_text(
        &truncate_text_to_width(&info.status_line, text_w, 11.0),
        info.x + 15.0,
        info.y + 86.0,
        11.0,
        info.status_color,
    );
    draw_ui_text(
        &truncate_text_to_width(
            &format!("Hazards: {}", hazard_summary(info.planet)),
            text_w,
            10.0,
        ),
        info.x + 15.0,
        info.y + 104.0,
        10.0,
        hazard_color(info.planet),
    );

    let mut next_y = info.y + 124.0;
    if let Some(telemetry) = info.telemetry {
        let live = format!(
            "{} | Live O +{:.1}/s  A +{:.1}/s  Parts +{:.1}/s",
            core_stage_summary(info.view.data, &telemetry),
            telemetry.ore_rate,
            telemetry.alloy_rate,
            telemetry.components_rate
        );
        draw_ui_text(
            &truncate_text_to_width(&live, text_w, 9.0),
            info.x + 15.0,
            next_y,
            9.0,
            Colors::TEXT,
        );
        draw_ui_text(
            &truncate_text_to_width(
                &format!(
                    "Stock M {:.0}/{:.0} D {:.0} B {:.0}{}  Grid {:+.1}/s",
                    telemetry.minerals,
                    telemetry.mineral_capacity,
                    telemetry.data,
                    telemetry.biomass,
                    storage_resource_label(telemetry.storage_full_resource)
                        .map(|label| format!(" FULL {label}"))
                        .unwrap_or_default(),
                    telemetry.power_balance
                ),
                text_w,
                9.0,
            ),
            info.x + 15.0,
            next_y + 14.0,
            9.0,
            if telemetry.alerts == 0 {
                Colors::SUCCESS
            } else {
                Colors::ERROR
            },
        );
        draw_ui_text(
            &truncate_text_to_width(
                &format!(
                    "Refined A {:.0}  Parts {:.0}",
                    telemetry.alloy, telemetry.components
                ),
                text_w,
                9.0,
            ),
            info.x + 15.0,
            next_y + 28.0,
            9.0,
            Colors::TEXT,
        );
        next_y += 46.0;
    }
    if let Some(shipping) = info.shipping.as_deref() {
        draw_ui_text(
            &truncate_text_to_width(shipping, text_w, 9.0),
            info.x + 15.0,
            next_y,
            9.0,
            Colors::PRIMARY,
        );
        if let Some(status) = info.route_status {
            draw_ui_text(
                &truncate_text_to_width(&shipping_status_line(status), text_w, 9.0),
                info.x + 15.0,
                next_y + 14.0,
                9.0,
                route_status_color(status),
            );
        }
        next_y += 32.0;
    }
    draw_ui_text(
        "World rules:",
        info.x + 15.0,
        next_y,
        10.0,
        Colors::TEXT_DIM,
    );
    let rules = world_rule_lines(info.view.data, info.index);
    for (rule_index, rule) in rules.iter().take(3).enumerate() {
        draw_ui_text(
            &truncate_text_to_width(rule, text_w, 9.0),
            info.x + 15.0,
            next_y + 14.0 + rule_index as f32 * 13.0,
            9.0,
            if rule == "No special build rules" {
                Colors::SUCCESS
            } else {
                Colors::WARNING
            },
        );
    }

    let action = info.action(true);
    draw_ui_text(
        &truncate_text_to_width(&action, text_w, 10.0),
        info.x + 15.0,
        info.y + info.panel_h - 14.0,
        10.0,
        info.action_color(),
    );
}
