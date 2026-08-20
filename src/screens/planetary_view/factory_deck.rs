//! The compact control-room overlay for the factory's deeper decks.

use crate::data::UiTheme;
use crate::state::{FactoryFocus, PlanetState};
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel, draw_resource_icon};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::draw_ui_text;

use super::metrics::HudMetrics;

const DEPTHS: [(&str, &str, Color); 4] = [
    ("01", "SURFACE", Color::new(0.10, 0.72, 0.94, 1.0)),
    ("02", "FOUNDRY", Color::new(0.98, 0.56, 0.22, 1.0)),
    ("03", "ASSEMBLY", Color::new(0.74, 0.35, 1.0, 1.0)),
    ("04", "ORBITAL", Color::new(0.24, 0.92, 0.66, 1.0)),
];

/// Draws the animated deck markers behind the drones and the interactive
/// control room over the map. This keeps the depth language useful in focus
/// mode without making the left/right HUD columns taller.
pub(super) fn draw_factory_depth_art(
    state: &mut PlanetState,
    metrics: HudMetrics,
    theme: &UiTheme,
    time: f32,
) {
    draw_depth_markers(state, metrics, theme, time);
}

pub(super) fn draw_factory_depth_overlay(
    state: &mut PlanetState,
    metrics: HudMetrics,
    theme: &UiTheme,
) {
    if !state.factory_deck_open {
        return;
    }

    let screen_w = screen_width();
    let screen_h = screen_height();
    let compact = screen_w < 900.0 || screen_h < 650.0;
    let panel_w = if compact { 390.0 } else { 500.0 };
    let panel_h = if compact { 268.0 } else { 302.0 };
    let panel_x = ((screen_w - panel_w) * 0.5).max(12.0);
    let panel_y = (metrics.top_bar_height + 18.0).max(18.0);
    let panel = Rect::new(panel_x, panel_y, panel_w, panel_h);
    let text = color_from_rgba(&theme.colors.text);
    let dim = color_from_rgba(&theme.colors.text_dim);
    let primary = color_from_rgba(&theme.colors.primary);
    let success = color_from_rgba(&theme.colors.success);
    let warning = color_from_rgba(&theme.colors.warning);

    draw_rectangle(
        panel.x + 4.0,
        panel.y + 5.0,
        panel.w,
        panel.h,
        with_alpha(color_from_rgba(&theme.colors.shadow), 0.92),
    );
    draw_hud_panel(theme, panel, Some("FACTORY DECK / CONTROL ROOM"));
    draw_ui_text(
        &format!(
            "DEPTH {:02}  {}  //  CORE STAGE {}",
            state.factory_depth() + 1,
            state.factory_depth_label(),
            state.core_stage_index() + 1
        ),
        panel.x + 18.0,
        panel.y + 48.0,
        12.0,
        primary,
    );
    draw_ui_text(
        "Choose one deck to receive the next scheduling cycle.",
        panel.x + 18.0,
        panel.y + 67.0,
        10.0,
        dim,
    );

    let depth = state.factory_depth() as usize;
    let progress = state.factory_depth_progress().0.clamp(0.0, 1.0);
    let rail_y = panel.y + 88.0;
    let rail_x = panel.x + 22.0;
    let rail_w = panel.w - 44.0;
    draw_line(rail_x, rail_y, rail_x + rail_w, rail_y, 2.0, dim);
    for (index, (number, label, color)) in DEPTHS.iter().enumerate() {
        let x = rail_x + index as f32 * rail_w / 3.0;
        let online = index <= depth;
        let node_color = if online {
            *color
        } else {
            with_alpha(*color, 0.25)
        };
        draw_circle(x, rail_y, if online { 7.0 } else { 5.0 }, node_color);
        draw_circle_lines(x, rail_y, 10.0, 1.0, with_alpha(node_color, 0.6));
        draw_ui_text(number, x - 8.0, rail_y - 17.0, 8.0, node_color);
        draw_ui_text(label, x - 27.0, rail_y + 26.0, 8.0, node_color);
    }
    let next_label = state.factory_depth_progress().1;
    let progress_text = if depth >= 3 {
        "ALL DECKS ONLINE".to_string()
    } else {
        format!("NEXT: {}  {:.0}%", next_label, progress * 100.0)
    };
    draw_ui_text(
        &progress_text,
        panel.x + 18.0,
        panel.y + 132.0,
        10.0,
        success,
    );

    let focus_y = panel.y + 151.0;
    let focus_w = (panel.w - 44.0 - 18.0) / 4.0;
    for (index, focus) in FactoryFocus::ALL.into_iter().enumerate() {
        let x = panel.x + 22.0 + index as f32 * (focus_w + 6.0);
        let selected = state.factory_focus == focus;
        let button_color = if selected { primary } else { dim };
        if draw_hud_button(
            theme,
            Rect::new(x, focus_y, focus_w, 30.0),
            focus.short_label(),
        ) {
            state.set_factory_focus(focus);
        }
        if selected {
            draw_rectangle_lines(
                x - 1.0,
                focus_y - 1.0,
                focus_w + 2.0,
                32.0,
                1.5,
                button_color,
            );
        }
    }
    draw_ui_text(
        state.factory_focus.description(),
        panel.x + 22.0,
        panel.y + 202.0,
        10.0,
        if state.factory_focus == FactoryFocus::Balanced {
            dim
        } else {
            warning
        },
    );
    draw_ui_text(
        &format!(
            "Live draw: {:+.1}/s  |  drill {:.1}/s  |  alloy {:.1}/s  |  parts {:.1}/s",
            -state.factory_focus_power_tax(),
            state.drill_output_rate(),
            state.observed_alloy_rate(),
            state.observed_components_rate()
        ),
        panel.x + 22.0,
        panel.y + 223.0,
        9.0,
        text,
    );
    if compact {
        draw_ui_text(
            "Tap DECK to close",
            panel.x + 22.0,
            panel.y + 248.0,
            9.0,
            dim,
        );
    } else {
        draw_ui_text(
            "Focus changes production rates and power draw; it never rewires the map.",
            panel.x + 22.0,
            panel.y + 248.0,
            9.0,
            dim,
        );
    }

    // Resource chips give the deck a visual readout that is distinct from the
    // six full-width cards in the top bar.
    let chips = [
        (
            crate::engine::ResourceType::Minerals,
            state.resources.minerals,
        ),
        (crate::engine::ResourceType::Alloy, state.resources.alloy),
        (
            crate::engine::ResourceType::Components,
            state.resources.components,
        ),
    ];
    for (index, (resource, amount)) in chips.into_iter().enumerate() {
        let x = panel.x + panel.w - 128.0 + index as f32 * 36.0;
        draw_resource_icon(
            resource,
            Rect::new(x, panel.y + 36.0, 16.0, 16.0),
            resource_chip_color(resource, theme),
        );
        draw_ui_text(&format!("{:.0}", amount), x, panel.y + 62.0, 8.0, text);
    }
}

fn draw_depth_markers(state: &PlanetState, metrics: HudMetrics, theme: &UiTheme, time: f32) {
    let Some(core) = state.grid.find_core() else {
        return;
    };
    let (x, y) = super::metrics::grid_to_screen(core, metrics);
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

fn resource_chip_color(resource: crate::engine::ResourceType, theme: &UiTheme) -> Color {
    match resource {
        crate::engine::ResourceType::Minerals => color_from_rgba(&theme.colors.minerals),
        crate::engine::ResourceType::Alloy => color_from_rgba(&theme.colors.alloy),
        crate::engine::ResourceType::Components => color_from_rgba(&theme.colors.components),
        _ => color_from_rgba(&theme.colors.text),
    }
}
