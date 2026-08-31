//! Planet list and world dossier panels for the Solar System view.

use crate::ui::{color_from_rgba, draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, note_neighbour, touch_area, Pointer};

use super::core_marker::core_stage_label;
use super::shipping_panel::{route_status_color, route_status_for};
use super::status::{launch_action, launch_prompt};
use super::world_dossier::{
    draw_world_telemetry, hazard_color, hazard_summary, shipping_status_line, shipping_summary,
    world_alert_code, world_alert_color, world_rule_lines, world_telemetry_height,
};
use super::{InterplanetaryAction, MapView, PLANET_INFO_MARGIN, PLANET_INFO_WIDTH};

/// Every world and what is piling up on it. A world left behind keeps working,
/// so the map says so rather than making the player fly there to find out.
/// Rows are controls as well as labels: the list remains usable when an orbit
/// is crowded, off-screen, or too small to comfortably tap.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_planet_list(
    view: &MapView,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    can_launch: bool,
    pointer: Pointer,
    compact: bool,
) -> (Option<usize>, InterplanetaryAction) {
    draw_panel(x, y, w, h);
    draw_ui_text("Planets", x + 12.0, y + 28.0, 16.0, Colors::PRIMARY);
    draw_ui_text("CORE", x + 92.0, y + 28.0, 9.0, Colors::TEXT_DIM);
    draw_ui_text("alerts", x + w - 104.0, y + 28.0, 9.0, Colors::TEXT_DIM);
    draw_ui_text("MIN", x + w - 40.0, y + 28.0, 9.0, Colors::TEXT_DIM);

    let mut focused = None;
    let mut action = InterplanetaryAction::None;
    let row_height = if compact { 19.0 } else { 24.0 };
    let row_top_offset = if compact { 14.0 } else { 18.0 };
    let mut row_y = y + if compact { 45.0 } else { 56.0 };
    for (index, planet) in view.data.planets.iter().enumerate() {
        let is_current = index == view.current_planet;
        let is_colonized = view.colonized.get(index).copied().unwrap_or(false);
        let is_editing = index == view.editing_world
            && view.orders.get(index).is_some_and(|order| order.is_some());
        let row = Rect::new(x + 8.0, row_y - row_top_offset, w - 16.0, row_height);
        note_neighbour(row);
        let hit_row = touch_area(row);
        let is_focused = pointer.hovering_over(hit_row)
            || pointer.pressing(hit_row)
            || pointer.released_on(hit_row);
        if is_focused {
            focused = Some(index);
            let fill = if pointer.pressing(hit_row) {
                Colors::PRIMARY_SOFT
            } else {
                Color::new(0.0, 0.7, 0.9, 0.12)
            };
            draw_rectangle(row.x, row.y, row.w, row.h, fill);
        } else if view.launch_confirmation_target == Some(index) {
            draw_rectangle(row.x, row.y, row.w, row.h, Color::new(1.0, 0.45, 0.1, 0.14));
        }
        if is_editing {
            draw_rectangle_lines(row.x, row.y, row.w, row.h, 1.0, Colors::ACCENT);
        }
        let confirming = view.launch_confirmation_target == Some(index);
        let label_color = if is_current {
            Colors::PRIMARY
        } else if is_editing {
            Colors::ACCENT
        } else if is_colonized {
            Colors::SUCCESS
        } else if confirming {
            Colors::WARNING
        } else {
            Colors::TEXT_DIM
        };
        draw_ui_text(&planet.name, x + 12.0, row_y, 13.0, label_color);
        let telemetry = view.telemetry.get(index).copied().flatten();
        if let Some(core_label) = core_stage_label(telemetry) {
            draw_ui_text(&core_label, x + 92.0, row_y, 9.0, Colors::PRIMARY);
        }
        let alert_code = if confirming {
            "CONFIRM".to_string()
        } else {
            world_alert_code(telemetry)
        };
        draw_ui_text(
            &alert_code,
            x + w - 104.0,
            row_y,
            9.0,
            if confirming {
                Colors::WARNING
            } else {
                world_alert_color(telemetry)
            },
        );
        if let Some(Some(minerals)) = view.stockpiles.get(index) {
            draw_ui_text(
                &format!("{:.0}", minerals),
                x + w - 50.0,
                row_y,
                12.0,
                Colors::ACCENT,
            );
        }
        if pointer.released_on(hit_row) {
            if is_colonized && !is_current {
                action = InterplanetaryAction::SelectPlanet(index);
            } else if !is_colonized && can_launch {
                action = launch_action(index, view.launch_confirmation_target == Some(index));
            }
        }
        row_y += row_height;
    }
    (focused, action)
}

