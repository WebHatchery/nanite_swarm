//! Bottom command bar: alerts, controls legend, mission time, speed, and help overlay

use crate::data::UiTheme;
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::metrics::HudMetrics;
use super::PanelColors;

/// What the player asked the clock to do. The bar reads state and returns
/// intent; the caller applies it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum ClockAction {
    None,
    TogglePause,
    Faster,
    Slower,
}

#[must_use]
pub(super) fn draw(
    state: &PlanetState,
    screen_w: f32,
    screen_h: f32,
    theme: &UiTheme,
    metrics: HudMetrics,
    colors: &PanelColors,
) -> ClockAction {
    let text = colors.text;
    let dim = colors.dim;
    let primary = colors.primary;
    let primary_soft = colors.primary_soft;
    let warning = colors.warning;
    let success = colors.success;
    let error = colors.error;

    let bottom_y = screen_h - metrics.bottom_bar_height;
    draw_hud_panel(
        theme,
        Rect::new(0.0, bottom_y, screen_w, metrics.bottom_bar_height),
        None,
    );
    let bottom_gap = if screen_w < 1160.0 { 8.0 } else { 12.0 };
    let alert_w = (screen_w * 0.18).clamp(188.0, 294.0);
    let mut status_w = (screen_w * 0.30).clamp(300.0, 452.0);
    if screen_w < 1050.0 {
        status_w = (screen_w * 0.31).clamp(278.0, 330.0);
    }
    draw_hud_panel(
        theme,
        Rect::new(
            10.0,
            bottom_y + 8.0,
            alert_w,
            metrics.bottom_bar_height - 16.0,
        ),
        None,
    );
    let congested = state.congested_tiles();
    let alert_count = i32::from(state.power_balance < 0.0)
        + i32::from(state.battery_seconds <= 0.0)
        + i32::from(state.power_collapse_shutdown > 0.0)
        + i32::from(congested > 0)
        + i32::from(state.stalled_drone_count() > 0);
    draw_ui_text("ALERTS", 30.0, bottom_y + 31.0, 12.0, warning);
    draw_ui_text(
        &format!("{}", alert_count),
        52.0,
        bottom_y + metrics.bottom_bar_height - 17.0,
        if metrics.bottom_bar_height < 70.0 {
            18.0
        } else {
            24.0
        },
        warning,
    );
    let alert_text = if state.power_collapse_shutdown > 0.0 {
        "POWER COLLAPSE"
    } else if state.battery_seconds <= 0.0 {
        "LOW BATTERY"
    } else if state.power_balance < 0.0 {
        "NEGATIVE POWER"
    } else if state.stalled_drone_count() > 0 {
        "ROUTE SEVERED"
    } else if congested > 0 {
        "TRAFFIC SATURATED"
    } else {
        "SYSTEM NOMINAL"
    };
    draw_ui_text(
        alert_text,
        106.0,
        bottom_y + 35.0,
        11.0,
        if alert_count > 0 { error } else { success },
    );

    let controls_x = 10.0 + alert_w + bottom_gap;
    let status_x = screen_w - status_w - 10.0;
    let controls_w = (status_x - controls_x - bottom_gap).max(0.0);
    draw_hud_panel(
        theme,
        Rect::new(
            controls_x,
            bottom_y + 8.0,
            controls_w,
            metrics.bottom_bar_height - 16.0,
        ),
        None,
    );
    let control_y = bottom_y + 30.0;
    let controls = if controls_w < 420.0 {
        vec![
            ("SELECT", "Left Click"),
            ("PAN", "Drag"),
            ("ZOOM", "Wheel"),
            ("PAUSE", "Space"),
        ]
    } else if controls_w < 560.0 {
        vec![
            ("SELECT", "Left Click"),
            ("PAN", "Middle Drag"),
            ("ZOOM", "Wheel"),
            ("BUILD", "B"),
            ("PAUSE", "Space"),
        ]
    } else {
        vec![
            ("SELECT", "Left Click"),
            ("PAN", "Middle Drag"),
            ("BOX SELECT", "Shift + Drag"),
            ("ZOOM", "Mouse Wheel"),
            ("BUILD MENU", "B"),
            ("DEMOLISH", "X"),
            ("PAUSE", "Space"),
        ]
    };
    let slot_w = controls_w / controls.len() as f32;
    for (index, (label, hint)) in controls.iter().enumerate() {
        let x = controls_x + index as f32 * slot_w + 12.0;
        draw_ui_text(label, x, control_y, 10.0, text);
        draw_ui_text(hint, x, control_y + 18.0, 9.0, dim);
        if index > 0 {
            let divider_x = controls_x + index as f32 * slot_w;
            draw_line(
                divider_x,
                bottom_y + 16.0,
                divider_x,
                bottom_y + metrics.bottom_bar_height - 16.0,
                1.0,
                color_from_rgba(&theme.colors.border),
            );
        }
    }

    draw_hud_panel(
        theme,
        Rect::new(
            status_x,
            bottom_y + 8.0,
            status_w,
            metrics.bottom_bar_height - 16.0,
        ),
        None,
    );
    let time_seconds = state.time_played.max(0.0) as i32;
    let time_h = time_seconds / 3600;
    let time_m = (time_seconds % 3600) / 60;
    let time_s = time_seconds % 60;
    draw_ui_text("MISSION TIME", status_x + 22.0, bottom_y + 27.0, 9.0, dim);
    draw_ui_text(
        &format!("{:02}:{:02}:{:02}", time_h, time_m, time_s),
        status_x + 22.0,
        bottom_y + 52.0,
        16.0,
        text,
    );
    let mut clock = ClockAction::None;
    let speed_x = status_x + status_w * 0.36;
    draw_ui_text("GAME SPEED", speed_x, bottom_y + 27.0, 9.0, dim);
    // The readout sits between the two buttons, so a long word has to come
    // down in size rather than run over them.
    let (speed_label, speed_color, speed_size) = if state.paused {
        ("PAUSED".to_string(), colors.warning, 12.0)
    } else {
        (format!("{:.1}x", state.time_scale), text, 16.0)
    };
    draw_ui_text(
        &speed_label,
        speed_x + 10.0,
        bottom_y + 52.0,
        speed_size,
        speed_color,
    );
    if draw_hud_button(
        theme,
        Rect::new(speed_x - 30.0, bottom_y + 34.0, 24.0, 24.0),
        "-",
    ) {
        clock = ClockAction::Slower;
    }
    if draw_hud_button(
        theme,
        Rect::new(speed_x + 58.0, bottom_y + 34.0, 24.0, 24.0),
        "+",
    ) {
        clock = ClockAction::Faster;
    }
    if draw_hud_button(
        theme,
        Rect::new(speed_x + 86.0, bottom_y + 34.0, 24.0, 24.0),
        if state.paused { ">" } else { "II" },
    ) {
        clock = ClockAction::TogglePause;
    }

    let graph_x = status_x + status_w * 0.66;
    let graph_y = bottom_y + 22.0;
    let graph_w = status_x + status_w - 18.0 - graph_x;
    let graph_h = if metrics.bottom_bar_height < 70.0 {
        28.0
    } else {
        34.0
    };
    draw_rectangle(
        graph_x,
        graph_y,
        graph_w,
        graph_h,
        color_from_rgba(&theme.colors.panel_deep),
    );
    draw_rectangle_lines(
        graph_x,
        graph_y,
        graph_w,
        graph_h,
        1.0,
        color_from_rgba(&theme.colors.border),
    );
    for i in 1..24 {
        let x0 = graph_x + (i - 1) as f32 * graph_w / 23.0;
        let x1 = graph_x + i as f32 * graph_w / 23.0;
        let y0 =
            graph_y + graph_h * (0.55 + (state.time_played as f32 * 0.1 + i as f32).sin() * 0.18);
        let y1 = graph_y
            + graph_h * (0.55 + (state.time_played as f32 * 0.1 + i as f32 + 1.0).sin() * 0.18);
        draw_line(x0, y0, x1, y1, 1.0, primary);
    }

    let mode_y = bottom_y + metrics.bottom_bar_height - 8.0;
    draw_ui_text(
        if state.selected_building.is_some() {
            "BUILD MODE"
        } else {
            "SELECT MODE"
        },
        controls_x + 12.0,
        mode_y,
        9.0,
        primary_soft,
    );
    if let Some(selected) = state.selected_building {
        draw_ui_text(selected.name(), controls_x + 86.0, mode_y, 9.0, text);
    }

    // Help overlay
    if state.show_help {
        let help_w = 360.0;
        let help_h = 200.0;
        let help_x = screen_w - help_w - 20.0;
        let help_y = 90.0;
        draw_hud_panel(
            theme,
            Rect::new(help_x, help_y, help_w, help_h),
            Some("HELP & CONTROLS"),
        );
        draw_ui_text(
            "Left Click / Drag: Place building",
            help_x + 16.0,
            help_y + 55.0,
            14.0,
            text,
        );
        draw_ui_text(
            "Right Click: Cancel selection / Harvest",
            help_x + 16.0,
            help_y + 75.0,
            14.0,
            text,
        );
        draw_ui_text(
            "H: Harvest terrain",
            help_x + 16.0,
            help_y + 95.0,
            14.0,
            text,
        );
        draw_ui_text(
            "R: Research  |  M: Map",
            help_x + 16.0,
            help_y + 115.0,
            14.0,
            text,
        );
        draw_ui_text(
            "1-9: Select buildings",
            help_x + 16.0,
            help_y + 135.0,
            14.0,
            text,
        );
        draw_ui_text(
            "F: Convert forest to filter",
            help_x + 16.0,
            help_y + 155.0,
            14.0,
            text,
        );
        draw_ui_text(
            "Space: Pause  |  F1: Toggle help",
            help_x + 16.0,
            help_y + 175.0,
            14.0,
            dim,
        );
    }

    clock
}
