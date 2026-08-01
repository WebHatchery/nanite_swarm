//! Toasts, anywhere.
//!
//! The notification stack was drawn by the planetary view alone, so anything
//! that landed while the player was reading the research tree or the solar map
//! was never seen at all — which is exactly when a directive completing or a
//! world finishing a stage matters most.

use crate::state::PlanetState;
use macroquad::prelude::*;
use macroquad_toolkit::notifications::{draw_notification, NotificationRenderConfig};

/// Where a screen wants its toasts.
#[derive(Debug, Clone, Copy)]
pub struct ToastAnchor {
    pub x: f32,
    pub y: f32,
}

fn config() -> NotificationRenderConfig {
    NotificationRenderConfig {
        width: 260.0,
        row_height: 30.0,
        spacing: 6.0,
        font_size: 13.0,
        ..NotificationRenderConfig::default()
    }
}

/// Draw the stack from `anchor` downwards.
pub fn draw_toasts(state: &PlanetState, anchor: ToastAnchor) {
    let config = config();
    let mut y = anchor.y;
    for notification in state.notifications.get_notifications() {
        draw_notification(notification, anchor.x, y, &config);
        y += config.row_height + config.spacing;
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
