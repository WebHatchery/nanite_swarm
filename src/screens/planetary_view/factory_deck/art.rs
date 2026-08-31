use std::collections::HashSet;

use crate::data::UiTheme;
use crate::engine::{Building, GridPos};
use crate::state::{FactoryFocus, PlanetState};
use crate::ui::color_from_rgba;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use super::super::metrics::{grid_to_screen, HudMetrics};

pub(super) fn draw_depth_markers(
    state: &PlanetState,
    metrics: HudMetrics,
    theme: &UiTheme,
    time: f32,
) {
    let Some(core) = state.grid.find_core() else {
        return;
    };
    let (x, y) = grid_to_screen(core, metrics);
    let center = vec2(x + metrics.tile_size * 0.5, y + metrics.tile_size * 0.5);
    let depth = state.factory_depth().min(3);
    if depth == 0 || metrics.tile_size < 26.0 {
        return;
    }
    let color = color_from_rgba(&theme.colors.primary);
    for index in 0..depth {
        let radius = metrics.tile_size * (0.72 + index as f32 * 0.18);
        let phase = time * (0.8 + index as f32 * 0.13) + index as f32 * 1.8;
        let marker = center + vec2(phase.cos(), phase.sin()) * radius;
        draw_circle(
            marker.x,
            marker.y,
            2.0 + index as f32 * 0.5,
            with_alpha(color, 0.62),
        );
        draw_line(
            center.x,
            center.y,
            marker.x,
            marker.y,
            0.6,
            with_alpha(color, 0.13),
        );
    }
}

pub(super) fn draw_focus_targets(
    state: &PlanetState,
    metrics: HudMetrics,
    theme: &UiTheme,
    time: f32,
) {
    if state.factory_focus == FactoryFocus::Balanced {
        return;
    }

    let starved: HashSet<_> = state.starved_factories().into_iter().collect();
    let blocked: HashSet<_> = state.blocked_factories().into_iter().collect();
    let pulse = 0.35 + (time * 2.6).sin().abs() * 0.18;
    for position in state.factory_focus_target_positions() {
        let Some(building) = state
            .grid
            .get(position)
            .and_then(|tile| tile.building.as_ref())
        else {
            continue;
        };
        let color = focus_target_color(state, position, building, &starved, &blocked, theme);
        let (x, y) = grid_to_screen(position, metrics);
        let inset = (metrics.tile_size * 0.1).clamp(2.0, 4.0);
        let size = metrics.tile_size - inset * 2.0;
        draw_rectangle_lines(
            x + inset,
            y + inset,
            size,
            size,
            1.2,
            with_alpha(color, pulse),
        );
        let beacon = 2.0 + (time * 3.0 + position.x as f32 * 0.17).sin().abs();
        draw_circle(x + metrics.tile_size * 0.5, y + inset, beacon, color);
    }
}

pub(super) fn focus_target_color(
    state: &PlanetState,
    position: GridPos,
    building: &Building,
    starved: &HashSet<GridPos>,
    blocked: &HashSet<GridPos>,
    theme: &UiTheme,
) -> Color {
    if !building.powered {
        color_from_rgba(&theme.colors.text_dim)
    } else if building.standby || building.is_dust_stalled_with(&state.resolved_dust_response()) {
        color_from_rgba(&theme.colors.warning)
    } else if blocked.contains(&position) {
        color_from_rgba(&theme.colors.error)
    } else if starved.contains(&position) {
        color_from_rgba(&theme.colors.warning)
    } else {
        color_from_rgba(&theme.colors.primary)
    }
}
