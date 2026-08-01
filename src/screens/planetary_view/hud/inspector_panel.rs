//! Right sidebar: "SELECTED STRUCTURE" inspector panel

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::engine::{BuildingType, GridPos};
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel, draw_status_row};
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
            }
        }
    }

    let inspected_building = tile_building.or(state.selected_building);
    if let Some(building_type) = inspected_building {
        let header_y = inspector_y + 44.0;
        let compact_inspector = inspector_h < 204.0;
        let icon_size = if compact_inspector { 54.0 } else { 72.0 };
        let icon_rect = Rect::new(right_x + 16.0, header_y, icon_size, icon_size);
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
        let info_x = icon_rect.x + icon_rect.w + 14.0;
        draw_ui_text(building_type.name(), info_x, header_y + 16.0, 13.0, text);
        draw_ui_text(
            "Tier 1",
            right_x + right_w - 56.0,
            header_y + 16.0,
            10.0,
            dim,
        );
        let description = fit_text_to_width(
            building_type.description(),
            right_x + right_w - info_x - 14.0,
            10.0,
        );
        draw_ui_text(&description, info_x, header_y + 40.0, 10.0, dim);
        let output = format_power_delta(building_type.power_delta());
        // Four rows now. They start higher than they did, but not so high as
        // to run into the description above them.
        let row_base = inspector_y + inspector_h - 72.0;
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
            row_base + 20.0,
            right_w - 32.0,
            "Dust",
            &format!("{:.0}% {}", tile_dust, dust_label),
            dust_color,
        );

        // What the ground itself is worth. Without this a deposit is invisible
        // and drill placement is guesswork.
        if let Some(tile) = display_pos.and_then(|pos| state.grid.get(pos)) {
            let (ore_label, ore_color) = ore_status(tile.ore_richness, colors);
            draw_status_row(
                theme,
                right_x + 16.0,
                row_base + 60.0,
                right_w - 32.0,
                "Ore",
                &format!("{:.0}% {}", tile.ore_richness * 100.0, ore_label),
                ore_color,
            );
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
            if waiting_in >= 1.0 {
                status_text = format!("{} - {:.0} in", status_text, waiting_in);
            }
            if waiting_out >= 1.0 {
                status_text = format!("{} - {:.0} out", status_text, waiting_out);
            }
        }
        // A driver's route matters more than the word "Powered" does, and the
        // Power row above is already saying that.
        if building_type == BuildingType::MassDriver {
            if let Some(tile_pos) = tile_pos_with_building {
                status_text = state.export_summary(tile_pos);
            }
        }
        let status_color = if tile_building.is_some() && !tile_powered {
            error
        } else {
            success
        };
        draw_status_row(
            theme,
            right_x + 16.0,
            row_base + 40.0,
            right_w - 32.0,
            "Status",
            &status_text,
            status_color,
        );
        if let Some(tile_pos) = tile_pos_with_building {
            if building_type != BuildingType::Core
                && draw_hud_button(
                    theme,
                    Rect::new(right_x + right_w - 82.0, inspector_y + 40.0, 62.0, 24.0),
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
            }
        } else {
            draw_ui_text(
                "Hover a tile or select a build option.",
                right_x + 16.0,
                inspector_y + 82.0,
                11.0,
                dim,
            );
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
