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
}
