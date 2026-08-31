//! Right sidebar: "SELECTED STRUCTURE" inspector panel

use crate::assets::GameTextures;
use crate::data::{GameData, UiTheme};
use crate::engine::{BuildingType, GridPos, TerrainType};
use crate::state::PlanetState;
use crate::ui::draw_hud_panel;
use macroquad::prelude::*;

use super::{PanelColors, RightStackLayout};

mod building;
mod core_status;
mod recipe_flow;
mod terrain;
mod terrain_layout;
use building::draw as draw_building;
use terrain_layout::{feature_label_for, feature_survey_label};

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
    let mut tile_filter = false;
    if let Some(tile_pos) = display_pos {
        if let Some(tile) = state.grid.get(tile_pos) {
            tile_terrain = Some(tile.terrain);
            tile_harvest = state.harvest_rewards_at(tile_pos);
            tile_bonus = tile.terrain.preservation_bonus(data);
            tile_biomass = tile.biomass_amount;
            tile_filter = tile.filter;
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
        draw_building(BuildingInspectorInput {
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
            local_feature_label: local_feature_label.as_deref(),
            textures,
            theme,
            colors,
            right,
            tight,
        });
    } else {
        terrain::draw(
            state,
            terrain::TerrainInspectorInput {
                data,
                display_pos,
                tile_terrain,
                tile_harvest,
                tile_bonus,
                tile_feature: tile_feature.as_ref(),
                local_feature_label: local_feature_label.as_deref(),
                tile_filter,
            },
            theme,
            colors,
            right,
            tight,
        );
    }
}

/// All snapshot values needed by the building-specific inspector renderer.
struct BuildingInspectorInput<'a> {
    state: &'a mut PlanetState,
    data: &'a GameData,
    display_pos: Option<GridPos>,
    building_type: BuildingType,
    tile_building: Option<BuildingType>,
    tile_pos_with_building: Option<GridPos>,
    tile_terrain: Option<TerrainType>,
    tile_powered: bool,
    tile_dust: f32,
    tile_acid: f32,
    tile_heat: f32,
    tile_overclocked: bool,
    tile_standby: bool,
    tile_can_overclock: bool,
    tile_output_full: bool,
    boost_unlocked: bool,
    tile_biomass: f32,
    tile_bonus: Option<&'a str>,
    local_feature_label: Option<&'a str>,
    textures: &'a GameTextures,
    theme: &'a UiTheme,
    colors: &'a PanelColors,
    right: &'a RightStackLayout,
    tight: bool,
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
