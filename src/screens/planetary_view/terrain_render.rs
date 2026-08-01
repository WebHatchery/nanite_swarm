//! Background, terrain, conduit, and building rendering for the grid

use crate::assets::GameTextures;
use crate::data;
use crate::engine::{BuildingType, GridPos, TerrainType};
use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::math::{lerp, pulse01_at};
use macroquad_toolkit::ui::draw_ui_text;

use super::format::hash01;
use super::metrics::{grid_to_screen, HudMetrics};

pub(super) fn draw_planetary_background(screen_w: f32, screen_h: f32, time: f32) {
    // Subtle star field
    for i in 0..80u32 {
        let star_x = hash01(i) * screen_w;
        let star_y = hash01(i + 17) * screen_h;
        let twinkle = lerp(0.5, 1.0, pulse01_at(time as f64 + 2.0 * i as f64, 0.5));
        let color = Color::new(0.6, 0.7, 0.8, 0.15 + twinkle * 0.2);
        draw_circle(star_x, star_y, 1.0 + hash01(i + 31), color);
    }

    // Planet glow
    let planet_x = screen_w * 0.82;
    let planet_y = screen_h * 0.85;
    let glow = lerp(0.25, 0.35, pulse01_at(time as f64, 0.6));
    draw_circle(planet_x, planet_y, 220.0, Color::new(0.0, 0.2, 0.35, 0.12));
    draw_circle(planet_x, planet_y, 170.0, Color::new(0.0, 0.3, 0.45, glow));
    draw_circle(planet_x, planet_y, 120.0, Color::new(0.0, 0.25, 0.4, 0.25));
}

/// Get color for terrain type with subtle variation
fn terrain_color_at(pos: GridPos, terrain: TerrainType, revealed: bool) -> Color {
    if !revealed {
        return Color::new(0.05, 0.05, 0.05, 1.0);
    }

    let def = data::game_data().terrain(terrain.id());
    let mut color = Color::new(def.color[0], def.color[1], def.color[2], def.color[3]);

    if terrain == TerrainType::Empty {
        let noise =
            hash01((pos.x as u32).wrapping_mul(73856093) ^ (pos.y as u32).wrapping_mul(19349663));
        let offset = noise * 0.04;
        color = Color::new(
            (color.r + offset).min(1.0),
            (color.g + offset).min(1.0),
            (color.b + offset * 0.8).min(1.0),
            color.a,
        );
    }

    color
}

fn terrain_texture(terrain: TerrainType, textures: &GameTextures) -> &Texture2D {
    let id = terrain.id();
    textures
        .terrain
        .by_id
        .get(id)
        .unwrap_or(&textures.terrain.by_id["empty"])
}

fn building_texture(building_type: BuildingType, textures: &GameTextures) -> &Texture2D {
    textures
        .buildings
        .by_id
        .get(building_type.id())
        .unwrap_or(&textures.buildings.core_stage_1a)
}

fn conduit_texture<'a>(connections: &[bool; 4], textures: &'a GameTextures) -> &'a Texture2D {
    let n = connections[0];
    let e = connections[1];
    let s = connections[2];
    let w = connections[3];

    let count = connections.iter().filter(|c| **c).count();
    match count {
        4 => &textures.buildings.conduit_cross,
        3 => {
            if !n {
                &textures.buildings.conduit_tee_s
            } else if !e {
                &textures.buildings.conduit_tee_w
            } else if !s {
                &textures.buildings.conduit_tee_n
            } else {
                &textures.buildings.conduit_tee_e
            }
        }
        2 => {
            if (n && s) && !e && !w {
                &textures.buildings.conduit_straight_v
            } else if (e && w) && !n && !s {
                &textures.buildings.conduit_straight_h
            } else if n && e {
                &textures.buildings.conduit_corner_ne
            } else if n && w {
                &textures.buildings.conduit_corner_nw
            } else if s && e {
                &textures.buildings.conduit_corner_se
            } else if s && w {
                &textures.buildings.conduit_corner_sw
            } else if n || s {
                &textures.buildings.conduit_straight_v
            } else {
                &textures.buildings.conduit_straight_h
            }
        }
        1 => {
            if n || s {
                &textures.buildings.conduit_straight_v
            } else {
                &textures.buildings.conduit_straight_h
            }
        }
        _ => &textures.buildings.conduit_straight_h,
    }
}

