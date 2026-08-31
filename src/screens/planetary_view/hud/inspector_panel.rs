//! Right sidebar: "SELECTED STRUCTURE" inspector panel

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::engine::{BuildingType, GridPos, TerrainType};
use crate::state::{Milestone, PlanetState};
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel, draw_status_row};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::format::{dust_status, fit_text_to_width, format_power_delta};
use super::{PanelColors, RightStackLayout};

mod recipe_flow;
mod terrain_layout;
use recipe_flow::{draw_recipe_flow_row, recipe_flow_data, recipe_status};
use terrain_layout::{
    draw_local_effect_labels, feature_label_for, feature_survey_label, terrain_detail_offsets,
    terrain_effect_label, terrain_feature_lines,
};

fn processor_action_label(
    output_full: bool,
    purge_armed: bool,
    boost_unlocked: bool,
    standby: bool,
    overclocked: bool,
) -> &'static str {
    if output_full && purge_armed {
        "PURGE AGAIN"
    } else if output_full {
        "PURGE PAD"
    } else if !boost_unlocked {
        "BOOST LOCKED"
    } else if standby {
        "BOOST PAUSED"
    } else if overclocked {
        "BOOST ON"
    } else {
        "BOOST"
    }
}

pub(super) fn draw(
    state: &mut PlanetState,
    hovered_pos: Option<GridPos>,
    textures: &GameTextures,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
) {
    let data_context = state.data.clone();
    let data = data_context.data();
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
    let tight = right.compact && inspector_h < 140.0;

    draw_hud_panel(theme, right.inspector, Some("SELECTED STRUCTURE"));

    let display_pos = hovered_pos.or(state.selected_tile);
    let mut tile_building = None;
    let mut tile_pos_with_building = None;
    let mut tile_terrain = None;
    let mut tile_powered = false;
    let mut tile_dust = 0.0;
    let mut tile_acid = 0.0;
    let mut tile_heat = 0.0;
    let mut tile_overclocked = false;
    let mut tile_standby = false;
    let mut tile_can_overclock = false;
    let mut tile_output_full = false;
    let boost_unlocked = state
        .research
        .unlocked_techs
        .iter()
        .any(|tech| tech == "adaptive_clocking");
    let mut tile_harvest = None;
    let mut tile_bonus = None;
    let mut tile_biomass = 0.0;
    if let Some(tile_pos) = display_pos {
        if let Some(tile) = state.grid.get(tile_pos) {
            tile_terrain = Some(tile.terrain);
            tile_harvest = state.harvest_rewards_at(tile_pos);
            tile_bonus = tile.terrain.preservation_bonus(data);
            tile_biomass = tile.biomass_amount;
            if let Some(building) = &tile.building {
                tile_building = Some(building.building_type);
                tile_pos_with_building = Some(tile_pos);
                tile_powered = building.powered;
                tile_dust = building.dust;
                tile_acid = building.acid_wear;
                tile_heat = building.heat;
                tile_overclocked = building.overclocked;
                tile_standby = building.standby;
                tile_can_overclock = building.supports_overclock();
                tile_output_full = state
                    .output_buffers
                    .get(&(tile_pos.x, tile_pos.y))
                    .copied()
                    .unwrap_or(0.0)
                    >= state.processor_pad_capacity() - 0.001;
            }
        }
    }
    let tile_feature = display_pos.and_then(|pos| state.feature_at(pos)).cloned();
    let feature_survey_progress = display_pos.and_then(|pos| state.feature_survey_progress_at(pos));
    let feature_effects = display_pos.and_then(|pos| state.feature_effects_at(pos));
    let local_feature_label = feature_label_for(tile_feature.as_ref(), feature_effects)
        .map(|label| feature_survey_label(&label, feature_survey_progress));

    let inspected_building = tile_building.or(state.selected_building);
    if let Some(building_type) = inspected_building {
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
            &[
                local_feature_label.as_deref(),
                local_terrain_label.as_deref(),
            ],
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
        if let Some(pos) = hovered_pos.or(state.selected_tile) {
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
    } else {
        let feature_lines = terrain_feature_lines(inspector_h, tile_feature.is_some());
        let feature_shift = feature_lines as f32 * 16.0;
        let (terrain_y, reward_y, bonus_y) = terrain_detail_offsets(inspector_h);
        draw_ui_text(
            "NO STRUCTURE",
            right_x + 16.0,
            inspector_y + if tight { 44.0 } else { 56.0 },
            13.0,
            text,
        );
        if let Some(feature) = tile_feature.as_ref() {
            if feature_lines > 0 {
                let feature_name = local_feature_label.as_deref().unwrap_or("Feature");
                draw_ui_text(
                    &fit_text_to_width(feature_name, right_w - 32.0, 10.0),
                    right_x + 16.0,
                    inspector_y + terrain_y - 14.0,
                    10.0,
                    colors.primary,
                );
                if feature_lines > 1 {
                    draw_ui_text(
                        &fit_text_to_width(&feature.description, right_w - 32.0, 9.0),
                        right_x + 16.0,
                        inspector_y + terrain_y + 1.0,
                        9.0,
                        dim,
                    );
                }
            }
        }
        if let Some(terrain) = tile_terrain {
            draw_ui_text(
                &format!("Terrain: {}", terrain.name(data)),
                right_x + 16.0,
                inspector_y + terrain_y + feature_shift,
                11.0,
                dim,
            );
            if terrain.is_harvestable(data) {
                let (minerals, biomass) = tile_harvest.unwrap_or((0.0, 0.0));
                let reward_text = if minerals > 0.0 {
                    format!("Harvest +{} minerals", minerals as i32)
                } else {
                    format!("Harvest +{} biomass", biomass as i32)
                };
                draw_ui_text(
                    &reward_text,
                    right_x + 16.0,
                    inspector_y + reward_y + feature_shift,
                    11.0,
                    warning,
                );
                if !tight {
                    if let Some(bonus) = tile_bonus {
                        let bonus_text = fit_text_to_width(bonus, right_w - 32.0, 10.0);
                        draw_ui_text(
                            &bonus_text,
                            right_x + 16.0,
                            inspector_y + bonus_y + feature_shift,
                            10.0,
                            success,
                        );
                    }
                }
                let button_y = inspector_y + inspector_h - if tight { 36.0 } else { 38.0 };
                let filter_available = terrain == TerrainType::Forest;
                let button_w = if filter_available {
                    (right_w - 40.0) * 0.5
                } else {
                    right_w - 32.0
                };
                if let Some(tile_pos) = display_pos {
                    if draw_hud_button(
                        theme,
                        Rect::new(right_x + 16.0, button_y, button_w, 28.0),
                        "HARVEST",
                    ) {
                        state.try_harvest_terrain(tile_pos);
                    }
                    if filter_available
                        && draw_hud_button(
                            theme,
                            Rect::new(right_x + 24.0 + button_w, button_y, button_w, 28.0),
                            "MAKE FILTER",
                        )
                    {
                        state.try_convert_forest_to_filter(tile_pos);
                    }
                }
            }
        } else {
            draw_ui_text(
                "Tap a tile or select a build option.",
                right_x + 16.0,
                inspector_y + 82.0,
                11.0,
                dim,
            );
        }
    }
}

fn core_stage_status_text(state: &PlanetState) -> String {
    let stages = state.core_stages();
    if stages.is_empty() {
        return "CORE // NO STAGE DATA".to_string();
    }

    let current_index = state.core_stage_index();
    let stage_count = stages.len();
    if stages.get(current_index + 1).is_none() {
        return format!("CORE {stage_count}/{stage_count} // FULLY EVOLVED");
    }

    let requirements = state
        .core_requirement_progress()
        .iter()
        .map(|progress| {
            format!(
                "{}{}/{}",
                core_requirement_label(progress.milestone),
                compact_requirement_amount(progress.current),
                compact_requirement_amount(progress.target)
            )
        })
        .collect::<Vec<_>>()
        .join("+");
    if requirements.is_empty() {
        format!("CORE {}/{stage_count} // READY", current_index + 1)
    } else {
        format!("CORE {}/{stage_count} / {requirements}", current_index + 1)
    }
}

fn core_requirement_label(milestone: Milestone) -> &'static str {
    match milestone {
        Milestone::Structures => "STR",
        Milestone::NetworkTiles => "NET",
        Milestone::Technologies => "TECH",
        Milestone::SeedShipStages => "SHIP",
        _ => milestone.id(),
    }
}

