//! Mission clock, speed controls, and telemetry graph for the desktop bar.

use crate::data::UiTheme;
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::super::super::metrics::HudMetrics;
use super::super::super::PanelColors;
use super::super::{graph, ClockAction};

pub(super) fn draw(
    state: &PlanetState,
    bottom_y: f32,
    status_x: f32,
    status_w: f32,
    theme: &UiTheme,
    metrics: HudMetrics,
    colors: &PanelColors,
) -> ClockAction {
    let text = colors.text;
    let dim = colors.dim;
    let primary = colors.primary;
    let mut clock = ClockAction::None;

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
    // The saved marker borrows the mission-time slot for a few seconds; it is
    // the one label the player is not reading second to second.
    if state.save_notice_timer > 0.0 {
        draw_ui_text(
            "SAVED",
            status_x + 22.0,
            bottom_y + 27.0,
            9.0,
            colors.success,
        );
    } else {
        draw_ui_text("MISSION TIME", status_x + 22.0, bottom_y + 27.0, 9.0, dim);
    }
    draw_ui_text(
        &format!("{:02}:{:02}:{:02}", time_h, time_m, time_s),
        status_x + 22.0,
        bottom_y + 52.0,
        16.0,
        text,
    );
    let speed_x = status_x + status_w * 0.36;
    draw_ui_text("GAME SPEED", speed_x, bottom_y + 27.0, 9.0, dim);
    // The readout sits between the two buttons, so a long word has to come
    // down in size rather than run over them.
    let (speed_label, speed_color, speed_size) = if state.paused {
        ("PAUSED".to_string(), colors.warning, 12.0)
    } else {
        (
            if state.time_scale >= 8.0 {
                "8.0x MAX".to_string()
            } else {
                format!("{:.1}x", state.time_scale)
            },
            text,
            16.0,
        )
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
    if graph::draw_throughput(
        state,
        Rect::new(graph_x, graph_y, graph_w, graph_h),
        theme,
        primary,
        text,
    ) {
        clock = ClockAction::CycleGraph;
    }
    clock
}
