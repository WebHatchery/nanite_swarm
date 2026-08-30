//! Portrait command bar: drawer navigation, recovery, and clock controls.

use crate::data::UiTheme;
use crate::state::{MobilePanel, PlanetState};
use crate::ui::{draw_hud_button, draw_hud_panel};
use macroquad::prelude::Rect;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::super::metrics::HudMetrics;
use super::super::PanelColors;
use super::{alert_button, has_flow_alert, has_pending_interaction, next_event_code, ClockAction};

pub(super) fn draw(
    state: &PlanetState,
    screen_w: f32,
    screen_h: f32,
    theme: &UiTheme,
    metrics: HudMetrics,
    colors: &PanelColors,
) -> ClockAction {
    let bottom_y = screen_h - metrics.bottom_bar_height;
    draw_hud_panel(
        theme,
        Rect::new(0.0, bottom_y, screen_w, metrics.bottom_bar_height),
        None,
    );

    let gap = 6.0;
    let top_w = ((screen_w - 20.0 - gap * 3.0) / 4.0).max(48.0);
    let panel = state.mobile_panel;
    let first_label = if panel == MobilePanel::Map {
        "BUILD"
    } else {
        "MAP"
    };
    let second_label = if panel == MobilePanel::Info {
        "BUILD"
    } else {
        "INFO"
    };
    let mut action = ClockAction::None;
    if draw_hud_button(
        theme,
        Rect::new(10.0, bottom_y + 8.0, top_w, 22.0),
        first_label,
    ) {
        action = if panel == MobilePanel::Map {
            ClockAction::ToggleMobileBuild
        } else {
            ClockAction::CloseMobilePanel
        };
    }
    if draw_hud_button(
        theme,
        Rect::new(10.0 + top_w + gap, bottom_y + 8.0, top_w, 22.0),
        second_label,
    ) {
        action = if panel == MobilePanel::Info {
            ClockAction::ToggleMobileBuild
        } else {
            ClockAction::ToggleMobileInfo
        };
    }
    if draw_hud_button(
        theme,
        Rect::new(10.0 + (top_w + gap) * 2.0, bottom_y + 8.0, top_w, 22.0),
        if state.flow_overlay {
            "FLOW ON"
        } else {
            "FLOW"
        },
    ) {
        action = ClockAction::ToggleFlow;
    }
    if draw_hud_button(
        theme,
        Rect::new(10.0 + (top_w + gap) * 3.0, bottom_y + 8.0, top_w, 22.0),
        if state.factory_deck_open {
            "DECK ON"
        } else {
            "DECK"
        },
    ) {
        action = ClockAction::ToggleDeck;
    }

    let congested = state.congested_tiles();
    let starved = state.starved_factories().len();
    let blocked = state.blocked_factories().len();
    let stalled = state.stalled_drone_count();
    let upkeep = state.upkeep_alert_positions().len();
    let focusable_alert = congested > 0 || starved > 0 || blocked > 0 || stalled > 0 || upkeep > 0;
    let flow_alert = has_flow_alert(state);
    let row_y = bottom_y + 36.0;
    let alert_width = 80.0_f32.min((screen_w - 20.0).max(48.0));
    let alert = alert_button(
        state.save_failed,
        state.restored_from_backup,
        flow_alert,
        state.flow_overlay,
        focusable_alert,
    );
    let cancel_available = has_pending_interaction(state) && alert.is_none();
    let first_row_label = if cancel_available {
        "CANCEL".to_string()
    } else if let Some((label, _)) = alert {
        label.to_string()
    } else {
        state
            .next_interesting_event()
            .map(|event| format!("{} {:.0}s", next_event_code(event.label), event.seconds))
            .unwrap_or_else(|| "NEXT".to_string())
    };
    if draw_hud_button(
        theme,
        Rect::new(10.0, row_y, alert_width, 22.0),
        &first_row_label,
    ) {
        action = if cancel_available {
            ClockAction::CancelModes
        } else {
            alert
                .map(|(_, alert_action)| alert_action)
                .unwrap_or(ClockAction::NextEvent)
        };
    }
    if draw_hud_button(theme, Rect::new(96.0, row_y, 32.0, 22.0), "-") {
        action = ClockAction::Slower;
    }
    if draw_hud_button(theme, Rect::new(134.0, row_y, 32.0, 22.0), "+") {
        action = ClockAction::Faster;
    }
    if draw_hud_button(
        theme,
        Rect::new(172.0, row_y, 40.0, 22.0),
        if state.paused { ">" } else { "II" },
    ) {
        action = ClockAction::TogglePause;
    }
    if draw_hud_button(
        theme,
        Rect::new(218.0, row_y, (screen_w - 228.0).max(48.0), 22.0),
        "HELP",
    ) {
        action = ClockAction::ToggleHelp;
    }

    if state.show_help
        && draw_mobile_help(theme, colors, screen_w, screen_h, metrics.bottom_bar_height)
    {
        action = ClockAction::ToggleHelp;
    }
    action
}

fn draw_mobile_help(
    theme: &UiTheme,
    colors: &PanelColors,
    screen_w: f32,
    screen_h: f32,
    bottom_bar_height: f32,
) -> bool {
    let x = 10.0;
    let y = 90.0;
    let w = (screen_w - 20.0).max(0.0);
    let h = mobile_help_height(screen_h, bottom_bar_height, y);
    draw_hud_panel(theme, Rect::new(x, y, w, h), Some("HELP & CONTROLS"));
    if draw_hud_button(theme, Rect::new(x + w - 68.0, y + 7.0, 56.0, 22.0), "CLOSE") {
        return true;
    }
    let lines = [
        "BUILD: tap a card, then tap the map",
        "INFO: inspect the selected tile and network",
        "Drag the map to pan; use - / + to zoom",
        "FLOW and DECK open their live overlays",
        "JUMP / RETRY follows the current alert",
        "Top buttons open Research, Ship, Map, Menu",
    ];
    for (index, line) in lines.iter().enumerate() {
        draw_ui_text(
            line,
            x + 16.0,
            y + 54.0 + index as f32 * 24.0,
            11.0,
            colors.text,
        );
    }
    false
}

fn mobile_help_height(screen_h: f32, bottom_bar_height: f32, top: f32) -> f32 {
    let available = (screen_h - top - bottom_bar_height - 10.0).max(0.0);
    (screen_h - 112.0).min(available).clamp(180.0, 430.0)
}

#[cfg(test)]
mod tests;
