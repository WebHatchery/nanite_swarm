use crate::data::{GameData, UiTheme};
use crate::engine::{GridPos, TerrainType};
use crate::screens::planetary_view::format::fit_text_to_width;
use crate::state::{PlanetFeature, PlanetState};
use crate::ui::draw_hud_button;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::{PanelColors, RightStackLayout};
use super::terrain_layout::{forest_filter_label, terrain_detail_offsets, terrain_feature_lines};

pub(super) struct TerrainInspectorInput<'a> {
    pub(super) data: &'a GameData,
    pub(super) display_pos: Option<GridPos>,
    pub(super) tile_terrain: Option<TerrainType>,
    pub(super) tile_harvest: Option<(f32, f32)>,
    pub(super) tile_bonus: Option<&'a str>,
    pub(super) tile_feature: Option<&'a PlanetFeature>,
    pub(super) local_feature_label: Option<&'a str>,
    pub(super) tile_filter: bool,
}

pub(super) fn draw(
    state: &mut PlanetState,
    input: TerrainInspectorInput<'_>,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
    tight: bool,
) {
    let TerrainInspectorInput {
        data,
        display_pos,
        tile_terrain,
        tile_harvest,
        tile_bonus,
        tile_feature,
        local_feature_label,
        tile_filter,
    } = input;
    let right_x = right.right_x;
    let right_w = right.right_w;
    let inspector_y = right.inspector.y;
    let inspector_h = right.inspector.h;
    let text = colors.text;
    let dim = colors.dim;
    let success = colors.success;
    let warning = colors.warning;

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
    if let Some(feature) = tile_feature {
        if feature_lines > 0 {
            let feature_name = local_feature_label.unwrap_or("Feature");
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
        let filter_label = forest_filter_label(tile_filter, state.config.upkeep.filter_multiplier);
        draw_ui_text(
            &format!("Terrain: {}", terrain.name(data)),
            right_x + 16.0,
            inspector_y + terrain_y + feature_shift,
            11.0,
            dim,
        );
        if let Some(filter_label) = filter_label {
            draw_ui_text(
                &filter_label,
                right_x + 16.0,
                inspector_y + terrain_y + 16.0 + feature_shift,
                10.0,
                success,
            );
        }
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
