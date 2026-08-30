//! Right sidebar: the tutorial while it is running, the directive after.
//!
//! One panel, two jobs, and only ever one at a time. The panel used to be
//! titled TUTORIAL and show directive rows, with the tutorial reduced to a step
//! counter squeezed underneath them.

use crate::data::UiTheme;
use crate::directives::Directive;
use crate::state::PlanetState;
use crate::ui::{draw_hud_button, draw_hud_panel, draw_status_row};
use macroquad::prelude::Rect;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

use super::super::format::fit_text_to_width;
use super::{PanelColors, RightStackLayout};

pub(super) fn draw(
    state: &mut PlanetState,
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
        if !state.tutorial_done
            && state.tutorial_hidden
            && draw_hud_button(
                theme,
                Rect::new(
                    right.right_x + right.right_w - 78.0,
                    right.directive.y + 8.0,
                    66.0,
                    26.0,
                ),
                "SHOW HELP",
            )
        {
            state.tutorial_hidden = false;
        }
    }
}

fn draw_tutorial(
    state: &mut PlanetState,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
) {
    let Some(step) = state.tutorial_current() else {
        return;
    };
    let right_x = right.right_x;
    let right_w = right.right_w;
    if right.compact {
        let title = step.title.clone();
        let instruction = step.instruction.clone();
        draw_compact_tutorial(state, &title, &instruction, theme, colors, right);
        return;
    }
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
    if draw_hud_button(
        theme,
        Rect::new(
            right_x + right_w - 62.0,
            right.directive.y + 8.0,
            50.0,
            26.0,
        ),
        "HIDE",
    ) {
        state.tutorial_hidden = true;
    }
}

fn draw_directive(
    directive: &Directive,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
) {
    let right_x = right.right_x;
    let right_w = right.right_w;
    if right.compact {
        draw_compact_directive(directive, theme, colors, right);
        return;
    }
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

fn draw_compact_tutorial(
    state: &mut PlanetState,
    step_title: &str,
    step_instruction: &str,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
) {
    draw_hud_panel(theme, right.directive, None);
    draw_ui_text(
        "TUTORIAL",
        right.right_x + 12.0,
        right.directive.y + 18.0,
        theme.typography.section,
        colors.primary,
    );
    if draw_hud_button(
        theme,
        Rect::new(
            right.right_x + right.right_w - 58.0,
            right.directive.y + 5.0,
            50.0,
            22.0,
        ),
        "HIDE",
    ) {
        state.tutorial_hidden = true;
    }

    let step_label = format!(
        "{}/{}  {}",
        state.tutorial_step.saturating_add(1),
        state.tutorial_step_count(),
        step_title
    );
    if right.directive.h < 68.0 {
        let summary = fit_text_to_width(
            &format!("{}: {}", step_label, step_instruction),
            right.right_w - 24.0,
            9.0,
        );
        draw_ui_text(
            &summary,
            right.right_x + 12.0,
            right.directive.y + 40.0,
            9.0,
            colors.text,
        );
    } else {
        let title = fit_text_to_width(&step_label, right.right_w - 86.0, 10.0);
        draw_ui_text(
            &title,
            right.right_x + 12.0,
            right.directive.y + 44.0,
            10.0,
            colors.primary_soft,
        );
        let lines = wrap_text_to_width(step_instruction, right.right_w - 24.0, 10.0);
        let first_y = right.directive.y + right.directive.h
            - 16.0
            - (lines.len().saturating_sub(1) as f32 * 12.0);
        for (index, line) in lines.iter().enumerate() {
            draw_ui_text(
                line,
                right.right_x + 12.0,
                first_y + index as f32 * 12.0,
                10.0,
                colors.text,
            );
        }
    }
}

fn draw_compact_directive(
    directive: &Directive,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
) {
    draw_hud_panel(theme, right.directive, None);
    draw_ui_text(
        "DIRECTIVE",
        right.right_x + 12.0,
        right.directive.y + 18.0,
        theme.typography.section,
        colors.primary,
    );
    if right.directive.h < 68.0 {
        let summary = fit_text_to_width(
            &format!(
                "{}  {}/{}  {:.0}s",
                directive.description,
                directive.progress,
                directive.target,
                directive.duration.max(0.0)
            ),
            right.right_w - 24.0,
            9.0,
        );
        draw_ui_text(
            &summary,
            right.right_x + 12.0,
            right.directive.y + 40.0,
            9.0,
            colors.text,
        );
    } else {
        let description = fit_text_to_width(&directive.description, right.right_w - 24.0, 10.0);
        draw_ui_text(
            &description,
            right.right_x + 12.0,
            right.directive.y + 44.0,
            10.0,
            colors.text,
        );
        draw_ui_text(
            &format!(
                "PROGRESS {}/{}   TIMER {:.0}s",
                directive.progress,
                directive.target,
                directive.duration.max(0.0)
            ),
            right.right_x + 12.0,
            right.directive.y + right.directive.h - 16.0,
            10.0,
            if directive.completed {
                colors.success
            } else {
                colors.warning
            },
        );
    }
}

fn wrap_text_to_width(text: &str, width: f32, font_size: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        let measured = measure_ui_text(&candidate, None, font_size as u16, 1.0).width;
        if measured <= width || current.is_empty() {
            current = candidate;
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
