//! Ore on the map.
//!
//! Richness was readable one tile at a time in the inspector, which made
//! prospecting a matter of sweeping the cursor over the whole world. Deposits
//! are marked on the ground instead, and marked harder while a Drill is in
//! hand — the moment the difference is about to matter.

use crate::engine::BuildingType;
use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use super::metrics::{grid_to_screen, HudMetrics};

/// Below this a tile is ordinary ground and not worth marking either way.
const DEPOSIT_AT: f32 = 1.05;
const LEAN_BELOW: f32 = 0.95;

pub(super) fn draw_ore(state: &PlanetState, metrics: HudMetrics) {
    // A drill in hand is the moment the ground matters, so the marks come up.
    let prospecting = state.selected_building == Some(BuildingType::Drill);
    // Quiet, but not invisible: at much less than this a deposit cannot be
    // picked out against the terrain at all, which defeats the point.
    let strength = if prospecting { 1.0 } else { 0.8 };

    for (pos, tile) in state.grid.iter_tiles() {
        if !tile.revealed {
            continue;
        }
        let richness = tile.ore_richness;
        if richness < DEPOSIT_AT && richness > LEAN_BELOW {
            continue;
        }
        // Poor ground is only worth marking when something is about to be
        // built on it; marked always, it is most of the map and reads as
        // clutter rather than as information.
        if richness <= LEAN_BELOW && !prospecting {
            continue;
        }
        // Ground already built on is not a placement decision any more.
        if tile.building.is_some() && !prospecting {
            continue;
        }

        let (x, y) = grid_to_screen(pos, metrics);
        let size = metrics.tile_size;
        if richness >= DEPOSIT_AT {
            draw_deposit(x, y, size, richness, strength);
        } else {
            draw_lean(x, y, size, strength);
        }
    }
}

/// A deposit: grains of ore, more of them the richer the ground.
fn draw_deposit(x: f32, y: f32, size: f32, richness: f32, strength: f32) {
    // Two grains at the low end of a deposit, five at the top of the range.
    let grains = ((richness - 1.0) * 4.0).ceil().clamp(1.0, 5.0) as usize;
    let radius = (size * 0.055).max(1.0);
    let color = with_alpha(Colors::ACCENT, (0.35 + (richness - 1.0) * 0.4) * strength);
    // A fixed scatter, so the same ground looks the same every frame.
    const SPOTS: [(f32, f32); 5] = [
        (0.28, 0.30),
        (0.68, 0.36),
        (0.44, 0.62),
        (0.76, 0.70),
        (0.22, 0.74),
    ];
    for (dx, dy) in SPOTS.iter().take(grains) {
        draw_circle(x + size * dx, y + size * dy, radius, color);
    }
}

/// Lean ground: one hollow mark, enough to say "not here" without shouting.
fn draw_lean(x: f32, y: f32, size: f32, strength: f32) {
    draw_circle_lines(
        x + size * 0.5,
        y + size * 0.5,
        (size * 0.11).max(1.5),
        1.0,
        with_alpha(Colors::TEXT_DIM, 0.3 * strength),
    );
}