fn conduit_connections(state: &PlanetState, pos: GridPos) -> [bool; 4] {
    let dirs = [
        GridPos::new(pos.x, pos.y - 1),
        GridPos::new(pos.x + 1, pos.y),
        GridPos::new(pos.x, pos.y + 1),
        GridPos::new(pos.x - 1, pos.y),
    ];

    let mut connections = [false; 4];
    for (index, neighbor) in dirs.iter().enumerate() {
        if let Some(tile) = state.grid.get(*neighbor) {
            if let Some(ref building) = tile.building {
                connections[index] = matches!(
                    building.building_type,
                    BuildingType::Conduit
                        | BuildingType::Bridge
                        | BuildingType::Core
                        | BuildingType::Drill
                        | BuildingType::PowerNode
                        | BuildingType::WindTurbine
                        | BuildingType::ServerBank
                        | BuildingType::Sweeper
                        | BuildingType::Storage
                        | BuildingType::BiomassHarvester
                );
            }
        }
    }
    connections
}

fn draw_conduit_tile(
    px: f32,
    py: f32,
    pos: GridPos,
    state: &PlanetState,
    brightness: f32,
    textures: &GameTextures,
    metrics: HudMetrics,
) {
    let connections = conduit_connections(state, pos);
    let tint = Color::new(brightness, brightness, brightness, 1.0);
    let texture = conduit_texture(&connections, textures);
    draw_texture_ex(
        texture,
        px,
        py,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(metrics.tile_size - 1.0, metrics.tile_size - 1.0)),
            ..Default::default()
        },
    );
}

/// Determine current Core evolution stage based on progress
fn core_stage(state: &PlanetState) -> u8 {
    let growth = state.time_played as f32 + (state.resources.minerals + state.resources.data) * 0.4;

    if growth < 60.0 {
        0
    } else if growth < 120.0 {
        1
    } else if growth < 200.0 {
        2
    } else if growth < 320.0 {
        3
    } else {
        4
    }
}

/// Draw evolved Core visuals
fn draw_core_visual(px: f32, py: f32, size: f32, state: &PlanetState, textures: &GameTextures) {
    let stage = core_stage(state);
    let center_x = px + size * 0.5;
    let center_y = py + size * 0.5;
    let pulse = pulse01_at(state.time_played, 2.0);

    let texture = match stage {
        0 => &textures.buildings.core_stage_1a,
        1 => &textures.buildings.core_stage_1b,
        2 => &textures.buildings.core_stage_1c,
        3 => &textures.buildings.core_stage_2a,
        _ => &textures.buildings.core_stage_2b,
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
        let glow_alpha = 0.2 + pulse * 0.2;
        draw_circle_lines(
            center_x,
            center_y,
            11.0,
            1.0,
            with_alpha(Colors::PRIMARY, glow_alpha),
        );
    }
}

