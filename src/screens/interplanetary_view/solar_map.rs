//! Orbit-map rendering and planet hit targets for the interplanetary screen.

use crate::ui::{color_from_rgba, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, Pointer};

use super::core_marker::draw_core_stage_marker;
use super::MapView;

/// Rendered world positions, plus the world most recently pointed at.
pub(super) struct SolarMapResult {
    pub(super) positions: Vec<(f32, f32)>,
    pub(super) hovered_planet: Option<usize>,
}

/// Inputs for the authored orbit map.
pub(super) struct SolarMapInput<'view, 'data> {
    pub(super) view: &'view MapView<'data>,
    pub(super) center_x: f32,
    pub(super) center_y: f32,
    pub(super) orbit_scale: f32,
    pub(super) map_compressed: bool,
    pub(super) animation_time: f64,
    pub(super) pointer: Pointer,
    pub(super) hovered_planet: Option<usize>,
}

/// Draw every orbit and planet that can be interacted with on the solar map.
pub(super) fn draw(input: SolarMapInput<'_, '_>) -> SolarMapResult {
    let mut hovered_planet = input.hovered_planet;
    let mut positions = Vec::with_capacity(input.view.data.planets.len());
    if input.map_compressed {
        return SolarMapResult {
            positions,
            hovered_planet,
        };
    }

    for (i, planet) in input.view.data.planets.iter().enumerate() {
        let orbit_color = if i == input.view.current_planet {
            Color::new(0.0, 0.85, 1.0, 0.3)
        } else {
            Colors::SECONDARY
        };
        draw_circle_lines(
            input.center_x,
            input.center_y,
            planet.orbit_radius * input.orbit_scale,
            1.0,
            orbit_color,
        );

        let angle =
            (i as f32) * 1.2 + input.animation_time as f32 * 0.02 * (1.0 / (i as f32 + 1.0));
        let px = input.center_x + planet.orbit_radius * input.orbit_scale * angle.cos();
        let py = input.center_y + planet.orbit_radius * input.orbit_scale * angle.sin();
        positions.push((px, py));

        let rendered_size = (planet.size * input.orbit_scale).max(4.0);
        let target_radius = (rendered_size + 5.0).max(22.0);
        let target = Rect::new(
            px - target_radius,
            py - target_radius,
            target_radius * 2.0,
            target_radius * 2.0,
        );
        let is_hovered = input.pointer.hovering_over(target);
        let focused =
            is_hovered || input.pointer.pressing(target) || input.pointer.released_on(target);
        if focused {
            hovered_planet = Some(i);
        }

        let planet_size = if i == input.view.current_planet {
            rendered_size * 1.3
        } else {
            rendered_size
        };
        let is_colonized = input.view.colonized.get(i).copied().unwrap_or(false);

        if is_colonized {
            draw_circle(
                px,
                py,
                planet_size + (6.0 * input.orbit_scale).max(3.0),
                Color::new(0.0, 1.0, 0.5, 0.2),
            );
        }

        draw_circle(px, py, planet_size, color_from_rgba(&planet.color));
        draw_core_stage_marker(
            input.view.telemetry.get(i).copied().flatten(),
            vec2(px, py),
            planet_size,
            input.orbit_scale,
            input.animation_time,
        );

        if is_hovered {
            draw_circle_lines(
                px,
                py,
                planet_size + (4.0 * input.orbit_scale).max(3.0),
                2.0,
                Colors::PRIMARY,
            );
        }

        if i == input.view.current_planet {
            draw_circle_lines(
                px,
                py,
                planet_size + (8.0 * input.orbit_scale).max(4.0),
                2.0,
                Colors::SUCCESS,
            );
        }

        let name_color = if is_colonized {
            Colors::SUCCESS
        } else {
            Colors::TEXT_DIM
        };
        draw_ui_text(
            &planet.name,
            px - 20.0,
            py + planet_size + (15.0 * input.orbit_scale.max(0.75)),
            12.0,
            name_color,
        );
    }

    SolarMapResult {
        positions,
        hovered_planet,
    }
}
