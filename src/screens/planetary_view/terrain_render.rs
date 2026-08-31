//! Background, terrain, conduit, and building rendering for the grid

use crate::assets::GameTextures;
use crate::data::PlanetFeatureDef;
use crate::engine::{BuildingType, GridPos, TerrainType};
use crate::state::{PlanetFeature, PlanetState};
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::math::pulse01_at;
use macroquad_toolkit::ui::draw_ui_text;

use super::metrics::{grid_to_screen, HudMetrics};

mod background;
mod core;
use core::draw_core_visual;

pub(super) fn draw_core_stage_ambient_for_map(state: &PlanetState, metrics: HudMetrics, time: f32) {
    background::draw_core_stage_ambient_for_map(state, metrics, time);
}

pub(super) fn draw_core_stage_planetary_backdrop(
    state: &PlanetState,
    screen_w: f32,
    screen_h: f32,
    time: f32,
) {
    background::draw_core_stage_planetary_backdrop(state, screen_w, screen_h, time);
}

pub(super) fn draw_planetary_background(
    screen_w: f32,
    screen_h: f32,
    time: f32,
    state: &PlanetState,
) {
    background::draw_planetary_background(screen_w, screen_h, time, state);
}

pub(super) fn draw_collapse_shake(state: &PlanetState, screen_w: f32, screen_h: f32, time: f32) {
    background::draw_collapse_shake(state, screen_w, screen_h, time);
}

