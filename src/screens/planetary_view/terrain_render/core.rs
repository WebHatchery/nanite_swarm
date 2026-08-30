//! Evolved Core sprite selection and stage accents.

use crate::assets::GameTextures;
use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::math::pulse01_at;

use super::super::metrics::{grid_to_screen, HudMetrics};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CoreStageAmbient {
    Surface,
    SpaceElevator,
    PlanetaryRing,
}

fn core_stage_ambient(stage: usize) -> CoreStageAmbient {
    match stage {
        0..=2 => CoreStageAmbient::Surface,
        3 => CoreStageAmbient::SpaceElevator,
        _ => CoreStageAmbient::PlanetaryRing,
    }
}

pub(super) fn draw_core_visual(
    px: f32,
    py: f32,
    size: f32,
    state: &PlanetState,
    textures: &GameTextures,
) {
    let stage = state.core_stage_index();
    let center_x = px + size * 0.5;
    let center_y = py + size * 0.5;
    let pulse = pulse01_at(state.time_played, 2.0);
    let sprite = state
        .core_stage_def()
        .and_then(|definition| definition.sprite.as_deref());
    let texture = match sprite.unwrap_or("") {
        "core_stage_1a" => &textures.buildings.core_stage_1a,
        "core_stage_1b" => &textures.buildings.core_stage_1b,
        "core_stage_1c" => &textures.buildings.core_stage_1c,
        "core_stage_2a" => &textures.buildings.core_stage_2a,
        "core_stage_2b" => &textures.buildings.core_stage_2b,
        "core_stage_3a" => &textures.buildings.core_stage_3a,
        "core_stage_3b" => &textures.buildings.core_stage_3b,
        "core_stage_4a" => &textures.buildings.core_stage_4a,
        "core_stage_4b" => &textures.buildings.core_stage_4b,
        _ => match stage {
            0 => &textures.buildings.core_stage_1a,
            1 => &textures.buildings.core_stage_1b,
            2 => &textures.buildings.core_stage_1c,
            3 => &textures.buildings.core_stage_2a,
            4 => &textures.buildings.core_stage_2b,
            5 => &textures.buildings.core_stage_3a,
            6 => &textures.buildings.core_stage_3b,
            7 => &textures.buildings.core_stage_4a,
            _ => &textures.buildings.core_stage_4b,
        },
    };
    draw_texture_ex(
        texture,
        px,
        py,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(size - 1.0, size - 1.0)),
            ..Default::default()
        },
    );
    if stage >= 1 {
        draw_circle_lines(center_x, center_y, 7.0, 1.0, Colors::ACCENT);
    }
    if stage >= 2 {
        draw_line(
            center_x - 6.0,
            center_y,
            center_x + 6.0,
            center_y,
            1.0,
            Colors::TEXT,
        );
        draw_line(
            center_x,
            center_y - 6.0,
            center_x,
            center_y + 6.0,
            1.0,
            Colors::TEXT,
        );
    }
    if stage >= 3 {
        draw_circle_lines(
            center_x,
            center_y,
            11.0,
            1.0,
            with_alpha(Colors::PRIMARY, 0.2 + pulse * 0.2),
        );
    }

    // The Core now reflects the physical depth of the factory, not only its
    // abstract evolution milestone. Each opened deck adds a thin animated
    // ring, making a growing production chain readable even when the HUD is
    // collapsed for route planning.
    let depth = state.factory_depth();
    for deck in 0..depth {
        let radius = 14.0 + deck as f32 * 4.5;
        let phase = state.time_played as f32 * (1.2 + deck as f32 * 0.18);
        let start = phase + deck as f32 * 1.7;
        let end = start + std::f32::consts::PI * (0.7 + pulse * 0.2);
        draw_depth_arc(
            center_x,
            center_y,
            radius,
            start,
            end,
            with_alpha(Colors::PRIMARY, 0.55),
        );
        draw_depth_arc(
            center_x,
            center_y,
            radius,
            start + std::f32::consts::PI,
            end + std::f32::consts::PI,
            with_alpha(Colors::ACCENT, 0.35),
        );
    }
}

fn draw_depth_arc(cx: f32, cy: f32, radius: f32, start: f32, end: f32, color: Color) {
    let segments = 10;
    let mut previous = vec2(cx + start.cos() * radius, cy + start.sin() * radius);
    for index in 1..=segments {
        let ratio = index as f32 / segments as f32;
        let angle = start + (end - start) * ratio;
        let next = vec2(cx + angle.cos() * radius, cy + angle.sin() * radius);
        draw_line(previous.x, previous.y, next.x, next.y, 1.0, color);
        previous = next;
    }
}

