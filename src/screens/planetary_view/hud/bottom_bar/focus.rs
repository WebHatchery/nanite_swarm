//! Compact bottom edge controls for the unobstructed planetary map.

use crate::data::UiTheme;
use crate::state::{DesktopPanel, PlanetState};
use crate::ui::{draw_hud_button, draw_hud_panel};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::super::metrics::HudMetrics;
use super::super::PanelColors;
use super::alerts::{alert_button, has_flow_alert, has_pending_interaction, next_event_code};
use super::{zoom_button_rects, ClockAction};

pub(super) fn draw(
    state: &PlanetState,
    screen_w: f32,
    screen_h: f32,
    theme: &UiTheme,
    metrics: HudMetrics,
    colors: &PanelColors,
) -> ClockAction {
    let bottom_y = screen_h - metrics.bottom_bar_height;
    let rail_y = bottom_y + 7.0;
    let rail_h = (metrics.bottom_bar_height - 14.0).max(34.0);
    let gap = 8.0;
    let available = (screen_w - 20.0).max(0.0);
    let left_w = (available * if screen_w < 900.0 { 0.56 } else { 0.53 }).max(0.0);
    let right_w = (available - left_w - gap).max(0.0);
    let right_x = 10.0 + left_w + gap;

    draw_hud_panel(theme, Rect::new(10.0, rail_y, left_w, rail_h), None);
    draw_hud_panel(theme, Rect::new(right_x, rail_y, right_w, rail_h), None);

    let mut action = draw_left_rail(state, theme, colors, left_w, rail_y, rail_h);
    let right_action = draw_right_rail(state, theme, colors, right_x, right_w, rail_y, rail_h);
    if right_action != ClockAction::None {
        action = right_action;
    }
    if state.show_help {
        super::help::draw(theme, screen_w, colors.text, colors.dim);
    }
    action
}

fn draw_left_rail(
    state: &PlanetState,
    theme: &UiTheme,
    colors: &PanelColors,
    rail_w: f32,
    y: f32,
    rail_h: f32,
) -> ClockAction {
    let congested = state.congested_tiles();
    let starved = state.starved_factories().len();
    let blocked = state.blocked_factories().len();
    let stalled = state.stalled_drone_count();
    let upkeep = state.upkeep_alert_positions().len();
    let alert_count = i32::from(state.save_failed)
        + i32::from(state.restored_from_backup)
        + i32::from(state.power_balance < 0.0)
        + i32::from(state.battery_seconds <= 0.0)
        + i32::from(state.power_collapse_shutdown > 0.0)
        + i32::from(congested > 0)
        + i32::from(starved > 0)
        + i32::from(blocked > 0)
        + i32::from(stalled > 0)
        + i32::from(upkeep > 0);
    let focusable_alert = congested > 0 || starved > 0 || blocked > 0 || stalled > 0 || upkeep > 0;
    let flow_alert = has_flow_alert(state);
    let alert = alert_button(
        state.save_failed,
        state.restored_from_backup,
        flow_alert,
        state.flow_overlay,
        focusable_alert,
    );
    let cancel_available = has_pending_interaction(state) && alert.is_none();
    let next_label = state
        .next_interesting_event()
        .map(|event| format!("{} {:.0}s", next_event_code(event.label), event.seconds))
        .unwrap_or_else(|| "NEXT".to_string());
    let alert_label = if cancel_available {
        "CANCEL".to_string()
    } else {
        alert
            .map(|(label, _)| label.to_string())
            .unwrap_or(next_label)
    };

    let labels = [
        alert_label,
        format!("ALERT {}", alert_count),
        if state.desktop_panel == DesktopPanel::Build && !state.focus_mode {
            "BUILD ON".to_string()
        } else {
            "BUILD".to_string()
        },
        if state.desktop_panel == DesktopPanel::Info && !state.focus_mode {
            "INFO ON".to_string()
        } else {
            "INFO".to_string()
        },
        if state.flow_overlay {
            "FLOW ON"
        } else {
            "FLOW"
        }
        .to_string(),
        if state.factory_deck_open {
            "DECK ON"
        } else {
            "DECK"
        }
        .to_string(),
    ];
    let mut widths = [54.0, 52.0, 52.0, 48.0, 48.0, 52.0];
    let total = widths.iter().sum::<f32>() + 5.0 * 5.0;
    let scale = ((rail_w - 16.0) / total).clamp(0.62, 1.0);
    for width in &mut widths {
        *width *= scale;
    }
    let mut x = 18.0;
    let mut action = ClockAction::None;
    for (index, label) in labels.iter().enumerate() {
        let rect = Rect::new(x, y + (rail_h - 24.0) * 0.5, widths[index], 24.0);
        if draw_hud_button(theme, rect, label) {
            action = match index {
                0 if cancel_available => ClockAction::CancelModes,
                0 => alert
                    .map(|(_, alert_action)| alert_action)
                    .unwrap_or(ClockAction::NextEvent),
                2 => ClockAction::OpenBuildPanel,
                3 => ClockAction::OpenInfoPanel,
                4 => ClockAction::ToggleFlow,
                5 => ClockAction::ToggleDeck,
                _ => ClockAction::None,
            };
        }
        x += widths[index] + 5.0 * scale;
    }
    draw_ui_text(
        if state.paused { "PAUSED" } else { "LIVE" },
        18.0,
        y + rail_h - 5.0,
        8.0,
        if state.paused {
            colors.warning
        } else {
            colors.success
        },
    );
    action
}

