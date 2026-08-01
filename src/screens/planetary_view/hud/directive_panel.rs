//! Right sidebar: the tutorial while it is running, the directive after.
//!
//! One panel, two jobs, and only ever one at a time. The panel used to be
//! titled TUTORIAL and show directive rows, with the tutorial reduced to a step
//! counter squeezed underneath them.

use crate::data::UiTheme;
use crate::directives::Directive;
use crate::state::PlanetState;
use crate::ui::{draw_hud_panel, draw_status_row};
use macroquad_toolkit::ui::draw_ui_text;

use super::super::format::fit_text_to_width;
use super::{PanelColors, RightStackLayout};

pub(super) fn draw(
    state: &PlanetState,
    directive: &Directive,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
) {
    let teaching = state
        .tutorial_current()
        .filter(|_| !state.tutorial_hidden)
        .is_some();

    if teaching {
        draw_tutorial(state, theme, colors, right);
    } else {
        draw_directive(directive, theme, colors, right);
    }
}

fn draw_tutorial(
    state: &PlanetState,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
) {
    let Some(step) = state.tutorial_current() else {
        return;
    };
    let right_x = right.right_x;
    let right_w = right.right_w;
    let body_y = right.directive.y
        + if right.directive.h < 112.0 {
            48.0
        } else {
            56.0
        };

    draw_hud_panel(theme, right.directive, Some("TUTORIAL"));
    draw_ui_text(
        &format!(
            "{}/{}  {}",
            state.tutorial_step.saturating_add(1),
            state.tutorial_step_count(),
            step.title
        ),
        right_x + 12.0,
        body_y,
        11.0,
        colors.primary_soft,
    );
    let instruction = fit_text_to_width(&step.instruction, right_w - 28.0, 11.0);
    draw_ui_text(
        &instruction,
        right_x + 12.0,
        body_y + 22.0,
        11.0,
        colors.text,
    );
    draw_ui_text(
        "[T] hide",
        right_x + 12.0,
        right.directive.y + right.directive.h - 12.0,
        9.0,
        colors.dim,
    );
}

fn draw_directive(
    directive: &Directive,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
) {
    let right_x = right.right_x;
    let right_w = right.right_w;
    let body_y = right.directive.y
        + if right.directive.h < 112.0 {
            48.0
        } else {
            56.0
        };

    draw_hud_panel(theme, right.directive, Some("DIRECTIVE"));
    let description = fit_text_to_width(&directive.description, right_w - 28.0, 11.0);
    draw_ui_text(&description, right_x + 12.0, body_y, 11.0, colors.text);
    draw_status_row(
        theme,
        right_x + 12.0,
        body_y + 24.0,
        right_w - 24.0,
        "Progress",
        &format!("{}/{}", directive.progress, directive.target),
        if directive.completed {
            colors.success
        } else {
            colors.warning
        },
    );
    draw_status_row(
        theme,
        right_x + 12.0,
        body_y + 44.0,
        right_w - 24.0,
        "Timer",
        &format!("{:.0}s", directive.duration.max(0.0)),
        colors.primary_soft,
    );
}
