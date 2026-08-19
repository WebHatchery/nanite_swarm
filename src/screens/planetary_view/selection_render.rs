//! Visible area-selection feedback, drawn after the terrain so it stays visible.

use crate::engine::GridPos;
use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::draw_ui_text;

use super::metrics::{grid_to_screen, HudMetrics};

pub(super) fn draw_box_selection(
    state: &PlanetState,
    metrics: HudMetrics,
    hovered_pos: Option<GridPos>,
) {
    if state.box_select_mode {
        draw_selection_preview(state, metrics, hovered_pos);
    }
    if state.box_selected.is_empty() {
        return;
    }

    for selected in &state.box_selected {
        let (x, y) = grid_to_screen(*selected, metrics);
        draw_rectangle(
            x + 1.0,
            y + 1.0,
            metrics.tile_size - 2.0,
            metrics.tile_size - 2.0,
            with_alpha(Colors::ACCENT, 0.13),
        );
        draw_rectangle_lines(
            x + 1.0,
            y + 1.0,
            metrics.tile_size - 2.0,
            metrics.tile_size - 2.0,
            2.0,
            Colors::ACCENT,
        );
    }

    let processors = state
        .box_selected
        .iter()
        .filter(|pos| {
            state
                .grid
                .get(**pos)
                .and_then(|tile| tile.building.as_ref())
                .is_some_and(|building| building.supports_overclock())
        })
        .count();
    let anchor = state
        .box_selected
        .iter()
        .min_by_key(|pos| (pos.y, pos.x))
        .copied()
        .unwrap();
    let (x, y) = grid_to_screen(anchor, metrics);
    let label = format!(
        "{} SELECTED // {} PROCESSORS",
        state.box_selected.len(),
        processors
    );
    let width = 22.0 + label.len() as f32 * 5.6;
    draw_rectangle(
        x,
        y - 25.0,
        width,
        20.0,
        Color::new(0.005, 0.025, 0.035, 0.94),
    );
    draw_rectangle_lines(x, y - 25.0, width, 20.0, 1.0, Colors::ACCENT);
    draw_ui_text(&label, x + 8.0, y - 12.0, 9.0, Colors::TEXT);
}

fn draw_selection_preview(state: &PlanetState, metrics: HudMetrics, hovered_pos: Option<GridPos>) {
    let (Some(start), Some(end)) = (state.box_select_start, hovered_pos) else {
        return;
    };
    let min = GridPos::new(start.x.min(end.x), start.y.min(end.y));
    let max = GridPos::new(start.x.max(end.x), start.y.max(end.y));
    let (x, y) = grid_to_screen(min, metrics);
    let (end_x, end_y) = grid_to_screen(GridPos::new(max.x + 1, max.y + 1), metrics);
    draw_rectangle(
        x,
        y,
        end_x - x,
        end_y - y,
        with_alpha(Colors::PRIMARY, 0.12),
    );
    draw_rectangle_lines(x, y, end_x - x, end_y - y, 2.0, Colors::PRIMARY);
}