pub(super) fn draw_planet_features(state: &PlanetState, metrics: HudMetrics) {
    for feature in &state.features {
        if !feature_is_surveyed(state, feature) {
            continue;
        }
        let definition = state
            .data()
            .planet(state.planet_index)
            .features
            .iter()
            .find(|definition| definition.id == feature.id);
        let target_kind = definition.and_then(feature_target_kind);
        let (x, y, w, h) = feature.bounds;
        let origin = GridPos::new(
            (x * state.grid.width as f32) as i32,
            (y * state.grid.height as f32) as i32,
        );
        let end = GridPos::new(
            ((x + w) * state.grid.width as f32) as i32,
            ((y + h) * state.grid.height as f32) as i32,
        );
        let (sx, sy) = grid_to_screen(origin, metrics);
        let (ex, ey) = grid_to_screen(end, metrics);
        draw_rectangle_lines(
            sx,
            sy,
            (ex - sx).max(metrics.tile_size),
            (ey - sy).max(metrics.tile_size),
            1.0,
            with_alpha(Colors::ACCENT, 0.55),
        );
        let label = target_kind
            .map(|kind| format!("{} // {}", feature.name, kind.label()))
            .unwrap_or_else(|| feature.name.clone());
        draw_ui_text(&label, sx + 4.0, sy + 14.0, 9.0, Colors::ACCENT);
        if let Some(kind) = target_kind {
            draw_feature_target_markers(state, feature, kind, metrics);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FeatureTargetKind {
    Drill,
    Harvest,
    Cooling,
}

impl FeatureTargetKind {
    fn label(self) -> &'static str {
        match self {
            Self::Drill => "DRILL ZONE",
            Self::Harvest => "HARVEST ZONE",
            Self::Cooling => "COOLING EDGE",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Drill => Colors::ACCENT,
            Self::Harvest => Colors::SUCCESS,
            Self::Cooling => Colors::PRIMARY,
        }
    }
}

fn feature_target_kind(definition: &PlanetFeatureDef) -> Option<FeatureTargetKind> {
    if definition
        .ore_multiplier
        .is_some_and(|multiplier| multiplier > 1.0)
    {
        Some(FeatureTargetKind::Drill)
    } else if definition
        .harvest_multiplier
        .is_some_and(|multiplier| multiplier > 1.0)
    {
        Some(FeatureTargetKind::Harvest)
    } else if definition
        .cooling_multiplier
        .is_some_and(|multiplier| multiplier > 1.0)
    {
        Some(FeatureTargetKind::Cooling)
    } else {
        None
    }
}

fn draw_feature_target_markers(
    state: &PlanetState,
    feature: &PlanetFeature,
    kind: FeatureTargetKind,
    metrics: HudMetrics,
) {
    let color = kind.color();
    for (pos, tile) in state.grid.iter_tiles() {
        if !feature.contains_tile(pos, state.grid.width, state.grid.height)
            || !feature_target_is_available(state, pos, tile, kind)
        {
            continue;
        }
        let (x, y) = grid_to_screen(pos, metrics);
        let inset = (metrics.tile_size * 0.14).max(2.0);
        let size = (metrics.tile_size - inset * 2.0).max(1.0);
        draw_rectangle_lines(
            x + inset,
            y + inset,
            size,
            size,
            1.5,
            with_alpha(color, 0.85),
        );
        draw_circle(
            x + metrics.tile_size * 0.5,
            y + metrics.tile_size * 0.5,
            (metrics.tile_size * 0.08).max(1.0),
            with_alpha(color, 0.9),
        );
    }
}

fn feature_target_is_available(
    state: &PlanetState,
    pos: GridPos,
    tile: &crate::engine::Tile,
    kind: FeatureTargetKind,
) -> bool {
    if !tile.revealed || tile.building.is_some() {
        return false;
    }
    match kind {
        FeatureTargetKind::Drill => {
            tile.terrain.is_buildable(state.data())
                && state.feature_ore_multiplier_at(pos).is_some()
        }
        FeatureTargetKind::Harvest => state.harvest_rewards_at(pos).is_some(),
        FeatureTargetKind::Cooling => {
            tile.terrain.is_buildable(state.data())
                && state.feature_cooling_multiplier_at(pos).is_some()
        }
    }
}

fn feature_is_surveyed(state: &PlanetState, feature: &PlanetFeature) -> bool {
    let mut has_tiles = false;
    for (pos, tile) in state.grid.iter_tiles() {
        if !feature.contains_tile(pos, state.grid.width, state.grid.height) {
            continue;
        }
        has_tiles = true;
        if !tile.revealed {
            return false;
        }
    }
    has_tiles
}

pub(super) fn draw_tutorial_route_hint(state: &PlanetState, metrics: HudMetrics, time: f32) {
    let Some(step) = state.tutorial_current() else {
        return;
    };
    if step.goal.kind != "connect" {
        return;
    }
    let (Some(core), Some(drill)) = (
        state.grid.find_core(),
        state
            .grid
            .find_buildings(BuildingType::Drill)
            .first()
            .copied(),
    ) else {
        return;
    };
    let center = |pos: GridPos| {
        let (x, y) = grid_to_screen(pos, metrics);
        vec2(x + metrics.tile_size * 0.5, y + metrics.tile_size * 0.5)
    };
    let from = center(core);
    let to = center(drill);
    let distance = from.distance(to).max(1.0);
    let direction = (to - from) / distance;
    let dash = (metrics.tile_size * 0.65).max(8.0);
    let mut travelled = 0.0;
    while travelled < distance {
        let start = from + direction * travelled;
        let end = from + direction * (travelled + dash * 0.55).min(distance);
        draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            2.0,
            with_alpha(Colors::WARNING, 0.8),
        );
        travelled += dash;
    }
    let pulse = if macroquad_toolkit::settings::reduced_motion_enabled() {
        0.0
    } else {
        pulse01_at(time as f64, 1.5) * 3.0
    };
    draw_circle_lines(
        from.x,
        from.y,
        metrics.tile_size * 0.35 + pulse,
        2.0,
        Colors::WARNING,
    );
    draw_circle_lines(
        to.x,
        to.y,
        metrics.tile_size * 0.35 + pulse,
        2.0,
        Colors::WARNING,
    );
    draw_ui_text(
        "ROUTE: CORE -> DRILL",
        from.x + 8.0,
        from.y - 8.0,
        10.0,
        Colors::WARNING,
    );
}

/// Get color for terrain type with subtle variation
mod tiles;
pub(super) use tiles::draw_grid_tiles;

#[cfg(test)]
mod tests;