fn compact_requirement_amount(amount: f32) -> String {
    if amount.fract().abs() > 0.05 {
        format!("{amount:.1}")
    } else {
        format!("{amount:.0}")
    }
}

fn core_next_stage_label(state: &PlanetState) -> String {
    state
        .core_stages()
        .get(state.core_stage_index() + 1)
        .map(|stage| stage.name.clone())
        .unwrap_or_else(|| "FULLY EVOLVED".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct InspectorRowLayout {
    show_icon: bool,
    show_detail_rows: bool,
    row_base: f32,
    row_gap: f32,
}

fn inspector_row_layout(inspector_y: f32, inspector_h: f32) -> InspectorRowLayout {
    if inspector_h < 140.0 {
        InspectorRowLayout {
            show_icon: false,
            show_detail_rows: false,
            row_base: inspector_y + 74.0,
            row_gap: 16.0,
        }
    } else if inspector_h < 175.0 {
        InspectorRowLayout {
            show_icon: false,
            show_detail_rows: true,
            row_base: inspector_y + 88.0,
            row_gap: 18.0,
        }
    } else {
        InspectorRowLayout {
            show_icon: true,
            show_detail_rows: true,
            row_base: inspector_y + inspector_h - 72.0,
            row_gap: 20.0,
        }
    }
}

/// How the ground under a tile reads: a deposit, ordinary, or poor.
fn ore_status(richness: f32, colors: &PanelColors) -> (&'static str, Color) {
    if richness > 1.05 {
        ("DEPOSIT", colors.success)
    } else if richness < 0.95 {
        ("LEAN", colors.warning)
    } else {
        ("ORDINARY", colors.dim)
    }
}

#[cfg(test)]
mod tests;
