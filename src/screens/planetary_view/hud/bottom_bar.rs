//! Bottom command bar: alerts, controls legend, mission time, speed, and help overlay

use crate::data::UiTheme;
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
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
    NextEvent,
    ToggleFocus,
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
    let alert_count = i32::from(state.save_failed)
        + i32::from(state.restored_from_backup)
        + i32::from(state.power_balance < 0.0)
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
    let alert_text = if state.save_failed {
        "SAVE FAILED"
    } else if state.restored_from_backup {
        "RESTORED FROM BACKUP"
    } else if state.power_collapse_shutdown > 0.0 {
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
            ("SELECT", "Tap grid"),
            ("PAN", "Drag grid"),
            ("ZOOM", "Pinch grid"),
            ("PAUSE", "Tap II"),
        ]
    } else if controls_w < 560.0 {
        vec![
            ("SELECT", "Tap grid"),
            ("PAN", "Drag grid"),
            ("ZOOM", "Pinch grid"),
            ("BUILD", "Tap card"),
            ("PAUSE", "Tap II"),
        ]
    } else {
        vec![
            ("SELECT", "Tap grid"),
            ("PAN", "Drag grid"),
            ("ZOOM", "Pinch grid"),
            ("BUILD", "Tap card"),
            ("HARVEST", "Tap inspector"),
            ("DEMOLISH", "Tap button"),
            ("PAUSE", "Tap II"),
        ]
    };
    let text_controls_w = (controls_w - 130.0).max(120.0);
    let slot_w = text_controls_w / controls.len() as f32;
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
    let mut clock = ClockAction::None;
    let next_event = state.next_interesting_event();
    let next_label = next_event
        .map(|event| format!("NEXT {:.0}s", event.seconds))
        .unwrap_or_else(|| "NEXT".to_string());
    if draw_hud_button(
        theme,
        Rect::new(controls_x + controls_w - 60.0, bottom_y + 10.0, 52.0, 22.0),
        &next_label,
    ) {
        clock = ClockAction::NextEvent;
    }
    if draw_hud_button(
        theme,
        Rect::new(controls_x + controls_w - 120.0, bottom_y + 10.0, 54.0, 22.0),
        if state.focus_mode { "PANELS" } else { "FOCUS" },
    ) {
        clock = ClockAction::ToggleFocus;
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
    draw_throughput(
        state,
        Rect::new(graph_x, graph_y, graph_w, graph_h),
        theme,
        primary,
        text,
    );

    let mode_y = bottom_y + metrics.bottom_bar_height - 8.0;
    let (mode_label, mode_color) = if state.demolish_mode {
        ("DEMOLISH MODE", colors.error)
    } else if state.selected_building.is_some() {
        ("BUILD MODE", primary_soft)
    } else {
        ("SELECT MODE", primary_soft)
    };
    draw_ui_text(mode_label, controls_x + 12.0, mode_y, 9.0, mode_color);
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
            "Tap grid: build or inspect",
            help_x + 16.0,
            help_y + 55.0,
            14.0,
            text,
        );
        draw_ui_text(
            "Drag grid: pan the map",
            help_x + 16.0,
            help_y + 75.0,
            14.0,
            text,
        );
        draw_ui_text(
            "Pinch grid: zoom the map",
            help_x + 16.0,
            help_y + 95.0,
            14.0,
            text,
        );
        draw_ui_text(
            "Tap inspector: harvest, filter, or sell",
            help_x + 16.0,
            help_y + 115.0,
            14.0,
            text,
        );
        draw_ui_text(
            "Tap CANCEL: leave build or demolish mode",
            help_x + 16.0,
            help_y + 135.0,
            14.0,
            text,
        );
        draw_ui_text(
            "Top buttons open Research, Ship, Map, Menu",
            help_x + 16.0,
            help_y + 155.0,
            14.0,
            text,
        );
        draw_ui_text(
            "Tap HELP again to close",
            help_x + 16.0,
            help_y + 175.0,
            14.0,
            dim,
        );
    }

    clock
}