/// Draw all visible terrain tiles, buildings, conduits, and the hover/placement preview.
pub(super) fn draw_grid_tiles(
    state: &PlanetState,
    textures: &GameTextures,
    metrics: HudMetrics,
    hovered_pos: Option<GridPos>,
    pulse: f32,
    global_pulse: f32,
) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    let min_x = ((0.0 - metrics.grid_offset_x()) / metrics.tile_size).floor() as i32 - 1;
    let min_y = ((0.0 - metrics.grid_offset_y()) / metrics.tile_size).floor() as i32 - 1;
    let max_x = ((screen_w - metrics.grid_offset_x()) / metrics.tile_size).ceil() as i32 + 1;
    let max_y = ((screen_h - metrics.grid_offset_y()) / metrics.tile_size).ceil() as i32 + 1;
    let min_x = min_x.max(0);
    let min_y = min_y.max(0);
    let max_x = max_x.min(state.grid.width as i32 - 1);
    let max_y = max_y.min(state.grid.height as i32 - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let pos = GridPos::new(x, y);
            let Some(tile) = state.grid.get(pos) else {
                continue;
            };
            let (px, py) = grid_to_screen(pos, metrics);

            // Draw terrain
            if tile.revealed {
                let texture = terrain_texture(tile.terrain, textures);
                draw_texture_ex(
                    texture,
                    px,
                    py,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(metrics.tile_size - 1.0, metrics.tile_size - 1.0)),
                        ..Default::default()
                    },
                );
            } else {
                let color = terrain_color_at(pos, tile.terrain, tile.revealed);
                draw_rectangle(
                    px,
                    py,
                    metrics.tile_size - 1.0,
                    metrics.tile_size - 1.0,
                    color,
                );
            }

            // Draw harvestable indicator
            if tile.revealed && tile.terrain.is_harvestable() && tile.building.is_none() {
                let terrain_def = data::game_data().terrain(tile.terrain.id());
                let indicator_color = Color::new(
                    terrain_def.color[0],
                    terrain_def.color[1],
                    terrain_def.color[2],
                    0.5,
                );
                draw_rectangle_lines(
                    px + 2.0,
                    py + 2.0,
                    metrics.tile_size - 5.0,
                    metrics.tile_size - 5.0,
                    1.0,
                    indicator_color,
                );
            }

            if tile.filter {
                let filter_color = Color::new(0.2, 0.8, 0.6, 0.7);
                draw_circle_lines(
                    px + metrics.tile_size * 0.5,
                    py + metrics.tile_size * 0.5,
                    6.0,
                    1.0,
                    filter_color,
                );
            }
            if tile.forest_cleared {
                let scar_color = Color::new(0.8, 0.4, 0.2, 0.6);
                draw_circle_lines(
                    px + metrics.tile_size * 0.5,
                    py + metrics.tile_size * 0.5,
                    8.0,
                    1.0,
                    scar_color,
                );
            }

            // Draw building if present
            if let Some(ref building) = tile.building {
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
                }

                // Unpowered indicator
                if !building.powered && building.building_type != BuildingType::Core {
                    draw_ui_text("!", px + 18.0, py + 10.0, 12.0, Colors::ERROR);
                }
            }

            // Draw hover highlight
            if let Some(hover) = hovered_pos {
                if hover == pos && tile.revealed {
                    let line_thickness = 1.5 + pulse * 1.5;
                    // The cursor has to say what the click will do: red over
                    // something a click would tear down.
                    let doomed = state.demolish_mode && tile.building.is_some();
                    draw_rectangle_lines(
                        px,
                        py,
                        metrics.tile_size - 1.0,
                        metrics.tile_size - 1.0,
                        line_thickness,
                        if doomed {
                            Colors::ERROR
                        } else {
                            Colors::PRIMARY
                        },
                    );
                    if doomed {
                        draw_rectangle(
                            px,
                            py,
                            metrics.tile_size - 1.0,
                            metrics.tile_size - 1.0,
                            Color::new(1.0, 0.2, 0.2, 0.18 + pulse * 0.12),
                        );
                    }

                    // Show placement preview
                    if let Some(building_type) = state.selected_building {
                        let preview_alpha = 0.2 + pulse * 0.15;
                        if state.grid.can_place_building(pos, building_type) {
                            let preview_color = Color::new(0.0, 0.8, 1.0, preview_alpha);
                            draw_rectangle(
                                px,
                                py,
                                metrics.tile_size - 1.0,
                                metrics.tile_size - 1.0,
                                preview_color,
                            );
                        } else {
                            let invalid_color = Color::new(1.0, 0.2, 0.2, preview_alpha);
                            draw_rectangle(
                                px,
                                py,
                                metrics.tile_size - 1.0,
                                metrics.tile_size - 1.0,
                                invalid_color,
                            );
                        }
                    }

                    // Show harvest preview
                    if state.can_harvest(pos) {
                        let harvest_alpha = 0.2 + pulse * 0.15;
                        let harvest_color = Color::new(1.0, 0.5, 0.0, harvest_alpha);
                        draw_rectangle(
                            px,
                            py,
                            metrics.tile_size - 1.0,
                            metrics.tile_size - 1.0,
                            harvest_color,
                        );
                    }
                }
            }
        }
    }
}