/// Draw the map-scale silhouette promised by the Core's late stages.
///
/// These marks deliberately use the live Core tile as their anchor, so they
/// follow camera pan and zoom just like the grid and never become a detached
/// screen decoration. They sit above terrain but below drones and the HUD.
pub(super) fn draw_core_stage_ambient(state: &PlanetState, metrics: HudMetrics, time: f32) {
    let Some(core) = state.grid.find_core() else {
        return;
    };
    let (px, py) = grid_to_screen(core, metrics);
    let center = vec2(px + metrics.tile_size * 0.5, py + metrics.tile_size * 0.5);
    let motion = if macroquad_toolkit::settings::reduced_motion_enabled() {
        0.0
    } else {
        1.0
    };
    let pulse = pulse01_at(time as f64, 1.8) * motion;

    match core_stage_ambient(state.core_stage_index()) {
        CoreStageAmbient::Surface => {}
        CoreStageAmbient::SpaceElevator => draw_space_elevator(center, metrics, time, pulse),
        CoreStageAmbient::PlanetaryRing => {
            draw_space_elevator(center, metrics, time, pulse);
            draw_planetary_ring(center, metrics, time, pulse);
        }
    }
}

fn draw_space_elevator(center: Vec2, metrics: HudMetrics, time: f32, pulse: f32) {
    let top = metrics.base_offset_y() + metrics.panel_gap + 4.0;
    let anchor = center.y - metrics.tile_size * 0.48;
    if anchor <= top {
        return;
    }

    let rail_offset = (metrics.tile_size * 0.28).clamp(2.5, 8.0);
    let rail_width = (metrics.tile_size * 0.055).clamp(1.0, 2.5);
    let glow = with_alpha(Colors::PRIMARY_SOFT, 0.10 + pulse * 0.06);
    let rail = with_alpha(Colors::PRIMARY, 0.52 + pulse * 0.16);
    draw_line(
        center.x - rail_offset,
        top,
        center.x - rail_offset,
        anchor,
        rail_width + 3.0,
        glow,
    );
    draw_line(
        center.x + rail_offset,
        top,
        center.x + rail_offset,
        anchor,
        rail_width + 3.0,
        glow,
    );
    draw_line(
        center.x - rail_offset,
        top,
        center.x - rail_offset,
        anchor,
        rail_width,
        rail,
    );
    draw_line(
        center.x + rail_offset,
        top,
        center.x + rail_offset,
        anchor,
        rail_width,
        rail,
    );

    let span = (metrics.tile_size * 0.72).clamp(10.0, 22.0);
    draw_line(
        center.x - span,
        top + 8.0,
        center.x + span,
        top + 8.0,
        1.5,
        with_alpha(Colors::ACCENT, 0.55),
    );
    draw_circle(
        center.x,
        top + 8.0,
        (2.0 + pulse * 1.5).min(3.5),
        Colors::ACCENT,
    );

    // Crossbars and signal nodes make the tether read as constructed
    // infrastructure instead of a stray vertical line in the map.
    let levels = 4;
    for index in 1..=levels {
        let ratio = index as f32 / (levels + 1) as f32;
        let y = top + (anchor - top) * ratio;
        let half_span = (metrics.tile_size * 0.18).clamp(4.0, 9.0);
        draw_line(
            center.x - rail_offset - half_span,
            y,
            center.x + rail_offset + half_span,
            y,
            1.0,
            with_alpha(Colors::PRIMARY_SOFT, 0.32 + pulse * 0.12),
        );
        let signal_ratio = (time * 0.16 + index as f32 * 0.23).fract();
        let signal_y = top + (anchor - top) * signal_ratio;
        draw_circle(
            center.x,
            signal_y,
            1.8 + pulse * 1.2,
            with_alpha(Colors::ACCENT, 0.70),
        );
    }
}

fn draw_planetary_ring(center: Vec2, metrics: HudMetrics, time: f32, pulse: f32) {
    let width = (metrics.tile_size * 13.0).clamp(260.0, 560.0);
    let height = (metrics.tile_size * 3.4).clamp(68.0, 150.0);
    let rotation = -15.0 + (time * 0.08).sin() * motion_rotation(pulse);
    draw_ellipse_lines(
        center.x,
        center.y,
        width,
        height,
        rotation,
        2.0,
        with_alpha(Colors::PRIMARY, 0.42 + pulse * 0.14),
    );
    draw_ellipse_lines(
        center.x,
        center.y,
        width * 0.91,
        height * 0.62,
        rotation + 3.0,
        1.0,
        with_alpha(Colors::ACCENT, 0.30 + pulse * 0.10),
    );

    let radius_x = width * 0.5;
    let radius_y = height * 0.5;
    for (index, angle) in [0.25, 1.85, 3.35, 5.05].into_iter().enumerate() {
        let position = ellipse_point(center, radius_x, radius_y, angle, rotation);
        draw_circle(
            position.x,
            position.y,
            4.5,
            with_alpha(Colors::BACKGROUND, 0.85),
        );
        let color = if index % 2 == 0 {
            Colors::PRIMARY
        } else {
            Colors::ACCENT
        };
        draw_circle(position.x, position.y, 2.4 + pulse * 0.8, color);
    }
}

fn motion_rotation(pulse: f32) -> f32 {
    pulse * 1.8
}

fn ellipse_point(center: Vec2, radius_x: f32, radius_y: f32, angle: f32, rotation: f32) -> Vec2 {
    let angle = angle + rotation.to_radians();
    vec2(
        center.x + angle.cos() * radius_x,
        center.y + angle.sin() * radius_y,
    )
}

#[cfg(test)]
mod tests;