/// Ore banked at the Core per second, across the whole session.
///
/// This was a sine wave for a long time: an automation game with a decorative
/// graph of nothing. Each bucket keeps the range it covers, so a spike stays
/// visible however far back it happened.
fn draw_throughput(state: &PlanetState, area: Rect, theme: &UiTheme, line: Color, text: Color) {
    let buckets = state.throughput.buckets();
    if buckets.is_empty() {
        draw_ui_text(
            "NO DELIVERIES YET",
            area.x + 6.0,
            area.y + area.h * 0.62,
            9.0,
            text,
        );
        return;
    }

    // Always measured against zero, so a flat line at the bottom reads as
    // "nothing is arriving" rather than being stretched to fill the box.
    let graph_peak = state
        .graph_samples
        .iter()
        .flat_map(|sample| {
            [
                sample.minerals_consumed,
                sample.alloy_produced,
                sample.alloy_consumed,
                sample.components_produced,
                sample.components_consumed,
            ]
        })
        .fold(0.0, f32::max);
    let peak = state
        .throughput
        .max()
        .unwrap_or(0.0)
        .max(graph_peak)
        .max(0.001);
    let plot_h = area.h - 10.0;
    let step = area.w / buckets.len().max(2) as f32;

    for (index, bucket) in buckets.iter().enumerate() {
        let x = area.x + index as f32 * step;
        // The bucket's whole range, so a merged spike is still a spike.
        let low = area.y + area.h - 2.0 - (bucket.min / peak).clamp(0.0, 1.0) * plot_h;
        let high = area.y + area.h - 2.0 - (bucket.max / peak).clamp(0.0, 1.0) * plot_h;
        draw_line(x, low, x, high, step.max(1.0), with_alpha(line, 0.35));
        if index > 0 {
            let previous = buckets[index - 1].last;
            let y0 = area.y + area.h - 2.0 - (previous / peak).clamp(0.0, 1.0) * plot_h;
            let y1 = area.y + area.h - 2.0 - (bucket.last / peak).clamp(0.0, 1.0) * plot_h;
            draw_line(x - step, y0, x, y1, 1.0, line);
        }
    }

    let latest = state.throughput.last().unwrap_or(0.0);
    draw_ui_text(
        &format!(
            "IN {:.1}  A {:.1}  C {:.1}",
            latest,
            state.observed_alloy_rate(),
            state.observed_components_rate()
        ),
        area.x + 4.0,
        area.y + 9.0,
        9.0,
        text,
    );
    if !state.graph_samples.is_empty() {
        draw_graph_series(state, area, peak, theme, line);
    }
    let legend = [
        ("IN", line),
        ("O>", color_from_rgba(&theme.colors.minerals)),
        ("A+", color_from_rgba(&theme.colors.alloy)),
        ("A>", color_from_rgba(&theme.colors.warning)),
        ("C+", color_from_rgba(&theme.colors.components)),
    ];
    let legend_step = (area.w - 8.0) / legend.len() as f32;
    for (index, (label, color)) in legend.into_iter().enumerate() {
        draw_ui_text(
            label,
            area.x + 4.0 + index as f32 * legend_step,
            area.bottom() - 3.0,
            8.0,
            color,
        );
    }
}

fn draw_graph_series(state: &PlanetState, area: Rect, peak: f32, theme: &UiTheme, line: Color) {
    let count = state.graph_samples.len().max(2) as f32;
    let step = area.w / count;
    let series = [
        (0usize, color_from_rgba(&theme.colors.minerals)),
        (1, color_from_rgba(&theme.colors.alloy)),
        (2, color_from_rgba(&theme.colors.warning)),
        (3, color_from_rgba(&theme.colors.components)),
    ];
    for (kind, color) in series {
        for (index, pair) in state.graph_samples.windows(2).enumerate() {
            let x0 = area.x + index as f32 * step;
            let x1 = x0 + step;
            let read = |sample: &crate::state::GraphSample| match kind {
                0 => sample.minerals_consumed,
                1 => sample.alloy_produced,
                2 => sample.alloy_consumed,
                _ => sample.components_produced,
            };
            let y0 =
                area.y + area.h - 2.0 - (read(&pair[0]) / peak).clamp(0.0, 1.0) * (area.h - 10.0);
            let y1 =
                area.y + area.h - 2.0 - (read(&pair[1]) / peak).clamp(0.0, 1.0) * (area.h - 10.0);
            draw_line(x0, y0, x1, y1, 0.7, with_alpha(color, 0.55));
        }
    }
    draw_line(
        area.x + 4.0,
        area.bottom() - 5.0,
        area.x + 14.0,
        area.bottom() - 5.0,
        1.0,
        line,
    );
}
