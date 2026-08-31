//! Desktop command bar layout and readouts.

use crate::data::UiTheme;
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::super::metrics::HudMetrics;
use super::super::PanelColors;
use super::alerts::{alert_button, alert_text, has_flow_alert, next_event_code};
use super::{help, ClockAction};

mod status;

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
    let alert_message = alert_text(state, stalled, starved, blocked, upkeep, congested);
    let mut clock = ClockAction::None;
    draw_ui_text(
        alert_message,
        106.0,
        bottom_y + 35.0,
        11.0,
        if alert_count > 0 { error } else { success },
    );
    if let Some((alert_label, alert_action)) = alert_button(
        state.save_failed,
        state.restored_from_backup,
        flow_alert,
        state.flow_overlay,
        focusable_alert,
    ) {
        if draw_hud_button(
            theme,
            Rect::new(10.0 + alert_w - 74.0, bottom_y + 10.0, 66.0, 22.0),
            alert_label,
        ) {
            clock = alert_action;
        }
    }

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
            ("ZOOM", "Tap - / +"),
            ("PAUSE", "Tap II"),
        ]
    } else if controls_w < 560.0 {
        vec![
            ("SELECT", "Tap grid"),
            ("PAN", "Drag grid"),
            ("ZOOM", "Tap - / +"),
            ("BUILD", "Tap card"),
            ("PAUSE", "Tap II"),
        ]
    } else {
        vec![
            ("SELECT", "Tap grid"),
            ("PAN", "Drag grid"),
            ("ZOOM", "Tap - / +"),
            ("BUILD", "Tap card"),
            ("HARVEST", "Tap inspector"),
            ("DEMOLISH", "Tap button"),
            ("PAUSE", "Tap II"),
        ]
    };
    let text_controls_w = (controls_w - 190.0).max(120.0);
    let slot_w = text_controls_w / controls.len() as f32;
    for (index, (label, hint)) in controls.iter().enumerate() {
        let slot_x = controls_x + index as f32 * slot_w;
        let x = slot_x + 12.0;
        if *label == "ZOOM" {
            let label_width = 28.0;
            let label_x = slot_x + ((slot_w - label_width).max(0.0) * 0.5);
            draw_ui_text(label, label_x, control_y, 10.0, text);
            if let Some(action) = draw_zoom_controls(theme, slot_x, control_y + 5.0, slot_w) {
                clock = action;
            }
        } else {
            draw_ui_text(label, x, control_y, 10.0, text);
            draw_ui_text(hint, x, control_y + 18.0, 9.0, dim);
        }
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
    let next_event = state.next_interesting_event();
    let next_label = next_event
        .map(|event| format!("{} {:.0}s", next_event_code(event.label), event.seconds))
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
    if draw_hud_button(
        theme,
        Rect::new(controls_x + controls_w - 240.0, bottom_y + 10.0, 54.0, 22.0),
        if state.flow_overlay {
            "FLOW ON"
        } else {
            "FLOW"
        },
    ) {
        clock = ClockAction::ToggleFlow;
    }
    let deck_offset = if controls_w >= 360.0 { 300.0 } else { 180.0 };
    if draw_hud_button(
        theme,
        Rect::new(
            controls_x + controls_w - deck_offset,
            bottom_y + 10.0,
            54.0,
            22.0,
        ),
        if state.factory_deck_open {
            "DECK ON"
        } else {
            "DECK"
        },
    ) {
        clock = ClockAction::ToggleDeck;
    }

    let status_action = status::draw(state, bottom_y, status_x, status_w, theme, metrics, colors);
    if status_action != ClockAction::None {
        clock = status_action;
    }

    let mode_y = bottom_y + metrics.bottom_bar_height - 8.0;
    let (mode_label, mode_color) = if state.relocation_source.is_some() {
        ("RELOCATE MODE", colors.warning)
    } else if state.box_select_mode {
        ("BOX SELECT MODE", primary_soft)
    } else if state.bulk_purge_armed || state.purge_armed.is_some() {
        ("PURGE CONFIRM", colors.error)
    } else if state.demolish_mode {
        ("DEMOLISH MODE", colors.error)
    } else if state.selected_building.is_some() {
        ("BUILD MODE", primary_soft)
    } else {
        ("SELECT MODE", primary_soft)
    };
    draw_ui_text(mode_label, controls_x + 12.0, mode_y, 9.0, mode_color);
    if let Some(selected) = state.selected_building {
        draw_ui_text(
            selected.name(state.data()),
            controls_x + 86.0,
            mode_y,
            9.0,
            text,
        );
    }

    if state.show_help {
        help::draw(theme, screen_w, text, dim);
    }

    clock
}

fn draw_zoom_controls(theme: &UiTheme, slot_x: f32, y: f32, slot_w: f32) -> Option<ClockAction> {
    let (zoom_out, zoom_in) = super::zoom_button_rects(slot_x, y, slot_w);
    let mut action = None;
    if draw_hud_button(theme, zoom_out, "-") {
        action = Some(ClockAction::ZoomOut);
    }
    if draw_hud_button(theme, zoom_in, "+") {
        action = Some(ClockAction::ZoomIn);
    }
    action
}
