//! Touch-first solar-system sections for portrait screens.

use super::shipping_panel::{route_status_color, route_status_for};
use super::world_dossier::{
    compact_shipping_status, core_stage_summary, shipping_status_line, storage_resource_label,
};
use super::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width, Pointer};

const MOBILE_EDGE: f32 = 12.0;
const MOBILE_TAB_Y: f32 = 112.0;
const MOBILE_CONTENT_Y: f32 = 156.0;
const SHORT_MOBILE_HEIGHT: f32 = 440.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct MobileWorldsLayout {
    list_h: f32,
    row_height: f32,
    info_gap: f32,
    compact: bool,
}

fn mobile_status_text(is_colonized: bool, alert_code: Option<&str>) -> String {
    let status = if is_colonized {
        "Status: COLONIZED"
    } else {
        "Status: UNEXPLORED"
    };
    alert_code
        .map(|code| format!("{status} / {code}"))
        .unwrap_or_else(|| status.to_string())
}

pub(super) fn draw(
    view: &MapView,
    shipping_scroll: &mut ScrollArea,
    panel: InterplanetaryMobilePanel,
    screen_h: f32,
    can_launch: bool,
) -> InterplanetaryAction {
    let screen_w = screen_width();
    let pointer = Pointer::read(|position| position);
    let action = draw_tabs(panel, screen_w);
    if action != InterplanetaryAction::None {
        if action == InterplanetaryAction::ToggleMobileWorlds {
            shipping_scroll.set_offset(0.0);
        }
        return action;
    }

    let action = match panel {
        InterplanetaryMobilePanel::Worlds => {
            draw_worlds(view, can_launch, pointer, screen_w, screen_h)
        }
        InterplanetaryMobilePanel::Shipping => shipping_panel::draw(
            view,
            MOBILE_EDGE,
            MOBILE_CONTENT_Y,
            (screen_w - MOBILE_EDGE * 2.0).max(80.0),
            (screen_h - MOBILE_CONTENT_Y - 32.0).max(120.0),
            shipping_scroll,
        ),
    };

    let footer_hint = truncate_text_to_width(
        mobile_footer_hint_text(panel, view.launch_confirmation_target.is_some(), screen_w),
        (screen_w - MOBILE_EDGE * 2.0).max(40.0),
        10.0,
    );
    draw_ui_text(
        &footer_hint,
        MOBILE_EDGE,
        screen_h - 18.0,
        10.0,
        Colors::TEXT_DIM,
    );
    if is_key_pressed(KeyCode::Escape) {
        return InterplanetaryAction::Close;
    }
    action
}

fn mobile_footer_hint_text(
    panel: InterplanetaryMobilePanel,
    confirming: bool,
    screen_w: f32,
) -> &'static str {
    if screen_w < 360.0 {
        match (panel, confirming) {
            (InterplanetaryMobilePanel::Worlds, true) => "Tap CONFIRM | SHIPPING for routes",
            (InterplanetaryMobilePanel::Worlds, false) => "Tap world | SHIPPING for routes",
            (InterplanetaryMobilePanel::Shipping, _) => "Drag to scroll | WORLDS to return",
        }
    } else {
        match (panel, confirming) {
            (InterplanetaryMobilePanel::Worlds, true) => {
                "Tap CONFIRM again to launch | Tap Shipping for routes"
            }
            (InterplanetaryMobilePanel::Worlds, false) => {
                "Tap a world to travel | Tap Shipping for routes"
            }
            (InterplanetaryMobilePanel::Shipping, _) => {
                "Drag panel to scroll | Tap controls | Tap Worlds to return"
            }
        }
    }
}

fn draw_tabs(panel: InterplanetaryMobilePanel, screen_w: f32) -> InterplanetaryAction {
    let tab_w = ((screen_w - MOBILE_EDGE * 3.0) * 0.5).max(60.0);
    let shipping_x = MOBILE_EDGE * 2.0 + tab_w;
    let worlds = draw_button_sized(MOBILE_EDGE, MOBILE_TAB_Y, tab_w, 32.0, "");
    let shipping = draw_button_sized(shipping_x, MOBILE_TAB_Y, tab_w, 32.0, "");
    draw_tab_label(
        "WORLDS",
        MOBILE_EDGE,
        tab_w,
        panel == InterplanetaryMobilePanel::Worlds,
    );
    draw_tab_label(
        "SHIPPING",
        shipping_x,
        tab_w,
        panel == InterplanetaryMobilePanel::Shipping,
    );
    if worlds && panel != InterplanetaryMobilePanel::Worlds {
        return InterplanetaryAction::ToggleMobileWorlds;
    }
    if shipping && panel != InterplanetaryMobilePanel::Shipping {
        return InterplanetaryAction::ToggleMobileShipping;
    }
    InterplanetaryAction::None
}

