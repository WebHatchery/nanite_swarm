//! HUD orchestration: computes shared layout/colors and delegates to panel drawers

mod bottom_bar;
mod build_palette;
mod directive_panel;
mod inspector_panel;
mod power_ops_panel;
mod top_bar;

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::directives::Directive;
use crate::engine::GridPos;
use crate::state::PlanetState;
use crate::ui::color_from_rgba;
use macroquad::prelude::*;

use super::metrics::HudMetrics;
use super::PlanetaryAction;

pub(super) struct PanelColors {
    pub(super) text: Color,
    pub(super) dim: Color,
    pub(super) primary: Color,
    pub(super) primary_soft: Color,
    pub(super) success: Color,
    pub(super) warning: Color,
    pub(super) error: Color,
    pub(super) power: Color,
}

impl PanelColors {
    fn from_theme(theme: &UiTheme, power_balance: f32) -> Self {
        let success = color_from_rgba(&theme.colors.success);
        let error = color_from_rgba(&theme.colors.error);
        Self {
            text: color_from_rgba(&theme.colors.text),
            dim: color_from_rgba(&theme.colors.text_dim),
            primary: color_from_rgba(&theme.colors.primary),
            primary_soft: color_from_rgba(&theme.colors.primary_soft),
            power: if power_balance >= 0.0 { success } else { error },
            success,
            warning: color_from_rgba(&theme.colors.warning),
            error,
        }
    }
}

/// Shared geometry for the right-hand panel stack (inspector/power/ops/directive),
/// so each panel doesn't have to recompute the vertical layout independently.
pub(super) struct RightStackLayout {
    pub(super) right_x: f32,
    pub(super) right_w: f32,
    pub(super) inspector: Rect,
    pub(super) power: Rect,
    pub(super) ops: Rect,
    pub(super) directive: Rect,
}

impl RightStackLayout {
    fn compute(screen_w: f32, screen_h: f32, metrics: HudMetrics) -> Self {
        let right_x = screen_w - metrics.right_panel_width - 10.0;
        let right_y = metrics.top_bar_height + metrics.panel_gap;
        let right_w = metrics.right_panel_width;
        let right_h = screen_h - right_y - metrics.bottom_bar_height - metrics.panel_gap;

        let stack_gap = metrics.panel_gap;
        let available_stack = (right_h - stack_gap * 3.0).max(0.0);
        // Allocate the stack from the actual viewport first. Fixed minimums
        // made the directive panel run underneath the bottom bar on short
        // browser windows, exactly where touch users need the compact layout.
        let inspector_h = available_stack * 0.36;
        let power_h = available_stack * 0.24;
        let ops_h = available_stack * 0.18;
        let directive_h = available_stack - inspector_h - power_h - ops_h;
        let inspector_y = right_y;
        let power_y = inspector_y + inspector_h + stack_gap;
        let ops_y = power_y + power_h + stack_gap;
        let directive_y = ops_y + ops_h + stack_gap;

        Self {
            right_x,
            right_w,
            inspector: Rect::new(right_x, inspector_y, right_w, inspector_h),
            power: Rect::new(right_x, power_y, right_w, power_h),
            ops: Rect::new(right_x, ops_y, right_w, ops_h),
            directive: Rect::new(right_x, directive_y, right_w, directive_h),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_ui_panels(
    state: &mut PlanetState,
    screen_w: f32,
    screen_h: f32,
    hovered_pos: Option<GridPos>,
    directive: &Directive,
    textures: &GameTextures,
    theme: &UiTheme,
    metrics: HudMetrics,
) -> PlanetaryAction {
    let colors = PanelColors::from_theme(theme, state.power_balance);

    let mut ui_action = top_bar::draw(state, screen_w, metrics, theme, &colors);

    if !state.focus_mode {
        build_palette::draw(state, theme, textures, metrics, &colors, screen_h);

        let right = RightStackLayout::compute(screen_w, screen_h, metrics);
        inspector_panel::draw(state, hovered_pos, textures, theme, &colors, &right);
        let ops_action = power_ops_panel::draw(state, theme, &colors, &right);
        if ui_action == PlanetaryAction::None {
            ui_action = ops_action;
        }
        directive_panel::draw(state, directive, theme, &colors, &right);
    }

    match bottom_bar::draw(state, screen_w, screen_h, theme, metrics, &colors) {
        bottom_bar::ClockAction::TogglePause => state.toggle_pause(),
        bottom_bar::ClockAction::Faster => state.change_speed(true),
        bottom_bar::ClockAction::Slower => state.change_speed(false),
        bottom_bar::ClockAction::NextEvent => {
            state.fast_forward_to_next_event();
        }
        bottom_bar::ClockAction::ToggleFocus => state.focus_mode = !state.focus_mode,
        bottom_bar::ClockAction::ToggleFlow => state.flow_overlay = !state.flow_overlay,
        bottom_bar::ClockAction::ToggleDeck => state.factory_deck_open = !state.factory_deck_open,
        bottom_bar::ClockAction::None => {}
    }

    ui_action
}