/// What the hovered world is, and what clicking it would do.
pub(super) fn draw_planet_info(
    view: &MapView,
    index: usize,
    can_launch: bool,
    screen_w: f32,
    header_height: f32,
    activated: bool,
) -> Option<InterplanetaryAction> {
    let planet = &view.data.planets[index];
    let is_colonized = view.colonized.get(index).copied().unwrap_or(false);
    let is_current = index == view.current_planet;
    let shipping = shipping_summary(
        view.data,
        view.orders.get(index).copied().flatten(),
        view.export_rates.get(index).copied().unwrap_or(0.0),
    );
    let route_status = view
        .orders
        .get(index)
        .copied()
        .flatten()
        .map(|order| route_status_for(view, index, order));

    let panel_w = PLANET_INFO_WIDTH;
    let shipping_offset = if shipping.is_some() { 36.0 } else { 0.0 };
    let panel_h = 300.0 + shipping_offset;
    let panel_x = screen_w - panel_w - PLANET_INFO_MARGIN;
    let panel_y = header_height + 12.0;
    draw_panel(panel_x, panel_y, panel_w, panel_h);

    draw_ui_text(
        &planet.name,
        panel_x + 15.0,
        panel_y + 30.0,
        20.0,
        color_from_rgba(&planet.color),
    );
    draw_ui_text(
        &planet.description,
        panel_x + 15.0,
        panel_y + 55.0,
        13.0,
        Colors::TEXT,
    );
    draw_ui_text(
        &planet.difficulty,
        panel_x + 15.0,
        panel_y + 75.0,
        12.0,
        Colors::TEXT_DIM,
    );

    let (status, status_color) = if is_colonized {
        ("Status: COLONIZED", Colors::SUCCESS)
    } else {
        ("Status: Unexplored", Colors::WARNING)
    };
    draw_ui_text(status, panel_x + 15.0, panel_y + 105.0, 13.0, status_color);

    draw_ui_text(
        &format!("Hazards: {}", hazard_summary(planet)),
        panel_x + 15.0,
        panel_y + 128.0,
        11.0,
        hazard_color(planet),
    );

    let telemetry = view.telemetry.get(index).copied().flatten();
    if let Some(telemetry) = telemetry {
        draw_world_telemetry(view.data, &telemetry, panel_x + 15.0, panel_y + 147.0);
    }

    let info_end = if telemetry.is_some() {
        147.0 + world_telemetry_height()
    } else {
        149.0
    };
    if let Some(shipping) = shipping.as_deref() {
        draw_ui_text(
            shipping,
            panel_x + 15.0,
            panel_y + info_end,
            10.0,
            Colors::PRIMARY,
        );
        if let Some(status) = route_status {
            draw_ui_text(
                &shipping_status_line(status),
                panel_x + 15.0,
                panel_y + info_end + 16.0,
                10.0,
                route_status_color(status),
            );
        }
    }
    let rules_y = panel_y + info_end + shipping_offset;
    draw_ui_text(
        "World rules:",
        panel_x + 15.0,
        rules_y,
        11.0,
        Colors::TEXT_DIM,
    );
    let rules = world_rule_lines(view.data, index);
    let rules_color = if rules.len() == 1 && rules[0] == "No special build rules" {
        Colors::SUCCESS
    } else {
        Colors::WARNING
    };
    for (line_index, line) in rules.iter().enumerate() {
        draw_ui_text(
            line,
            panel_x + 15.0,
            rules_y + 16.0 + line_index as f32 * 16.0,
            10.0,
            rules_color,
        );
    }

    let action_y = panel_y + if telemetry.is_some() { 289.0 } else { 213.0 } + shipping_offset;
    if is_current {
        draw_ui_text(
            "(Current Location)",
            panel_x + 15.0,
            action_y,
            11.0,
            Colors::PRIMARY,
        );
    } else if is_colonized {
        draw_ui_text(
            "Tap to travel",
            panel_x + 15.0,
            action_y,
            12.0,
            Colors::PRIMARY,
        );
    } else if can_launch {
        draw_ui_text(
            launch_prompt(view.launch_confirmation_target == Some(index)),
            panel_x + 15.0,
            action_y,
            12.0,
            Colors::SUCCESS,
        );
    } else {
        let blocker = if !view.mass_driver_researched {
            "Requires: Mass Driver research"
        } else if view.current_drivers_online == 0 {
            "Requires: a powered Mass Driver"
        } else {
            "Requires: a finished Seed Ship"
        };
        draw_ui_text(blocker, panel_x + 15.0, action_y, 11.0, Colors::ERROR);
    }

    if !activated {
        return None;
    }
    if is_colonized && !is_current {
        Some(InterplanetaryAction::SelectPlanet(index))
    } else if can_launch && !is_colonized {
        Some(launch_action(
            index,
            view.launch_confirmation_target == Some(index),
        ))
    } else {
        None
    }
}

pub(super) fn draw_legend(screen_w: f32, screen_h: f32) {
    draw_panel(screen_w - 190.0, screen_h - 130.0, 180.0, 120.0);
    draw_circle_lines(
        screen_w - 175.0,
        screen_h - 110.0,
        6.0,
        1.0,
        Colors::PRIMARY,
    );
    draw_ui_text(
        "Core C1-C5",
        screen_w - 160.0,
        screen_h - 105.0,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_circle(screen_w - 175.0, screen_h - 85.0, 6.0, Colors::SUCCESS);
    draw_ui_text(
        "Colonized",
        screen_w - 160.0,
        screen_h - 80.0,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_circle(screen_w - 175.0, screen_h - 65.0, 6.0, Colors::WARNING);
    draw_ui_text(
        "Unexplored",
        screen_w - 160.0,
        screen_h - 60.0,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_circle(screen_w - 175.0, screen_h - 45.0, 6.0, Colors::PRIMARY);
    draw_ui_text(
        "Current",
        screen_w - 160.0,
        screen_h - 40.0,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_line(
        screen_w - 181.0,
        screen_h - 25.0,
        screen_w - 169.0,
        screen_h - 25.0,
        2.0,
        Colors::ACCENT,
    );
    draw_ui_text(
        "Freight lane",
        screen_w - 160.0,
        screen_h - 20.0,
        12.0,
        Colors::TEXT_DIM,
    );
}