fn draw_tab_label(label: &str, x: f32, width: f32, selected: bool) {
    let text_width = measure_ui_text(label, None, 11, 1.0).width;
    draw_ui_text(
        label,
        x + (width - text_width) * 0.5,
        MOBILE_TAB_Y + 21.0,
        11.0,
        if selected {
            Colors::PRIMARY
        } else {
            Colors::TEXT
        },
    );
}

fn draw_worlds(
    view: &MapView,
    can_launch: bool,
    pointer: Pointer,
    screen_w: f32,
    screen_h: f32,
) -> InterplanetaryAction {
    let panel_w = (screen_w - MOBILE_EDGE * 2.0).max(80.0);
    let layout = mobile_worlds_layout(screen_h, view.data.planets.len());
    let (focused, list_action) = draw_planet_list(
        view,
        MOBILE_EDGE,
        MOBILE_CONTENT_Y,
        panel_w,
        layout.list_h,
        can_launch,
        pointer,
        layout.compact,
    );
    let index = focused
        .or(view.launch_confirmation_target)
        .unwrap_or(view.current_planet);
    let info_y = MOBILE_CONTENT_Y + layout.list_h + layout.info_gap;
    draw_mobile_planet_info(
        view,
        index,
        can_launch,
        MOBILE_EDGE,
        info_y,
        panel_w,
        screen_h,
    );
    list_action
}

fn mobile_worlds_layout(screen_h: f32, planet_count: usize) -> MobileWorldsLayout {
    let compact = screen_h < SHORT_MOBILE_HEIGHT;
    let row_height = if compact { 19.0 } else { 24.0 };
    let list_header_height = if compact { 34.0 } else { 44.0 };
    MobileWorldsLayout {
        list_h: list_header_height + planet_count as f32 * row_height,
        row_height,
        info_gap: if compact { 6.0 } else { 10.0 },
        compact,
    }
}

fn draw_mobile_planet_info(
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
        .map(|telemetry| super::world_alert_code(Some(telemetry)));
    let status_line = mobile_status_text(is_colonized, alert_code.as_deref());
    let status_color = if alert_code.is_some() {
        Colors::ERROR
    } else if is_colonized {
        Colors::SUCCESS
    } else {
        Colors::WARNING
    };
    let order = view.orders.get(index).copied().flatten();
    let shipping = super::shipping_summary(
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
                "{} | M {:.0} D {:.0} B {:.0}",
                core_stage_summary(view.data, &telemetry),
                telemetry.minerals,
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
        let action = if is_current {
            "Current location"
        } else if is_colonized {
            "Tap world row to travel"
        } else if can_launch {
            super::launch_prompt(view.launch_confirmation_target == Some(index))
        } else {
            "World not ready to launch"
        };
        let action_color = if is_current || is_colonized {
            Colors::PRIMARY
        } else if can_launch {
            Colors::SUCCESS
        } else {
            Colors::ERROR
        };
        draw_ui_text(
            &truncate_text_to_width(action, text_w, 8.0),
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
                        "{} | Stock M {:.0} D {:.0} B {:.0}{}",
                        core_stage_summary(view.data, &telemetry),
                        telemetry.minerals,
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
        let action = if is_current {
            "Current location"
        } else if is_colonized {
            "Tap world row to travel"
        } else if can_launch {
            super::launch_prompt(view.launch_confirmation_target == Some(index))
        } else {
            "World not ready to launch"
        };
        draw_ui_text(
            &truncate_text_to_width(action, text_w, 9.0),
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
                    "Stock M {:.0} D {:.0} B {:.0}{}  Grid {:+.1}/s",
                    telemetry.minerals,
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
    let action = if is_current {
        "(Current location)"
    } else if is_colonized {
        "Tap world row to travel"
    } else if can_launch {
        super::launch_prompt(view.launch_confirmation_target == Some(index))
    } else if !view.mass_driver_researched {
        "Requires Mass Driver research"
    } else if view.current_drivers_online == 0 {
        "Requires a powered Mass Driver"
    } else {
        "Requires a finished Seed Ship"
    };
    draw_ui_text(
        &truncate_text_to_width(action, text_w, 10.0),
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

fn mobile_info_height(screen_h: f32, has_telemetry: bool, has_shipping: bool) -> f32 {
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

#[cfg(test)]
mod tests;
