//! The Seed Ship on the skyline.
//!
//! The megastructure used to exist only on its own screen, so a world building
//! one looked exactly like a world that was not. It now stands over the Core —
//! the yard is wherever the swarm's attention is — growing with every stage
//! paid for, and it is gone the moment the ship leaves.

use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::math::pulse01_at;

use super::metrics::{grid_to_screen, HudMetrics};

/// Height of a finished ship, in tiles. Tall enough to read across a base,
/// short enough not to swallow the top of the map at full zoom.
const FULL_HEIGHT_TILES: f32 = 4.6;
/// Width of the hull at its base, in tiles.
const BASE_WIDTH_TILES: f32 = 1.5;

pub(super) fn draw_seed_ship(state: &PlanetState, metrics: HudMetrics, time: f32) {
    if !state.seed_ship.has_broken_ground() {
        return;
    }
    let Some(core) = state.grid.find_core() else {
        return;
    };

    let (core_x, core_y) = grid_to_screen(core, metrics);
    let tile = metrics.tile_size;
    let ground_x = core_x + tile * 0.5;
    let ground_y = core_y + tile * 0.45;

    let built = state.seed_ship.built_fraction();
    let full_height = FULL_HEIGHT_TILES * tile;
    let hull_height = full_height * built;
    let half_base = BASE_WIDTH_TILES * tile * 0.5;

    draw_gantry(ground_x, ground_y, full_height, half_base);
    if hull_height > 1.0 {
        draw_hull(ground_x, ground_y, hull_height, full_height, half_base);
    }

    // A finished ship says so from across the base: it is waiting on the
    // player, not on production.
    if state.seed_ship.is_complete() {
        let pulse = pulse01_at(time as f64, 1.6);
        draw_circle(
            ground_x,
            ground_y - full_height - tile * 0.15,
            tile * (0.10 + pulse * 0.06),
            with_alpha(Colors::SUCCESS, 0.35 + pulse * 0.45),
        );
    }
}

/// The scaffolding, which stands to full height from the moment the swarm
/// commits: it is what makes an empty yard read as a yard.
fn draw_gantry(ground_x: f32, ground_y: f32, full_height: f32, half_base: f32) {
    // Bright enough to read against dark terrain: an invisible gantry makes a
    // half-built ship look like a hole in the map rather than work in progress.
    let color = with_alpha(Colors::TEXT_DIM, 0.5);
    let top = ground_y - full_height;
    let left = ground_x - half_base * 1.35;
    let right = ground_x + half_base * 1.35;
    draw_line(left, ground_y, left, top, 1.5, color);
    draw_line(right, ground_y, right, top, 1.5, color);
    // Cross-bracing, thinning out with height.
    let rungs = 6;
    for rung in 0..=rungs {
        let t = rung as f32 / rungs as f32;
        let y = ground_y - full_height * t;
        draw_line(left, y, right, y, 1.0, with_alpha(color, 0.55 - t * 0.25));
    }
}

/// Plating, lighter the higher it goes: a flat silhouette reads as a gap in
/// the terrain rather than as something standing on it.
fn hull_shade(height_fraction: f32) -> Color {
    Color::new(
        0.13 + height_fraction * 0.09,
        0.16 + height_fraction * 0.11,
        0.21 + height_fraction * 0.14,
        0.94,
    )
}

/// The hull itself: a spire that tapers as it rises, drawn only as far as the
/// swarm has paid for.
fn draw_hull(ground_x: f32, ground_y: f32, hull_height: f32, full_height: f32, half_base: f32) {
    let segments = 14;
    for segment in 0..segments {
        let low = segment as f32 / segments as f32;
        let high = (segment + 1) as f32 / segments as f32;
        let y_low = ground_y - hull_height * low;
        let y_high = ground_y - hull_height * high;
        // Taper is measured against the *finished* height, so a half-built
        // ship is a stump of the real shape rather than a small whole ship.
        let width_low = half_base * (1.0 - 0.72 * (hull_height * low / full_height));
        let width_high = half_base * (1.0 - 0.72 * (hull_height * high / full_height));
        draw_triangle(
            vec2(ground_x - width_low, y_low),
            vec2(ground_x + width_low, y_low),
            vec2(ground_x - width_high, y_high),
            hull_shade(low),
        );
        draw_triangle(
            vec2(ground_x + width_low, y_low),
            vec2(ground_x + width_high, y_high),
            vec2(ground_x - width_high, y_high),
            hull_shade(high),
        );
        // Lit down one side, so the spire has a direction to it.
        draw_line(
            ground_x - width_low,
            y_low,
            ground_x - width_high,
            y_high,
            1.6,
            with_alpha(Colors::PRIMARY, 0.85),
        );
        draw_line(
            ground_x + width_low,
            y_low,
            ground_x + width_high,
            y_high,
            1.2,
            with_alpha(Colors::PRIMARY_SOFT, 0.55),
        );
    }
    // The working edge, where the swarm is currently pouring everything it has.
    let top_y = ground_y - hull_height;
    let top_width = half_base * (1.0 - 0.72 * (hull_height / full_height));
    draw_line(
        ground_x - top_width,
        top_y,
        ground_x + top_width,
        top_y,
        2.0,
        Colors::PRIMARY,
    );
}
