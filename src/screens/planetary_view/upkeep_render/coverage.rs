//! Spatial overlays for upkeep coverage, hazard fields, and their legends.

use crate::engine::GridPos;
use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::metrics::{grid_to_screen, HudMetrics};

/// Outline what the upkeep buildings actually cover.
///
/// Sweepers, Shield Generators and Heater Nodes all work by radius, and none of
/// them showed it, so placing them was guesswork. Coverage is drawn for the
/// type being built (with a preview under the cursor) and for whatever building
/// the player has selected.
pub(crate) fn draw_coverage(state: &PlanetState, metrics: HudMetrics, hovered: Option<GridPos>) {
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
        draw_coverage_area(center, radius, emphasised, metrics, Colors::PRIMARY);
    }

    let Some(selected) = state.selected_tile else {
        return;
    };
    let Some(tile) = state.grid.get(selected) else {
        return;
    };
    if tile.terrain != crate::engine::TerrainType::Forest || tile.forest_cleared || tile.filter {
        return;
    }

    let radius = state.config.upkeep.forest_buffer_radius;
    draw_coverage_area(selected, radius, true, metrics, Colors::SUCCESS);
    let reduction = (1.0 - state.config.upkeep.forest_buffer_multiplier.clamp(0.0, 1.0)) * 100.0;
    let (x, y) = grid_to_screen(selected, metrics);
    draw_ui_text(
        &format!("FOREST BUFFER // -{reduction:.0}% DUST"),
        x + 4.0,
        y - 6.0,
        9.0,
        Colors::SUCCESS,
    );
}

/// Mark network and upkeep buildings outside the nearest hazard counter. The
/// outline is intentionally patterned and not just a color wash, so it stays
/// legible for color-blind players and in screenshots.
pub(crate) fn draw_uncovered_hazards(state: &PlanetState, metrics: HudMetrics) {
    if !state.hazards.any() {
        return;
    }
    let counters: Vec<_> = state
        .grid
        .find_buildings(crate::engine::BuildingType::ShieldGenerator)
        .into_iter()
        .chain(
            state
                .grid
                .find_buildings(crate::engine::BuildingType::HeaterNode),
        )
        .collect();
    let radius = state.config.upkeep.hazard_counter_radius;
    for (pos, tile) in state.grid.iter_tiles() {
        let Some(building) = tile.building.as_ref() else {
            continue;
        };
        if !building.transmits_power() && !building.consumes_power() {
            continue;
        }
        if counters
            .iter()
            .any(|counter| pos.distance(*counter) as i32 <= radius)
        {
            continue;
        }
        let (x, y) = grid_to_screen(pos, metrics);
        let edge = if state.acid_strength() > 0.0 {
            Color::new(0.85, 0.25, 0.85, 0.8)
        } else {
            Color::new(0.35, 0.75, 1.0, 0.8)
        };
        draw_rectangle_lines(
            x + 2.0,
            y + 2.0,
            metrics.tile_size - 4.0,
            metrics.tile_size - 4.0,
            1.5,
            edge,
        );
        draw_line(
            x + 4.0,
            y + 4.0,
            x + metrics.tile_size - 4.0,
            y + metrics.tile_size - 4.0,
            1.0,
            edge,
        );
    }
}

/// Render authored spatial hazard regions and a compact legend.
pub(crate) fn draw_hazard_fields(state: &PlanetState, metrics: HudMetrics) {
    let def = state.data().planet(state.planet_index);
    let width = state.grid.width as f32;
    let height = state.grid.height as f32;
    for field in &def.hazard_fields {
        let center = GridPos::new(
            (field.center[0] * width) as i32,
            (field.center[1] * height) as i32,
        );
        let radius = (field.radius * width.min(height)).max(1.0) as i32;
        let color = if field.hazard == "acid" {
            Color::new(0.8, 0.15, 0.75, 0.12)
        } else {
            Color::new(0.2, 0.65, 1.0, 0.12)
        };
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let pos = GridPos::new(center.x + dx, center.y + dy);
                if !pos.in_bounds(state.grid.width, state.grid.height)
                    || center.distance(pos) as i32 > radius
                {
                    continue;
                }
                let (x, y) = grid_to_screen(pos, metrics);
                draw_rectangle(x, y, metrics.tile_size, metrics.tile_size, color);
            }
        }
    }
    if !def.hazard_fields.is_empty() {
        let field_names = def
            .hazard_fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>()
            .join(" // ");
        let legend = format!("HAZARDS  //  purple ACID  //  blue COLD  //  {field_names}");
        draw_ui_text(
            &legend,
            metrics.base_offset_x() + 8.0,
            metrics.base_offset_y() + 18.0,
            9.0,
            Colors::TEXT_DIM,
        );
    }
}

/// The covered area is a distance test, so draw exactly that: the tiles the
/// simulation would actually reach, not a circle that only looks like them.
fn draw_coverage_area(
    center: GridPos,
    radius: i32,
    emphasised: bool,
    metrics: HudMetrics,
    color: Color,
) {
    let fill = with_alpha(color, if emphasised { 0.16 } else { 0.08 });
    let edge = with_alpha(color, if emphasised { 0.85 } else { 0.45 });

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
