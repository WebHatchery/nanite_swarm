//! Evolved Core sprite selection and stage accents.

use crate::assets::GameTextures;
use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::math::pulse01_at;

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
