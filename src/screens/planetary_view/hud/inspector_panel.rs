//! Right sidebar: "SELECTED STRUCTURE" inspector panel

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::engine::{BuildingType, GridPos, ResourceType, TerrainType};
use crate::state::PlanetState;
use crate::ui::{
    color_from_rgba, draw_hud_button, draw_hud_panel, draw_resource_icon, draw_status_row,
    resource_color,
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::format::{dust_status, fit_text_to_width, format_power_delta};
use super::{PanelColors, RightStackLayout};

pub(super) fn draw(
    state: &mut PlanetState,
    hovered_pos: Option<GridPos>,
    textures: &GameTextures,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
) {
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
    let mut tile_can_overclock = false;
    let boost_unlocked = state
        .research
        .unlocked_techs
        .iter()
        .any(|tech| tech == "adaptive_clocking");
    let mut tile_harvest = None;
    let mut tile_bonus = None;
    if let Some(tile_pos) = display_pos {
        if let Some(tile) = state.grid.get(tile_pos) {
            tile_terrain = Some(tile.terrain);
            tile_harvest = Some(tile.terrain.harvest_rewards());
            tile_bonus = tile.terrain.preservation_bonus();
            if let Some(building) = &tile.building {
                tile_building = Some(building.building_type);
                tile_pos_with_building = Some(tile_pos);
                tile_powered = building.powered;
                tile_dust = building.dust;
                tile_acid = building.acid_wear;
                tile_heat = building.heat;
                tile_overclocked = building.overclocked;
                tile_can_overclock = building.supports_overclock();
            }
        }
    }

    let inspected_building = tile_building.or(state.selected_building);
    if let Some(building_type) = inspected_building {
        let header_y = inspector_y + 44.0;
        let row_layout = inspector_row_layout(inspector_y, inspector_h);
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
        draw_ui_text(building_type.name(), info_x, header_y + 16.0, 13.0, text);
        let description = fit_text_to_width(
            building_type.description(),
            right_x + right_w - info_x - 14.0,
            if row_layout.show_icon { 10.0 } else { 9.0 },
        );
        draw_ui_text(
            &description,
            info_x,
            header_y + if row_layout.show_icon { 40.0 } else { 32.0 },
            if row_layout.show_icon { 10.0 } else { 9.0 },
            dim,
        );
        let power_multiplier = if tile_overclocked { 1.75 } else { 1.0 };
        let output = format_power_delta(building_type.power_delta() * power_multiplier);
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
        let has_recipe = !crate::data::game_data()
            .building(building_type.id())
            .recipe
            .is_empty();
        if !has_recipe {
            if let Some(tile) = display_pos.and_then(|pos| state.grid.get(pos)) {
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
            if waiting_out >= 1.0
                && crate::data::game_data()
                    .building(building_type.id())
                    .recipe
                    .is_empty()
            {
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
        if let Some(pos) = tile_pos_with_building {
            if let Some(flow) = recipe_flow_data(state, pos, building_type) {
                draw_recipe_flow_row(
                    &flow,
                    theme,
                    right_x + 16.0,
                    row_base + row_gap * 3.0,
                    right_w - 32.0,
                    dim,
                    warning,
                );
            }
        }
        if let Some(tile_pos) = tile_pos_with_building {
            if tile_can_overclock
                && draw_hud_button(
                    theme,
                    Rect::new(right_x + right_w - 180.0, inspector_y + 8.0, 92.0, 24.0),
                    if !boost_unlocked {
                        "BOOST LOCKED"
                    } else if tile_overclocked {
                        "BOOST ON"
                    } else {
                        "BOOST"
                    },
                )
            {
                state.toggle_building_overclock(tile_pos);
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
        draw_ui_text(
            "NO STRUCTURE",
            right_x + 16.0,
            inspector_y + 56.0,
            13.0,
            text,
        );
        if let Some(terrain) = tile_terrain {
            draw_ui_text(
                &format!("Terrain: {}", terrain.name()),
                right_x + 16.0,
                inspector_y + 82.0,
                11.0,
                dim,
            );
            if terrain.is_harvestable() {
                let (minerals, biomass) = tile_harvest.unwrap_or((0.0, 0.0));
                let reward_text = if minerals > 0.0 {
                    format!("Harvest +{} minerals", minerals as i32)
                } else {
                    format!("Harvest +{} biomass", biomass as i32)
                };
                draw_ui_text(
                    &reward_text,
                    right_x + 16.0,
                    inspector_y + 106.0,
                    11.0,
                    warning,
                );
                if let Some(bonus) = tile_bonus {
                    let bonus_text = fit_text_to_width(bonus, right_w - 32.0, 10.0);
                    draw_ui_text(
                        &bonus_text,
                        right_x + 16.0,
                        inspector_y + 126.0,
                        10.0,
                        success,
                    );
                }
                let button_y = inspector_y + inspector_h - 38.0;
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

#[derive(Debug, Clone, PartialEq)]
struct RecipeFlowData {
    inputs: Vec<(ResourceType, f32)>,
    output: (ResourceType, f32),
}

fn recipe_flow_data(
    state: &PlanetState,
    pos: GridPos,
    building_type: BuildingType,
) -> Option<RecipeFlowData> {
    let recipe = &crate::data::game_data().building(building_type.id()).recipe;
    if recipe.is_empty() {
        return None;
    }

    let hoppers = state.input_hoppers.get(&(pos.x, pos.y));
    let inputs = recipe
        .carried_ids()
        .into_iter()
        .filter_map(|id| {
            let resource = crate::engine::ResourceType::from_id(id)?;
            let amount = hoppers
                .and_then(|values| values.get(&resource))
                .copied()
                .unwrap_or(0.0);
            Some((resource, amount))
        })
        .collect::<Vec<_>>();
    let output = ResourceType::ALL.into_iter().find(|resource| {
        recipe
            .outputs
            .get(resource.id())
            .is_some_and(|rate| *rate > 0.0)
    })?;
    let waiting = state
        .output_buffers
        .get(&(pos.x, pos.y))
        .copied()
        .unwrap_or(0.0);
    Some(RecipeFlowData {
        inputs,
        output: (output, waiting),
    })
}

fn recipe_status(flow: &RecipeFlowData, powered: bool) -> String {
    if !powered {
        return "No power".to_string();
    }
    let missing: Vec<&str> = flow
        .inputs
        .iter()
        .filter(|(_, amount)| *amount <= 0.001)
        .map(|(resource, _)| flow_resource_name(*resource))
        .collect();
    if !missing.is_empty() {
        return format!("Needs {}", missing.join(" + "));
    }
    if flow.output.1 >= 1.0 {
        "Output waiting".to_string()
    } else {
        "Running".to_string()
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_recipe_flow_row(
    flow: &RecipeFlowData,
    theme: &UiTheme,
    x: f32,
    y: f32,
    width: f32,
    label_color: Color,
    warning: Color,
) {
    draw_ui_text("Flow", x, y, theme.typography.body, label_color);
    let token_w = 46.0;
    let mut cursor = (x + width * 0.31).max(x + 48.0);
    for (index, (resource, amount)) in flow.inputs.iter().enumerate() {
        if index > 0 {
            draw_ui_text("+", cursor - 7.0, y, 9.0, label_color);
        }
        draw_resource_icon(
            *resource,
            Rect::new(cursor, y - 9.0, 10.0, 10.0),
            resource_color(theme, *resource),
        );
        draw_ui_text(
            &format!(
                "{} {}",
                flow_resource_abbrev(*resource),
                compact_amount(*amount)
            ),
            cursor + 13.0,
            y,
            9.0,
            if *amount <= 0.001 {
                warning
            } else {
                resource_color(theme, *resource)
            },
        );
        cursor += token_w;
    }
    draw_ui_text(">", cursor - 7.0, y, 9.0, label_color);
    let (resource, amount) = flow.output;
    draw_resource_icon(
        resource,
        Rect::new(cursor + 2.0, y - 9.0, 10.0, 10.0),
        resource_color(theme, resource),
    );
    draw_ui_text(
        &format!(
            "{} {}",
            flow_resource_abbrev(resource),
            compact_amount(amount)
        ),
        cursor + 15.0,
        y,
        9.0,
        resource_color(theme, resource),
    );
}

fn compact_amount(amount: f32) -> String {
    if amount < 10.0 && amount.fract().abs() > 0.05 {
        format!("{:.1}", amount)
    } else {
        format!("{:.0}", amount)
    }
}

fn flow_resource_abbrev(resource: ResourceType) -> &'static str {
    match resource {
        ResourceType::Minerals => "O",
        ResourceType::Energy => "P",
        ResourceType::Data => "D",
        ResourceType::Biomass => "B",
        ResourceType::Alloy => "A",
        ResourceType::Components => "C",
    }
}

fn flow_resource_name(resource: crate::engine::ResourceType) -> &'static str {
    match resource {
        crate::engine::ResourceType::Minerals => "Ore",
        crate::engine::ResourceType::Components => "Parts",
        _ => crate::state::cargo_name(resource),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct InspectorRowLayout {
    show_icon: bool,
    row_base: f32,
    row_gap: f32,
}

fn inspector_row_layout(inspector_y: f32, inspector_h: f32) -> InspectorRowLayout {
    if inspector_h < 175.0 {
        InspectorRowLayout {
            show_icon: false,
            row_base: inspector_y + 88.0,
            row_gap: 18.0,
        }
    } else {
        InspectorRowLayout {
            show_icon: true,
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
