//! Terrain tiles, structures, and motion rendering.

use super::*;

fn terrain_color_at(
    data: &crate::data::GameData,
    pos: GridPos,
    terrain: TerrainType,
    revealed: bool,
) -> Color {
    if !revealed {
        return Color::new(0.05, 0.05, 0.05, 1.0);
    }

    let def = data.terrain(terrain.id());
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
        connections[index] = state
            .grid
            .get(*neighbor)
            .is_some_and(|tile| tile.building.is_some());
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

/// Draw all visible terrain tiles, buildings, conduits, and the hover/placement preview.
pub(crate) fn draw_grid_tiles(
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
                let color = terrain_color_at(state.data(), pos, tile.terrain, tile.revealed);
                draw_rectangle(
                    px,
                    py,
                    metrics.tile_size - 1.0,
                    metrics.tile_size - 1.0,
                    color,
                );
            }

            // Draw harvestable indicator
            if tile.revealed && tile.terrain.is_harvestable(state.data()) && tile.building.is_none()
            {
                let terrain_def = state.data().terrain(tile.terrain.id());
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
            if tile.terrain == TerrainType::Mountain && tile.revealed {
                let richness = tile.ore_richness;
                let deposit_color = if richness > 1.01 {
                    Colors::ACCENT
                } else {
                    Colors::TEXT_DIM
                };
                draw_circle_lines(
                    px + metrics.tile_size * 0.5,
                    py + metrics.tile_size * 0.5,
                    metrics.tile_size * (0.20 + (richness.min(2.0) - 1.0).max(0.0) * 0.08),
                    1.0,
                    with_alpha(deposit_color, 0.7),
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
                    draw_building_motion(
                        building.building_type,
                        center_x,
                        center_y,
                        metrics.tile_size,
                        state.time_played,
                        building.overclocked,
                    );
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
                        let placement_reason = state.placement_block_reason(pos, building_type);
                        if placement_reason.is_none() {
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
                        if let Some(reason) = placement_reason {
                            if metrics.tile_size >= 28.0 {
                                draw_ui_text(
                                    reason.preview_label(),
                                    px,
                                    py - 4.0,
                                    9.0,
                                    Colors::ERROR,
                                );
                            }
                        } else if building_type == BuildingType::Bridge
                            && matches!(tile.terrain, TerrainType::Void | TerrainType::Water)
                        {
                            draw_ui_text(
                                "BRIDGE GAP - TAP TO SPAN",
                                px,
                                py - 4.0,
                                9.0,
                                Colors::ACCENT,
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

fn draw_building_motion(
    building_type: BuildingType,
    center_x: f32,
    center_y: f32,
    tile_size: f32,
    world_time: f64,
    overclocked: bool,
) {
    let motion = if macroquad_toolkit::settings::reduced_motion_enabled() {
        0.0
    } else {
        world_time as f32 * if overclocked { 1.65 } else { 1.0 }
    };
    if overclocked {
        let pulse = (motion * 2.4).sin() * 0.5 + 0.5;
        draw_circle_lines(
            center_x,
            center_y,
            tile_size * (0.36 + pulse * 0.08),
            1.2,
            with_alpha(Color::new(0.78, 0.55, 0.95, 1.0), 0.45 + pulse * 0.4),
        );
        for spark in 0..3 {
            let angle = motion * 3.0 + spark as f32 * std::f32::consts::TAU / 3.0;
            draw_circle(
                center_x + angle.cos() * tile_size * 0.34,
                center_y + angle.sin() * tile_size * 0.34,
                1.2,
                Colors::WARNING,
            );
        }
    }
    match building_type {
        BuildingType::Drill => {
            let angle = motion * 4.0;
            let tip = vec2(
                center_x + angle.cos() * tile_size * 0.28,
                center_y + angle.sin() * tile_size * 0.28,
            );
            draw_line(center_x, center_y, tip.x, tip.y, 1.5, Colors::TEXT);
        }
        BuildingType::ServerBank => {
            let blink = if (motion * 2.0).sin() > 0.0 {
                1.0
            } else {
                0.35
            };
            draw_circle(
                center_x - tile_size * 0.18,
                center_y,
                2.0,
                with_alpha(Colors::SUCCESS, blink),
            );
            draw_circle(
                center_x + tile_size * 0.18,
                center_y,
                2.0,
                with_alpha(Colors::PRIMARY, 1.0 - blink * 0.5),
            );
        }
        BuildingType::WindTurbine => {
            for spoke in 0..3 {
                let angle = motion * 3.0 + spoke as f32 * std::f32::consts::TAU / 3.0;
                draw_line(
                    center_x,
                    center_y,
                    center_x + angle.cos() * tile_size * 0.32,
                    center_y + angle.sin() * tile_size * 0.32,
                    1.0,
                    Colors::PRIMARY,
                );
            }
        }
        BuildingType::Assembler => {
            let phase = (motion * 3.0).sin() * 0.5 + 0.5;
            let reach = tile_size * (0.08 + phase * 0.11);
            let outer = tile_size * 0.28;
            draw_line(
                center_x - outer,
                center_y - tile_size * 0.14,
                center_x - reach,
                center_y,
                1.4,
                Colors::PRIMARY,
            );
            draw_line(
                center_x + outer,
                center_y - tile_size * 0.14,
                center_x + reach,
                center_y,
                1.4,
                Colors::PRIMARY,
            );
            draw_circle(
                center_x,
                center_y,
                1.5 + phase,
                with_alpha(Colors::WARNING, 0.35 + phase * 0.55),
            );
        }
        BuildingType::Smelter => {
            let phase = (motion * 2.5).sin() * 0.5 + 0.5;
            draw_line(
                center_x - tile_size * 0.2,
                center_y + tile_size * 0.14,
                center_x + tile_size * 0.2,
                center_y + tile_size * 0.14,
                1.2 + phase,
                with_alpha(Colors::ACCENT, 0.45 + phase * 0.45),
            );
            draw_circle(
                center_x,
                center_y - tile_size * 0.08,
                1.2 + phase * 1.4,
                with_alpha(Colors::WARNING, 0.4 + phase * 0.5),
            );
        }
        _ => {}
    }
}
