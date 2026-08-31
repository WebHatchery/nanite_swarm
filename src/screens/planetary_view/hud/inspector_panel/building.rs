//! Selected-building detail, status, and action rendering.

use crate::engine::{BuildingType, TerrainType};
use crate::screens::planetary_view::format::{dust_status, fit_text_to_width, format_power_delta};
use crate::ui::{color_from_rgba, draw_hud_button, draw_status_row};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::core_status::{core_next_stage_label, core_stage_status_text};
use super::recipe_flow::{draw_recipe_flow_row, recipe_flow_data, recipe_status};
use super::terrain_layout::{draw_local_effect_labels, terrain_effect_label};
use super::{inspector_row_layout, ore_status, processor_action_label, BuildingInspectorInput};

pub(super) fn draw(input: BuildingInspectorInput<'_>) {
    let BuildingInspectorInput {
        state,
        data,
        display_pos,
        building_type,
        tile_building,
        tile_pos_with_building,
        tile_terrain,
        tile_powered,
        tile_dust,
        tile_acid,
        tile_heat,
        tile_overclocked,
        tile_standby,
        tile_can_overclock,
        tile_output_full,
        boost_unlocked,
        tile_biomass,
        tile_bonus,
        local_feature_label,
        textures,
        theme,
        colors,
        right,
        tight,
    } = input;
    let text = colors.text;
    let dim = colors.dim;
    let warning = colors.warning;
    let success = colors.success;
    let error = colors.error;
    let power_color = colors.power;
    let right_x = right.right_x;
    let right_w = right.right_w;
    let inspector_y = right.inspector.y;
    let inspector_h = right.inspector.h;

    let header_y = inspector_y + if tight { 36.0 } else { 44.0 };
    let row_layout = inspector_row_layout(inspector_y, inspector_h);
    let local_terrain_label = terrain_effect_label(tile_terrain, building_type, tile_bonus);
    let icon_size = if inspector_h < 204.0 { 54.0 } else { 72.0 };
    let icon_rect = Rect::new(right_x + 16.0, header_y, icon_size, icon_size);
    if row_layout.show_icon {
        draw_rectangle(
            icon_rect.x,
            icon_rect.y,
            icon_rect.w,
            icon_rect.h,
            color_from_rgba(&theme.colors.panel_inner),
        );
        draw_rectangle_lines(
            icon_rect.x,
            icon_rect.y,
            icon_rect.w,
            icon_rect.h,
            1.0,
            color_from_rgba(&theme.colors.border),
        );
        if let Some(icon) = textures
            .building_icons
            .by_id
            .get(building_type.id())
            .or_else(|| textures.buildings.by_id.get(building_type.id()))
        {
            draw_texture_ex(
                icon,
                icon_rect.x + 6.0,
                icon_rect.y + 6.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(icon_size - 12.0, icon_size - 12.0)),
                    ..Default::default()
                },
            );
        }
    }
    let info_x = if row_layout.show_icon {
        icon_rect.x + icon_rect.w + 14.0
    } else {
        right_x + 16.0
    };
    draw_ui_text(
        building_type.name(data),
        info_x,
        header_y + 16.0,
        13.0,
        text,
    );
    let description = fit_text_to_width(
        building_type.description(data),
        right_x + right_w - info_x - 14.0,
        if row_layout.show_icon { 10.0 } else { 9.0 },
    );
    draw_ui_text(
        &description,
        info_x,
        header_y
            + if row_layout.show_icon {
                40.0
            } else if tight {
                27.0
            } else {
                32.0
            },
        if row_layout.show_icon { 10.0 } else { 9.0 },
        dim,
    );
    draw_local_effect_labels(
        row_layout.show_icon,
        row_layout.row_base,
        header_y,
        &[local_feature_label, local_terrain_label.as_deref()],
        info_x,
        right_x + right_w - info_x - 14.0,
        colors.primary,
    );
    let output = tile_pos_with_building
        .and_then(|pos| state.building_power_delta(pos))
        .map(format_power_delta)
        .unwrap_or_else(|| {
            let power_multiplier = if tile_overclocked { 1.75 } else { 1.0 };
            format_power_delta(building_type.power_delta(data) * power_multiplier)
        });
    let row_base = row_layout.row_base;
    let row_gap = row_layout.row_gap;
    draw_status_row(
        theme,
        right_x + 16.0,
        row_base,
        right_w - 32.0,
        "Power",
        &output,
        power_color,
    );
    let (dust_label, dust_color) = dust_status(tile_dust);
    draw_status_row(
        theme,
        right_x + 16.0,
        row_base + row_gap,
        right_w - 32.0,
        "Dust",
        &format!("{:.0}% {}", tile_dust, dust_label),
        dust_color,
    );

    // What the ground itself is worth. Without this a deposit is invisible
    // and drill placement is guesswork.
    let has_recipe = !data.building(building_type.id()).recipe.is_empty();
    if row_layout.show_detail_rows && !has_recipe {
        if building_type == BuildingType::Core {
            draw_status_row(
                theme,
                right_x + 16.0,
                row_base + row_gap * 3.0,
                right_w - 32.0,
                "Next",
                &core_next_stage_label(state),
                colors.primary,
            );
        } else if building_type == BuildingType::BiomassHarvester
            && tile_terrain == Some(TerrainType::Forest)
        {
            let fuel = if tile_biomass > 0.0 {
                format!("{:.0} biomass", tile_biomass)
            } else {
                "DEPLETED".to_string()
            };
            draw_status_row(
                theme,
                right_x + 16.0,
                row_base + row_gap * 3.0,
                right_w - 32.0,
                "Fuel",
                &fuel,
                if tile_biomass > 0.0 { warning } else { error },
            );
        } else if let Some(tile) = display_pos.and_then(|pos| state.grid.get(pos)) {
            let (ore_label, ore_color) = ore_status(tile.ore_richness, colors);
            draw_status_row(
                theme,
                right_x + 16.0,
                row_base + row_gap * 3.0,
                right_w - 32.0,
                "Ore",
                &format!("{:.0}% {}", tile.ore_richness * 100.0, ore_label),
                ore_color,
            );
        }
    }

    let mut status_text = if tile_building.is_some() {
        if tile_powered {
            "Powered".to_string()
        } else {
            "No power".to_string()
        }
    } else {
        "Blueprint".to_string()
    };
    // What is waiting on a building is the difference between "idle" and
    // "waiting on a drone", and the player cannot tell them apart otherwise.
    if let Some(pos) = display_pos {
        let key = (pos.x, pos.y);
        let waiting_in = state.input_buffers.get(&key).copied().unwrap_or(0.0);
        let waiting_out = state.output_buffers.get(&key).copied().unwrap_or(0.0);
        if let Some(flow) = recipe_flow_data(state, pos, building_type) {
            status_text = recipe_status(&flow, tile_powered);
        } else if waiting_in >= 1.0 {
            status_text = format!("{} - {:.0} in", status_text, waiting_in);
        }
        if waiting_out >= 1.0 && data.building(building_type.id()).recipe.is_empty() {
            status_text = format!("{} - {:.0} out", status_text, waiting_out);
        }
        let queued = state
            .drone_queues
            .values()
            .filter(|queue| {
                queue.iter().any(|id| {
                    state
                        .drones
                        .drones()
                        .iter()
                        .any(|drone| drone.id == *id && drone.home == pos)
                })
            })
            .map(Vec::len)
            .sum::<usize>();
        if queued > 0 {
            status_text = format!("{} - queue {}", status_text, queued);
        }
    }
    // What a driver is throwing and what a pad is holding both matter more
    // than the word "Powered" does, and the Power row above already says
    // whether it has any.
    if let Some(tile_pos) = tile_pos_with_building {
        match building_type {
            BuildingType::MassDriver => status_text = state.export_summary(tile_pos),
            BuildingType::LandingPad => status_text = state.pad_summary(tile_pos),
            BuildingType::Core => status_text = core_stage_status_text(state),
            _ => {}
        }
    }
    if tile_acid > 0.0 {
        status_text = format!("{} - acid {:.0}%", status_text, tile_acid);
    }
    if tile_heat > 0.0 {
        status_text = format!(
            "{} - heat {:.0}/{:.0}",
            status_text, tile_heat, state.config.buildings.server_bank_heat_capacity
        );
    }
    let status_color = if tile_building.is_some() && !tile_powered {
        error
    } else if status_text.starts_with("Needs ") {
        warning
    } else {
        success
    };
    let status_text = fit_text_to_width(&status_text, right_w * 0.52, theme.typography.body);
    draw_status_row(
        theme,
        right_x + 16.0,
        row_base + row_gap * 2.0,
        right_w - 32.0,
        "Status",
        &status_text,
        status_color,
    );
    if row_layout.show_detail_rows {
        if let Some(pos) = tile_pos_with_building {
            if let Some(flow) = recipe_flow_data(state, pos, building_type) {
                let (priority_toggled, standby_toggled) = draw_recipe_flow_row(
                    &flow,
                    theme,
                    right_x + 16.0,
                    row_base + row_gap * 3.0,
                    right_w - 32.0,
                    dim,
                    warning,
                );
                if priority_toggled {
                    state.toggle_input_priority(pos);
                }
                if standby_toggled {
                    state.toggle_processor_standby(pos);
                }
            }
        }
    }
    if let Some(tile_pos) = tile_pos_with_building {
        if tile_can_overclock
            && draw_hud_button(
                theme,
                Rect::new(right_x + right_w - 180.0, inspector_y + 8.0, 92.0, 24.0),
                processor_action_label(
                    tile_output_full,
                    state.purge_armed == Some(tile_pos),
                    boost_unlocked,
                    tile_standby,
                    tile_overclocked,
                ),
            )
        {
            if tile_output_full {
                state.request_processor_pad_purge(tile_pos);
            } else {
                state.toggle_building_overclock(tile_pos);
            }
        }
        if building_type != BuildingType::Core
            && draw_hud_button(
                theme,
                Rect::new(right_x + right_w - 82.0, inspector_y + 8.0, 62.0, 24.0),
                "SELL",
            )
        {
            state.try_sell_building(tile_pos);
        }
    }
}
