//! Quick actions beneath the scrollable build-card list.

use crate::data::UiTheme;
use crate::state::PlanetState;
use crate::ui::draw_hud_button;
use macroquad::prelude::Rect;

use super::QuickActionLayout;

/// Layout and state needed by the palette's action rows.
pub(super) struct QuickActionsInput<'a> {
    pub(super) state: &'a mut PlanetState,
    pub(super) theme: &'a UiTheme,
    pub(super) sidebar_x: f32,
    pub(super) sidebar_w: f32,
    pub(super) panel_padding: f32,
    pub(super) quick_actions_y: f32,
    pub(super) layout: QuickActionLayout,
}

pub(super) fn draw(input: QuickActionsInput<'_>) {
    let QuickActionsInput {
        state,
        theme,
        sidebar_x,
        sidebar_w,
        panel_padding,
        quick_actions_y,
        layout: action_layout,
    } = input;
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + panel_padding,
            quick_actions_y,
            sidebar_w - panel_padding * 2.0,
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
    let half_w = (sidebar_w - panel_padding * 2.0 - 8.0) * 0.5;
    let second_y = quick_actions_y + action_layout.button_height + action_layout.row_gap;
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + panel_padding,
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
            sidebar_x + panel_padding + half_w + 8.0,
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
            sidebar_x + panel_padding,
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
            sidebar_x + panel_padding + half_w + 8.0,
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
            sidebar_x + panel_padding,
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
            sidebar_x + panel_padding + half_w + 8.0,
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
            sidebar_x + panel_padding,
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
                sidebar_x + panel_padding + half_w + 8.0,
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
                sidebar_x + panel_padding,
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
                sidebar_x + panel_padding + half_w + 8.0,
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
                sidebar_x + panel_padding + half_w + 8.0,
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