fn draw_right_rail(
    state: &PlanetState,
    theme: &UiTheme,
    colors: &PanelColors,
    x: f32,
    w: f32,
    y: f32,
    h: f32,
) -> ClockAction {
    let text = colors.text;
    let dim = colors.dim;
    let button_y = y + 5.0;
    let button_h = (h - 10.0).max(24.0);
    let mut action = ClockAction::None;
    let mut cursor_x = x + 10.0;
    let core_w = 48.0_f32.min((w - 20.0).max(0.0));
    if draw_hud_button(
        theme,
        Rect::new(cursor_x, button_y, core_w, button_h),
        "CORE",
    ) {
        action = ClockAction::CenterCore;
    }
    cursor_x += core_w + 5.0;

    let zoom_slot_w = 54.0_f32.min((w - (cursor_x - x) - 8.0).max(0.0));
    let zoom_y = y + 7.0;
    let (zoom_out, zoom_in) = zoom_button_rects(cursor_x, zoom_y, zoom_slot_w);
    if draw_hud_button(theme, zoom_out, "-") {
        action = ClockAction::ZoomOut;
    }
    if draw_hud_button(theme, zoom_in, "+") {
        action = ClockAction::ZoomIn;
    }
    cursor_x += zoom_slot_w + 5.0;

    let pause_w = 42.0_f32.min((w - (cursor_x - x) - 8.0).max(0.0));
    if draw_hud_button(
        theme,
        Rect::new(cursor_x, button_y, pause_w, button_h),
        if state.paused { ">" } else { "II" },
    ) {
        action = ClockAction::TogglePause;
    }
    cursor_x += pause_w + 8.0;

    if w >= 300.0 {
        let (hours, minutes) = state.battery_time_left();
        draw_ui_text(
            &format!(
                "MISSION {:02}:{:02}",
                (state.time_played / 60.0) as u32,
                state.time_played as u32 % 60
            ),
            cursor_x,
            y + 15.0,
            8.0,
            text,
        );
        draw_ui_text(
            &format!(
                "{}x  BAT {}h {}m",
                speed_label(state.time_scale),
                hours,
                minutes
            ),
            cursor_x,
            y + 29.0,
            8.0,
            dim,
        );
    }
    let help_x = x + w - 62.0;
    if w >= 170.0 && draw_hud_button(theme, Rect::new(help_x, button_y, 52.0, button_h), "HELP") {
        action = ClockAction::ToggleHelp;
    }
    if state.show_help {
        draw_ui_text("HELP ON", help_x, y + h - 4.0, 7.0, colors.primary);
    }
    action
}

fn speed_label(speed: f32) -> &'static str {
    if speed >= 3.9 {
        "4"
    } else if speed >= 1.9 {
        "2"
    } else {
        "1"
    }
}
