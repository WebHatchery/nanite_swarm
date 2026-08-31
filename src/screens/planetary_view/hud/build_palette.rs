//! Left sidebar: the build palette list and quick actions

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::engine::BuildingType;
use crate::state::PlanetState;
use crate::ui::draw_hud_panel;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::format::fit_text_to_width;
use super::super::metrics::HudMetrics;
use super::PanelColors;

mod quick_actions;
mod rows;

#[derive(Debug, Clone, Copy, PartialEq)]
struct QuickActionLayout {
    compact: bool,
    button_height: f32,
    row_gap: f32,
    policy_gap: f32,
    policy_height: f32,
    total_height: f32,
}

fn quick_action_layout(sidebar_height: f32) -> QuickActionLayout {
    let compact = sidebar_height < 360.0;
    let button_height = if compact { 22.0 } else { 30.0 };
    let row_gap = if compact { 4.0 } else { 8.0 };
    let policy_gap = if compact { 4.0 } else { 6.0 };
    let policy_height = if compact { 20.0 } else { 24.0 };
    let total_height = button_height * 5.0 + row_gap * 4.0 + policy_gap + policy_height + 4.0;
    QuickActionLayout {
        compact,
        button_height,
        row_gap,
        policy_gap,
        policy_height,
        total_height,
    }
}

pub(super) fn draw(
    state: &mut PlanetState,
    theme: &UiTheme,
    textures: &GameTextures,
    metrics: HudMetrics,
    colors: &PanelColors,
    screen_h: f32,
    close_on_select: bool,
) {
    let data_context = state.data.clone();
    let data = data_context.data();
    let dim = colors.dim;

    let sidebar_x = 10.0;
    let sidebar_y = metrics.top_bar_height + metrics.panel_gap;
    let sidebar_w = metrics.left_panel_width - metrics.panel_gap * 1.5;
    let sidebar_h = screen_h - sidebar_y - metrics.bottom_bar_height - metrics.panel_gap;
    draw_hud_panel(
        theme,
        Rect::new(sidebar_x, sidebar_y, sidebar_w, sidebar_h),
        Some("BUILD PALETTE"),
    );
    draw_ui_text(
        "Tap card + grid; drag from a building",
        sidebar_x + metrics.panel_padding,
        sidebar_y + 50.0,
        theme.typography.small,
        dim,
    );
    let (constraints_met, constraint_text) = state.planet_constraint_status();
    if let Some(guidance) = state.arrival_guidance() {
        draw_ui_text(
            &fit_text_to_width(&guidance, sidebar_w - metrics.panel_padding * 2.0, 9.0),
            sidebar_x + metrics.panel_padding,
            sidebar_y + 66.0,
            9.0,
            colors.primary_soft,
        );
    } else if !constraints_met && !constraint_text.is_empty() {
        draw_ui_text(
            &format!("WORLD RULE: {}", constraint_text),
            sidebar_x + metrics.panel_padding,
            sidebar_y + 66.0,
            9.0,
            colors.warning,
        );
    }

    let mut building_defs: Vec<_> = data
        .buildings
        .iter()
        .filter(|def| def.show_in_build_menu)
        .collect();
    building_defs.sort_by_key(|def| def.build_menu_order);

    let list_top = sidebar_y + 78.0;
    let action_layout = quick_action_layout(sidebar_h);
    let quick_actions_h = action_layout.total_height;
    let list_bottom = sidebar_y + sidebar_h - quick_actions_h - metrics.panel_gap;
    let list_height = (list_bottom - list_top).max(0.0);
    let content_x = sidebar_x + metrics.panel_padding;
    let content_w = sidebar_w - metrics.panel_padding * 2.0;
    let row_gap = 8.0;
    let card_w = content_w;

    let mut visible_buildings = Vec::new();
    for def in building_defs {
        let Some(building) = BuildingType::from_id(&def.id) else {
            continue;
        };
        // Banned buildings stay on the list once researched, greyed out: the
        // constraint is the puzzle, so hiding it hides the puzzle.
        if !state.is_building_researched(building) {
            continue;
        }
        visible_buildings.push(building);
    }

    let total_rows = visible_buildings.len();
    let total_height = if total_rows == 0 {
        0.0
    } else {
        total_rows as f32 * metrics.build_row_height
            + (total_rows.saturating_sub(1)) as f32 * row_gap
    };
    let list_view = Rect::new(content_x, list_top, content_w, list_height);
    state.build_palette_scroll.update(list_view, total_height);
    let scroll_offset = state.build_palette_scroll.offset();

    let start_y = list_top - scroll_offset;
    let rows_interactive = !state.build_palette_scroll.absorbs_press();
    for (index, building) in visible_buildings.into_iter().enumerate() {
        let card_y = start_y + index as f32 * (metrics.build_row_height + row_gap);
        if card_y + metrics.build_row_height < list_top || card_y > list_bottom {
            continue;
        }
        rows::draw(
            state,
            theme,
            textures,
            content_x,
            card_y,
            card_w,
            metrics.build_row_height,
            building,
            rows_interactive,
            close_on_select,
        );
    }

    state
        .build_palette_scroll
        .draw_scrollbar(list_view, total_height);

    quick_actions::draw(quick_actions::QuickActionsInput {
        state,
        theme,
        sidebar_x,
        sidebar_w,
        panel_padding: metrics.panel_padding,
        quick_actions_y: list_bottom + 12.0,
        layout: action_layout,
    });
}

#[cfg(test)]
mod tests;
