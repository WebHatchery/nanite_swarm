//! Touch-sized dossier details for one world in the Solar System list.

use crate::ui::{color_from_rgba, draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

use super::super::shipping_panel::{route_status_color, route_status_for};
use super::super::status::launch_prompt;
use super::super::world_dossier::{
    compact_shipping_status, core_stage_summary, hazard_color, hazard_summary,
    shipping_status_line, shipping_summary, storage_resource_label, world_alert_code,
    world_rule_lines,
};
use super::super::MapView;
use super::{mobile_status_text, SHORT_MOBILE_HEIGHT};

pub(super) fn draw_mobile_planet_info(
    view: &MapView,
    index: usize,
    can_launch: bool,
    x: f32,
    y: f32,
    width: f32,
    screen_h: f32,
) {
    let planet = &view.data.planets[index];
    let is_colonized = view.colonized.get(index).copied().unwrap_or(false);
    let is_current = index == view.current_planet;
    let telemetry = view.telemetry.get(index).copied().flatten();
    let alert_code = telemetry
        .filter(|telemetry| telemetry.alerts > 0)
        .map(|telemetry| world_alert_code(Some(telemetry)));
    let status_line = mobile_status_text(is_colonized, alert_code.as_deref());
    let status_color = if alert_code.is_some() {
        Colors::ERROR
    } else if is_colonized {
        Colors::SUCCESS
    } else {
        Colors::WARNING
    };
    let order = view.orders.get(index).copied().flatten();
    let shipping = shipping_summary(
        view.data,
        order,
        view.export_rates.get(index).copied().unwrap_or(0.0),
    );
    let route_status = order.map(|order| route_status_for(view, index, order));
    let panel_h = mobile_info_height(screen_h, telemetry.is_some(), shipping.is_some());
    let text_w = (width - 30.0).max(40.0);
    draw_panel(x, y, width, panel_h);
    if screen_h < SHORT_MOBILE_HEIGHT {
        draw_ui_text(
            &truncate_text_to_width(&planet.name, text_w, 14.0),
            x + 12.0,
            y + 18.0,
            14.0,
            color_from_rgba(&planet.color),
        );
        draw_ui_text(
            &truncate_text_to_width(&status_line, text_w, 8.0),
            x + 12.0,
            y + 34.0,
            8.0,
            status_color,
        );
        draw_ui_text(
            &truncate_text_to_width(&format!("Hazard: {}", hazard_summary(planet)), text_w, 7.0),
            x + 12.0,
            y + 47.0,
            7.0,
            hazard_color(planet),
        );
        let compact_detail = if let (Some(order), Some(status)) = (order, route_status) {
            compact_shipping_status(&view.data.planet(order.target).name, status)
        } else if let Some(telemetry) = telemetry {
            format!(
                "{} | M {:.0}/{:.0} D {:.0} B {:.0}",
                core_stage_summary(view.data, &telemetry),
                telemetry.minerals,
                telemetry.mineral_capacity,
                telemetry.data,
                telemetry.biomass
            )
        } else {
            world_rule_lines(view.data, index)
                .first()
                .map(|rule| format!("Rule: {rule}"))
                .unwrap_or_else(|| "Rule: no special build rules".to_string())
        };
        draw_ui_text(
            &truncate_text_to_width(&compact_detail, text_w, 7.0),
            x + 12.0,
            y + 59.0,
            7.0,
            Colors::TEXT_DIM,
        );
        let action = if let Some(action) = mobile_route_action(
            &planet.name,
            order.map(|order| view.data.planet(order.target).name.as_str()),
            route_status,
            is_current,
            order.is_some_and(|order| order.target == view.current_planet),
            is_colonized,
        ) {
            action
        } else if can_launch {
            launch_prompt(view.launch_confirmation_target == Some(index)).to_string()
        } else {
            "World not ready to launch".to_string()
        };
        let action_color = if is_current || is_colonized {
            Colors::PRIMARY
        } else if can_launch {
            Colors::SUCCESS
        } else {
            Colors::ERROR
        };
        draw_ui_text(
            &truncate_text_to_width(&action, text_w, 8.0),
            x + 12.0,
            y + panel_h - 8.0,
            8.0,
            action_color,
        );
        return;
    }
    if screen_h < 560.0 {
        draw_ui_text(
            &truncate_text_to_width(&planet.name, text_w, 16.0),
            x + 15.0,
            y + 25.0,
            16.0,
            color_from_rgba(&planet.color),
        );
        draw_ui_text(
            &truncate_text_to_width(&status_line, text_w, 10.0),
            x + 15.0,
            y + 44.0,
            10.0,
            status_color,
        );
        draw_ui_text(
            &truncate_text_to_width(&format!("Hazard: {}", hazard_summary(planet)), text_w, 8.0),
            x + 15.0,
            y + 59.0,
            8.0,
            hazard_color(planet),
        );
        if let Some(telemetry) = telemetry {
            draw_ui_text(
                &truncate_text_to_width(
                    &format!(
                        "{} | Stock M {:.0}/{:.0} D {:.0} B {:.0}{}",
                        core_stage_summary(view.data, &telemetry),
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
                x + 15.0,
                y + 73.0,
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
                x + 15.0,
                y + 86.0,
                8.0,
                Colors::TEXT,
            );
        }
        let compact_line_y = y + if telemetry.is_some() { 99.0 } else { 73.0 };
        if let (Some(order), Some(status)) = (order, route_status) {
            let route_line = compact_shipping_status(&view.data.planet(order.target).name, status);
            draw_ui_text(
                &truncate_text_to_width(&route_line, text_w, 8.0),
                x + 15.0,
                compact_line_y,
                8.0,
                route_status_color(status),
            );
        } else {
            let rules = world_rule_lines(view.data, index);
            let rule = rules
                .first()
                .map(|rule| format!("Rule: {}", rule))
                .unwrap_or_else(|| "Rule: no special build rules".to_string());
            let rule_color = if rules.len() == 1 && rules[0] == "No special build rules" {
                Colors::SUCCESS
            } else {
                Colors::WARNING
            };
            draw_ui_text(
                &truncate_text_to_width(&rule, text_w, 8.0),
                x + 15.0,
                compact_line_y,
                8.0,
                rule_color,
            );
        }
        let action = if let Some(action) = mobile_route_action(
            &planet.name,
            order.map(|order| view.data.planet(order.target).name.as_str()),
            route_status,
            is_current,
            order.is_some_and(|order| order.target == view.current_planet),
            is_colonized,
        ) {
            action
        } else if can_launch {
            launch_prompt(view.launch_confirmation_target == Some(index)).to_string()
        } else {
            "World not ready to launch".to_string()
        };
        draw_ui_text(
            &truncate_text_to_width(&action, text_w, 9.0),
            x + 15.0,
            y + if telemetry.is_some() { 115.0 } else { 89.0 },
            9.0,
            if is_current || is_colonized {
                Colors::PRIMARY
            } else if can_launch {
                Colors::SUCCESS
            } else {
                Colors::ERROR
            },
        );
        return;
    }
    draw_ui_text(
        &truncate_text_to_width(&planet.name, text_w, 18.0),
        x + 15.0,
        y + 28.0,
        18.0,
        color_from_rgba(&planet.color),
    );
    draw_ui_text(
        &truncate_text_to_width(&planet.description, text_w, 10.0),
        x + 15.0,
        y + 48.0,
        10.0,
        Colors::TEXT,
    );
    draw_ui_text(
        &truncate_text_to_width(&planet.difficulty, text_w, 10.0),
        x + 15.0,
        y + 64.0,
        10.0,
        Colors::TEXT_DIM,
    );
    draw_ui_text(
        &truncate_text_to_width(&status_line, text_w, 11.0),
        x + 15.0,
        y + 86.0,
        11.0,
        status_color,
    );
    draw_ui_text(
        &truncate_text_to_width(
            &format!("Hazards: {}", hazard_summary(planet)),
            text_w,
            10.0,
        ),
        x + 15.0,
        y + 104.0,
        10.0,
        hazard_color(planet),
    );

    let mut next_y = y + 124.0;
    if let Some(telemetry) = telemetry {
        let live = format!(
            "{} | Live O +{:.1}/s  A +{:.1}/s  Parts +{:.1}/s",
            core_stage_summary(view.data, &telemetry),
            telemetry.ore_rate,
            telemetry.alloy_rate,
            telemetry.components_rate
        );
        draw_ui_text(
            &truncate_text_to_width(&live, text_w, 9.0),
            x + 15.0,
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
            x + 15.0,
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
            x + 15.0,
            next_y + 28.0,
            9.0,
            Colors::TEXT,
        );
        next_y += 46.0;
    }
    if let Some(shipping) = shipping.as_deref() {
        draw_ui_text(
            &truncate_text_to_width(shipping, text_w, 9.0),
            x + 15.0,
            next_y,
            9.0,
            Colors::PRIMARY,
        );
        if let Some(status) = route_status {
            draw_ui_text(
                &truncate_text_to_width(&shipping_status_line(status), text_w, 9.0),
                x + 15.0,
                next_y + 14.0,
                9.0,
                route_status_color(status),
            );
        }
        next_y += 32.0;
    }
    draw_ui_text("World rules:", x + 15.0, next_y, 10.0, Colors::TEXT_DIM);
    let rules = world_rule_lines(view.data, index);
    for (rule_index, rule) in rules.iter().take(3).enumerate() {
        draw_ui_text(
            &truncate_text_to_width(rule, text_w, 9.0),
            x + 15.0,
            next_y + 14.0 + rule_index as f32 * 13.0,
            9.0,
            if rule == "No special build rules" {
                Colors::SUCCESS
            } else {
                Colors::WARNING
            },
        );
    }
    let action_y = y + panel_h - 14.0;
    let action = if let Some(action) = mobile_route_action(
        &planet.name,
        order.map(|order| view.data.planet(order.target).name.as_str()),
        route_status,
        is_current,
        order.is_some_and(|order| order.target == view.current_planet),
        is_colonized,
    ) {
        action
    } else if can_launch {
        launch_prompt(view.launch_confirmation_target == Some(index)).to_string()
    } else if !view.mass_driver_researched {
        "Requires Mass Driver research".to_string()
    } else if view.current_drivers_online == 0 {
        "Requires a powered Mass Driver".to_string()
    } else {
        "Requires a finished Seed Ship".to_string()
    };
    draw_ui_text(
        &truncate_text_to_width(&action, text_w, 10.0),
        x + 15.0,
        action_y,
        10.0,
        if is_current || is_colonized {
            Colors::PRIMARY
        } else if can_launch {
            Colors::SUCCESS
        } else {
            Colors::ERROR
        },
    );
}

pub(crate) fn mobile_route_action(
    source_name: &str,
    target_name: Option<&str>,
    status: Option<&str>,
    is_current: bool,
    target_is_current: bool,
    is_colonized: bool,
) -> Option<String> {
    if !is_colonized {
        return None;
    }
    let target_needs_attention = matches!(
        status,
        Some("NO PAD" | "PAD OFFLINE" | "PAD TARGET UNAVAILABLE" | "HOLDING" | "OVERFLOW")
    );
    if target_needs_attention {
        if target_is_current {
            return Some("Current location - tap Back, then BUILD".to_string());
        }
        return target_name.map(|target| format!("Tap {target} row to travel"));
    }
    if is_current {
        Some("Current location".to_string())
    } else {
        Some(format!("Tap {source_name} row to travel"))
    }
}

pub(crate) fn mobile_info_height(screen_h: f32, has_telemetry: bool, has_shipping: bool) -> f32 {
    if screen_h < SHORT_MOBILE_HEIGHT {
        if has_telemetry {
            80.0
        } else {
            68.0
        }
    } else if screen_h < 560.0 {
        if has_telemetry {
            124.0
        } else {
            96.0
        }
    } else {
        250.0 + if has_shipping { 32.0 } else { 0.0 }
    }
}
