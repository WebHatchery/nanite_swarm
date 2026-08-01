//! Showing the two things the sim tracks per building that the map never did:
//! how worn a building is, and how far its upkeep reaches.

use crate::engine::GridPos;
use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::notifications::{draw_notification, NotificationRenderConfig};

use super::metrics::{grid_to_screen, HudMetrics};

/// Below this a building is merely grubby and not worth shouting about.
const WEAR_VISIBLE_AT: f32 = 20.0;

/// Tint buildings by how much dust and corrosion they are carrying.
///
/// Wear was a number in the inspector for one hovered tile, which is no use
/// when acid is eating a run twenty tiles long: the player needs to see *which*
/// part of the network is going.
pub(super) fn draw_wear(state: &PlanetState, metrics: HudMetrics, time: f32) {
    for (pos, tile) in state.grid.iter_tiles() {
        if !tile.revealed {
            continue;
        }
        let Some(building) = tile.building.as_ref() else {
            continue;
        };
        if building.dust < WEAR_VISIBLE_AT {
            continue;
        }

        let severity =
            ((building.dust - WEAR_VISIBLE_AT) / (100.0 - WEAR_VISIBLE_AT)).clamp(0.0, 1.0);
        let (screen_x, screen_y) = grid_to_screen(pos, metrics);
        let color = if building.is_dust_stalled() {
            // Stalled is a different state, not just more wear: this building
            // has stopped, and if it carried the network the run is cut.
            with_alpha(Colors::ERROR, 0.30 + (time * 3.0).sin().abs() * 0.25)
        } else {
            with_alpha(Colors::WARNING, 0.10 + severity * 0.30)
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

/// Outline what the upkeep buildings actually cover.
///
/// Sweepers, Shield Generators and Heater Nodes all work by radius, and none of
/// them showed it, so placing them was guesswork. Coverage is drawn for the
/// type being built (with a preview under the cursor) and for whatever building
/// the player has selected.
pub(super) fn draw_coverage(state: &PlanetState, metrics: HudMetrics, hovered: Option<GridPos>) {
    let mut shown: Vec<(GridPos, i32, bool)> = Vec::new();

    if let Some(building_type) = state.selected_building {
        if let Some(radius) = state.coverage_radius(building_type) {
            for pos in state.grid.find_buildings(building_type) {
                shown.push((pos, radius, false));
            }
            if let Some(pos) = hovered {
                shown.push((pos, radius, true));
            }
        }
    }

    if let Some(pos) = state.selected_tile {
        let radius = state
            .grid
            .get(pos)
            .and_then(|tile| tile.building.as_ref())
            .and_then(|building| state.coverage_radius(building.building_type));
        if let Some(radius) = radius {
            shown.push((pos, radius, true));
        }
    }

    for (center, radius, emphasised) in shown {
        draw_coverage_area(center, radius, emphasised, metrics);
    }
}

/// The covered area is a distance test, so draw exactly that: the tiles the
/// simulation would actually reach, not a circle that only looks like them.
fn draw_coverage_area(center: GridPos, radius: i32, emphasised: bool, metrics: HudMetrics) {
    let fill = with_alpha(Colors::PRIMARY, if emphasised { 0.16 } else { 0.08 });
    let edge = with_alpha(Colors::PRIMARY, if emphasised { 0.85 } else { 0.45 });

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let pos = GridPos::new(center.x + dx, center.y + dy);
            if center.distance(pos) as i32 > radius {
                continue;
            }
            let (screen_x, screen_y) = grid_to_screen(pos, metrics);
            draw_rectangle(
                screen_x,
                screen_y,
                metrics.tile_size,
                metrics.tile_size,
                fill,
            );

            // Draw an edge wherever the neighbour is outside the area, which
            // traces the real boundary however the distance metric is shaped.
            for (nx, ny) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let neighbour = GridPos::new(pos.x + nx, pos.y + ny);
                if center.distance(neighbour) as i32 <= radius {
                    continue;
                }
                let (x1, y1, x2, y2) = match (nx, ny) {
                    (1, 0) => (
                        screen_x + metrics.tile_size,
                        screen_y,
                        screen_x + metrics.tile_size,
                        screen_y + metrics.tile_size,
                    ),
                    (-1, 0) => (screen_x, screen_y, screen_x, screen_y + metrics.tile_size),
                    (0, 1) => (
                        screen_x,
                        screen_y + metrics.tile_size,
                        screen_x + metrics.tile_size,
                        screen_y + metrics.tile_size,
                    ),
                    _ => (screen_x, screen_y, screen_x + metrics.tile_size, screen_y),
                };
                draw_line(x1, y1, x2, y2, 1.5, edge);
            }
        }
    }
}

/// Draw the toast stack over the map, clear of both side panels.
///
/// The toolkit anchors its stack to a screen corner, and every corner here is
/// already a panel, so the placement is done by hand and only the drawing is
/// borrowed.
pub(super) fn draw_notifications(state: &PlanetState, metrics: HudMetrics, screen_w: f32) {
    let config = NotificationRenderConfig {
        width: 260.0,
        row_height: 30.0,
        spacing: 6.0,
        font_size: 13.0,
        ..NotificationRenderConfig::default()
    };
    let x = (screen_w - metrics.right_panel_width - config.width - 24.0)
        .max(metrics.base_offset_x() + 8.0);
    let mut y = metrics.top_bar_height + 96.0;

    for notification in state.notifications.get_notifications() {
        draw_notification(notification, x, y, &config);
        y += config.row_height + config.spacing;
    }
}
