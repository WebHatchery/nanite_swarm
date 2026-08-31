//! Building textures, topology marks, motion, and power-state visuals.

use super::{building_texture, draw_conduit_tile, motion};
use crate::assets::GameTextures;
use crate::engine::{BuildingType, GridPos, TerrainType, Tile};
use crate::screens::planetary_view::metrics::HudMetrics;
use crate::screens::planetary_view::terrain_render::draw_core_visual;
use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::draw_ui_text;

pub(super) struct BuildingTileInput<'a> {
    pub tile: &'a Tile,
    pub pos: GridPos,
    pub px: f32,
    pub py: f32,
    pub state: &'a PlanetState,
    pub textures: &'a GameTextures,
    pub metrics: HudMetrics,
    pub global_pulse: f32,
}

pub(super) fn draw_building_tile(input: BuildingTileInput<'_>) {
    let BuildingTileInput {
        tile,
        pos,
        px,
        py,
        state,
        textures,
        metrics,
        global_pulse,
    } = input;
    let Some(building) = tile.building.as_ref() else {
        return;
    };
    if building.building_type == BuildingType::Core {
        draw_core_visual(px, py, metrics.tile_size, state, textures);
    } else {
        let brightness = if building.powered { global_pulse } else { 0.6 };
        let tint = Color::new(brightness, brightness, brightness, 1.0);
        let margin = 2.0;
        let scale = state.placement_scale(pos);
        let size = (metrics.tile_size - margin * 2.0 - 1.0) * scale;
        let offset = (metrics.tile_size - margin * 2.0 - 1.0 - size) * 0.5;
        let box_x = px + margin + offset;
        let box_y = py + margin + offset;

        let center_x = px + metrics.tile_size * 0.5;
        let center_y = py + metrics.tile_size * 0.5;

        if building.building_type == BuildingType::Conduit {
            draw_conduit_tile(px, py, pos, state, brightness, textures, metrics);
        } else if building.building_type == BuildingType::Bridge {
            let texture = building_texture(building.building_type, textures);
            draw_texture_ex(
                texture,
                box_x,
                box_y,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(size, size)),
                    ..Default::default()
                },
            );
            if matches!(tile.terrain, TerrainType::Void | TerrainType::Water) {
                draw_line(
                    px + 2.0,
                    py + metrics.tile_size - 4.0,
                    px + metrics.tile_size - 2.0,
                    py + 4.0,
                    2.0,
                    Colors::ACCENT,
                );
                draw_line(
                    px + 2.0,
                    py + 4.0,
                    px + metrics.tile_size - 2.0,
                    py + metrics.tile_size - 4.0,
                    1.0,
                    with_alpha(Colors::TEXT, 0.7),
                );
            }
        } else {
            let texture = building_texture(building.building_type, textures);
            draw_texture_ex(
                texture,
                box_x,
                box_y,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(size, size)),
                    ..Default::default()
                },
            );
        }

        if building.building_type == BuildingType::PowerNode && building.powered {
            let glow_color = Color::new(0.0, 0.85, 1.0, 0.18);
            draw_circle(center_x, center_y, metrics.tile_size * 2.5, glow_color);
            draw_circle(
                center_x,
                center_y,
                metrics.tile_size * 1.8,
                Color::new(0.0, 0.85, 1.0, 0.28),
            );
        }
        motion::draw_building_motion(
            building.building_type,
            center_x,
            center_y,
            metrics.tile_size,
            state.time_played,
            building.overclocked,
        );
    }

    if !building.powered && building.building_type != BuildingType::Core {
        draw_ui_text("!", px + 18.0, py + 10.0, 12.0, Colors::ERROR);
    }
}
