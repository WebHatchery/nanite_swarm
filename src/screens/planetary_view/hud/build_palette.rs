//! Left sidebar: the build palette list and quick actions

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::engine::BuildingType;
use crate::state::PlanetState;
use crate::ui::{draw_hud_button, draw_hud_panel};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::format::fit_text_to_width;
use super::super::metrics::HudMetrics;
use super::PanelColors;

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

    let quick_actions_y = list_bottom + 12.0;
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + metrics.panel_padding,
            quick_actions_y,
            sidebar_w - metrics.panel_padding * 2.0,
            action_layout.button_height,
        ),
        if state.demolish_mode {
            "DEMOLISH MODE  [ON]"
        } else {
            "DEMOLISH MODE"
        },
    ) {
        state.toggle_demolish_mode();
    }
    let half_w = (sidebar_w - metrics.panel_padding * 2.0 - 8.0) * 0.5;
    let second_y = quick_actions_y + action_layout.button_height + action_layout.row_gap;
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + metrics.panel_padding,
            second_y,
            half_w,
            action_layout.button_height,
        ),
        "CANCEL",
    ) {
        state.cancel_interaction_modes();
    }
    let stamp_label = if state.blueprint.is_empty() {
        "STAMP EMPTY"
    } else {
        "STAMP"
    };
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + metrics.panel_padding + half_w + 8.0,
            second_y,
            half_w,
            action_layout.button_height,
        ),
        "HELP",
    ) {
        state.show_help = !state.show_help;
    }
    let third_y = second_y + action_layout.button_height + action_layout.row_gap;
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + metrics.panel_padding,
            third_y,
            half_w,
            action_layout.button_height,
        ),
        if state.undo_history.is_empty() {
            "UNDO EMPTY"
        } else {
            "UNDO"
        },
    ) {
        state.undo_last_action();
    }
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + metrics.panel_padding + half_w + 8.0,
            third_y,
            half_w,
            action_layout.button_height,
        ),
        "RELOCATE",
    ) {
        if let Some(tile) = state.selected_tile {
            state.begin_relocation(tile);
        } else {
            state
                .notifications
                .warning("Select a structure before relocating");
        }
    }
    let fourth_y = third_y + action_layout.button_height + action_layout.row_gap;
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + metrics.panel_padding,
            fourth_y,
            half_w,
            action_layout.button_height,
        ),
        "SAVE BP",
    ) {
        if let Some(tile) = state.selected_tile {
            let positions = if state.box_selected.is_empty() {
                vec![tile]
            } else {
                state.box_selected.clone()
            };
            state.save_blueprint(tile, &positions);
        } else {
            state
                .notifications
                .warning("Select an anchor tile before saving a blueprint");
        }
    }
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + metrics.panel_padding + half_w + 8.0,
            fourth_y,
            half_w,
            action_layout.button_height,
        ),
        stamp_label,
    ) {
        if let Some(tile) = state.selected_tile {
            state.place_blueprint(tile);
        } else {
            state
                .notifications
                .warning("Select an anchor tile before stamping");
        }
    }
    let fifth_y = fourth_y + action_layout.button_height + action_layout.row_gap;
    let blocked: std::collections::HashSet<_> = state.blocked_factories().into_iter().collect();
    let selected_blocked = state
        .box_selected
        .iter()
        .filter(|pos| blocked.contains(pos))
        .count();
    let selection_action = if selected_blocked > 0 {
        if state.bulk_purge_armed {
            "PURGE AGAIN".to_string()
        } else {
            format!("PURGE {}", selected_blocked)
        }
    } else if state.box_selected.is_empty() {
        "BOX SELECT".to_string()
    } else {
        "RESELECT".to_string()
    };
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + metrics.panel_padding,
            fifth_y,
            half_w,
            action_layout.button_height,
        ),
        &selection_action,
    ) {
        if selected_blocked > 0 {
            state.request_selected_pad_purge();
        } else {
            state.begin_box_select();
        }
    }
    let selected_processors: Vec<bool> = state
        .box_selected
        .iter()
        .filter_map(|pos| {
            state
                .grid
                .get(*pos)
                .and_then(|tile| tile.building.as_ref())
                .filter(|building| building.supports_overclock() && !building.standby)
                .map(|building| building.overclocked)
        })
        .collect();
    if !selected_processors.is_empty() {
        let enable = selected_processors.iter().any(|boosted| !boosted);
        let unlocked = state
            .research
            .unlocked_techs
            .iter()
            .any(|tech| tech == "adaptive_clocking");
        let label = if !unlocked {
            "BOOST LOCKED".to_string()
        } else if enable {
            format!(
                "BOOST {}",
                selected_processors
                    .iter()
                    .filter(|boosted| !**boosted)
                    .count()
            )
        } else {
            format!("NORMAL {}", selected_processors.len())
        };
        if draw_hud_button(
            theme,
            Rect::new(
                sidebar_x + metrics.panel_padding + half_w + 8.0,
                fifth_y,
                half_w,
                action_layout.button_height,
            ),
            &label,
        ) {
            state.set_selected_overclock(enable);
        }

        let selected_priority: Vec<bool> = state
            .box_selected
            .iter()
            .filter_map(|pos| {
                state
                    .grid
                    .get(*pos)
                    .and_then(|tile| tile.building.as_ref())
                    .filter(|building| building.supports_overclock())
                    .map(|building| building.input_priority)
            })
            .collect();
        let prioritize = selected_priority.iter().any(|priority| !priority);
        let label = if prioritize {
            format!(
                "PRIORITIZE {}",
                selected_priority
                    .iter()
                    .filter(|priority| !**priority)
                    .count()
            )
        } else {
            format!("STANDARDIZE {}", selected_priority.len())
        };
        if draw_hud_button(
            theme,
            Rect::new(
                sidebar_x + metrics.panel_padding,
                fifth_y + action_layout.button_height + action_layout.policy_gap,
                half_w,
                action_layout.policy_height,
            ),
            &label,
        ) {
            state.set_selected_input_priority(prioritize);
        }
        let selected_standby: Vec<bool> = state
            .box_selected
            .iter()
            .filter_map(|pos| {
                state
                    .grid
                    .get(*pos)
                    .and_then(|tile| tile.building.as_ref())
                    .filter(|building| building.supports_overclock())
                    .map(|building| building.standby)
            })
            .collect();
        let pause = selected_standby.iter().any(|standby| !standby);
        let label = if pause {
            format!(
                "PAUSE {}",
                selected_standby.iter().filter(|standby| !**standby).count()
            )
        } else {
            format!("RESUME {}", selected_standby.len())
        };
        if draw_hud_button(
            theme,
            Rect::new(
                sidebar_x + metrics.panel_padding + half_w + 8.0,
                fifth_y + action_layout.button_height + action_layout.policy_gap,
                half_w,
                action_layout.policy_height,
            ),
            &label,
        ) {
            state.set_selected_standby(pause);
        }
    } else {
        let unlocked = state
            .research
            .unlocked_techs
            .iter()
            .any(|tech| tech == "adaptive_clocking");
        let label = if !unlocked {
            "AUTO LOCKED"
        } else if state.auto_clocking {
            "AUTO ON"
        } else {
            "AUTO CLOCK"
        };
        if draw_hud_button(
            theme,
            Rect::new(
                sidebar_x + metrics.panel_padding + half_w + 8.0,
                fifth_y,
                half_w,
                action_layout.button_height,
            ),
            label,
        ) {
            state.toggle_auto_clocking();
        }
    }
}

#[cfg(test)]
mod tests;
