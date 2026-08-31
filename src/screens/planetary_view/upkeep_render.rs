//! Showing the two things the sim tracks per building that the map never did:
//! how worn a building is, and how far its upkeep reaches.

use crate::state::{MobilePanel, PlanetState};
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use super::metrics::{grid_to_screen, HudMetrics};

mod coverage;

pub(super) use coverage::{draw_coverage, draw_hazard_fields, draw_uncovered_hazards};

/// Below this a building is merely grubby and not worth shouting about.
const WEAR_VISIBLE_AT: f32 = 20.0;

/// Outline the part of the network that has lost its way home.
///
/// Only while something is actually stalled: a run half-laid out towards a new
/// drill is disconnected on purpose, and outlining that every time the player
/// places a conduit would be noise. When drones do stall, this is the answer
/// to the question the stalled counter raises and never answered — where.
pub(super) fn draw_severed_network(state: &PlanetState, metrics: HudMetrics, time: f32) {
    if state.stalled_drone_count() == 0 {
        return;
    }

    let pulse = (time * 3.0).sin() * 0.5 + 0.5;
    for pos in state.severed_network() {
        if !state.grid.get(pos).is_some_and(|tile| tile.revealed) {
            continue;
        }
        let (x, y) = grid_to_screen(pos, metrics);
        let size = metrics.tile_size;
        draw_rectangle(
            x,
            y,
            size,
            size,
            with_alpha(Colors::ERROR, 0.10 + pulse * 0.12),
        );
        draw_rectangle_lines(
            x + 1.0,
            y + 1.0,
            size - 2.0,
            size - 2.0,
            2.0,
            with_alpha(Colors::ERROR, 0.55 + pulse * 0.35),
        );
    }
}

/// Tint buildings by how much dust and corrosion they are carrying.
///
/// Wear was a number in the inspector for one hovered tile, which is no use
/// when acid is eating a run twenty tiles long: the player needs to see *which*
/// part of the network is going.
pub(super) fn draw_wear(state: &PlanetState, metrics: HudMetrics, time: f32) {
    let dust_response = state.resolved_dust_response();
    for (pos, tile) in state.grid.iter_tiles() {
        if !tile.revealed {
            continue;
        }
        let Some(building) = tile.building.as_ref() else {
            continue;
        };
        if building.dust < WEAR_VISIBLE_AT && building.acid_wear < WEAR_VISIBLE_AT {
            continue;
        }

        let severity =
            ((building.dust - WEAR_VISIBLE_AT) / (100.0 - WEAR_VISIBLE_AT)).clamp(0.0, 1.0);
        let acid_severity =
            ((building.acid_wear - WEAR_VISIBLE_AT) / (100.0 - WEAR_VISIBLE_AT)).clamp(0.0, 1.0);
        let (screen_x, screen_y) = grid_to_screen(pos, metrics);
        let color = if building.acid_wear >= 100.0 {
            with_alpha(Color::new(0.75, 0.2, 0.9, 1.0), 0.35 + acid_severity * 0.3)
        } else if building.is_dust_stalled_with(&dust_response) {
            // Stalled is a different state, not just more wear: this building
            // has stopped, and if it carried the network the run is cut.
            with_alpha(Colors::ERROR, 0.30 + (time * 3.0).sin().abs() * 0.25)
        } else {
            with_alpha(
                if acid_severity > severity {
                    Color::new(0.75, 0.2, 0.9, 1.0)
                } else {
                    Colors::WARNING
                },
                0.10 + severity.max(acid_severity) * 0.30,
            )
        };
        draw_rectangle(
            screen_x + 1.0,
            screen_y + 1.0,
            metrics.tile_size - 2.0,
            metrics.tile_size - 2.0,
            color,
        );
    }
}

/// Draw the toast stack over the map, clear of both side panels.
///
/// The toolkit anchors its stack to a screen corner, and every corner here is
/// already a panel, so the placement is done by hand and only the drawing is
/// borrowed.
pub(super) fn draw_notifications(state: &PlanetState, metrics: HudMetrics, screen_w: f32) {
    if metrics.mobile {
        // A drawer owns the whole portrait center; toasts would otherwise be
        // painted over building cards and their touch targets. The map still
        // has a reserved strip below the tutorial for transient notices.
        if state.mobile_panel != MobilePanel::Map
            || state.show_help
            || state.flow_overlay
            || state.factory_deck_open
        {
            return;
        }
        let anchor = crate::screens::ToastAnchor {
            x: 12.0,
            y: metrics.top_bar_height + 96.0,
        };
        crate::screens::draw_toasts(state, anchor);
        return;
    }
    // Clear of the right-hand panel stack, which the full-screen menus do not
    // have and so anchor differently.
    let anchor = crate::screens::ToastAnchor {
        x: (screen_w - metrics.right_panel_width - crate::screens::toast_width() - 24.0)
            .max(metrics.base_offset_x() + 8.0),
        y: metrics.top_bar_height + 96.0,
    };
    crate::screens::draw_toasts(state, anchor);
}
