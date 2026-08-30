//! Toasts, anywhere.
//!
//! The notification stack was drawn by the planetary view alone, so anything
//! that landed while the player was reading the research tree or the solar map
//! was never seen at all — which is exactly when a directive completing or a
//! world finishing a stage matters most.

use crate::state::PlanetState;
use macroquad::prelude::*;
use macroquad_toolkit::notifications::{draw_notification, NotificationRenderConfig};

const COMPACT_TOAST_WIDTH: f32 = 200.0;
const REGULAR_TOAST_WIDTH: f32 = 260.0;
const COMPACT_TOAST_LIMIT: usize = 3;

/// Where a screen wants its toasts.
#[derive(Debug, Clone, Copy)]
pub struct ToastAnchor {
    pub x: f32,
    pub y: f32,
}

fn config() -> NotificationRenderConfig {
    let compact = compact_screen(screen_width(), screen_height());
    NotificationRenderConfig {
        width: toast_width_for_viewport(screen_width(), screen_height()),
        row_height: if compact { 26.0 } else { 30.0 },
        spacing: if compact { 4.0 } else { 6.0 },
        padding: if compact { 7.0 } else { 10.0 },
        font_size: if compact { 11.0 } else { 13.0 },
        ..NotificationRenderConfig::default()
    }
}

/// Draw the stack from `anchor` downwards.
pub fn draw_toasts(state: &PlanetState, anchor: ToastAnchor) {
    let config = config();
    let notifications = state.notifications.get_notifications();
    let start = notifications
        .len()
        .saturating_sub(max_visible_toasts(screen_width(), screen_height()));
    let mut y = anchor.y;
    for notification in notifications.iter().skip(start) {
        draw_notification(notification, anchor.x, y, &config);
        y += config.row_height + config.spacing;
    }
}

/// Width used by other HUDs when reserving room for the toast stack.
pub fn toast_width() -> f32 {
    toast_width_for_viewport(screen_width(), screen_height())
}

/// Menu screens only draw this stack when their content has room to keep
/// transient cards away from controls and scrollable panels.
pub fn menu_toasts_fit_viewport(screen_w: f32, screen_h: f32) -> bool {
    screen_w >= 960.0 && screen_h >= 720.0
}

fn compact_screen(screen_w: f32, screen_h: f32) -> bool {
    screen_w < 900.0 || screen_h < 650.0
}

fn toast_width_for_viewport(screen_w: f32, screen_h: f32) -> f32 {
    if compact_screen(screen_w, screen_h) {
        COMPACT_TOAST_WIDTH
    } else {
        REGULAR_TOAST_WIDTH
    }
}

fn max_visible_toasts(screen_w: f32, screen_h: f32) -> usize {
    if compact_screen(screen_w, screen_h) {
        COMPACT_TOAST_LIMIT
    } else {
        usize::MAX
    }
}

/// Where a full-screen menu with side panels has room: across the top of the
/// middle, under the header.
///
/// Right-aligning them the way the planetary view does puts them straight on
/// top of a panel, because a menu's panels run the full height of the screen.
pub fn menu_anchor(screen_w: f32) -> ToastAnchor {
    ToastAnchor {
        x: ((screen_w - config().width) * 0.5).max(24.0),
        y: 92.0,
    }
}

/// Where a menu whose content is centred has room instead: down in the corner,
/// above the ESC hint.
pub fn menu_anchor_low(screen_h: f32) -> ToastAnchor {
    ToastAnchor {
        x: 24.0,
        y: (screen_h - 150.0).max(92.0),
    }
}

#[cfg(test)]
mod tests;
