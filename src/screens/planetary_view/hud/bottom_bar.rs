//! Bottom command bar: alerts, controls legend, mission time, speed, and help overlay

mod alerts;
mod desktop;
mod graph;
mod help;
mod mobile;

use crate::data::UiTheme;
use crate::state::PlanetState;
use macroquad::prelude::Rect;

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
    ToggleFlow,
    ToggleDeck,
    RetrySave,
    FocusAlert,
    CancelModes,
    ToggleMobileBuild,
    ToggleMobileInfo,
    CloseMobilePanel,
    ToggleHelp,
    CenterCore,
    ZoomOut,
    ZoomIn,
    CycleGraph,
}

pub(super) fn zoom_button_rects(slot_x: f32, y: f32, slot_w: f32) -> (Rect, Rect) {
    let button_w = ((slot_w - 2.0) * 0.5).clamp(14.0, 24.0);
    let controls_w = button_w * 2.0 + 2.0;
    let x = slot_x + ((slot_w - controls_w).max(0.0) * 0.5);
    (
        Rect::new(x, y, button_w, 18.0),
        Rect::new(x + button_w + 2.0, y, button_w, 18.0),
    )
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
    if metrics.mobile {
        mobile::draw(state, screen_w, screen_h, theme, metrics, colors)
    } else {
        desktop::draw(state, screen_w, screen_h, theme, metrics, colors)
    }
}

#[cfg(test)]
mod tests;
