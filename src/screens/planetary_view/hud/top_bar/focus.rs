//! Minimal top rail for the unobstructed planetary map.

use crate::data::UiTheme;
use crate::engine::ResourceType;
use crate::state::{DesktopPanel, PlanetState};
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel, draw_resource_icon};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::super::metrics::HudMetrics;
use super::super::super::PlanetaryAction;
use super::super::PanelColors;

pub(super) fn draw(
    state: &mut PlanetState,
    screen_w: f32,
    metrics: HudMetrics,
    theme: &UiTheme,
    colors: &PanelColors,
) -> PlanetaryAction {
    let top_y = 8.0;
    let top_h = (metrics.top_bar_height - 16.0).max(36.0);
    let compact = screen_w < 900.0;
    let chip_w = if compact { 164.0 } else { 206.0 };
    let action_w = 386.0;
    let action_x = screen_w - action_w - 10.0;
    let resource_x = chip_w + 18.0;
    let resource_w = (action_x - resource_x - 8.0).max(0.0);

    draw_hud_panel(theme, Rect::new(10.0, top_y, chip_w, top_h), None);
    draw_swarm_mark(
        27.0,
        top_y + top_h * 0.5,
        colors.primary,
        colors.primary_soft,
    );
    draw_ui_text(
        "NANITE SWARM",
        52.0,
        top_y + 19.0,
        if compact { 11.0 } else { 13.0 },
        colors.primary,
    );
    let subtitle = if state.hazard_label().is_empty() {
        format!("{} / SECTOR 7-B", state.name.to_uppercase())
    } else {
        format!("{} / {}", state.name.to_uppercase(), state.hazard_label())
    };
    draw_ui_text(&subtitle, 52.0, top_y + 30.0, 8.0, colors.dim);
    if state.tutorial_current().is_some() && !state.tutorial_hidden {
        draw_ui_text("NEXT: TAP BUILD", 52.0, top_y + 41.0, 8.0, colors.warning);
    }

    if resource_w >= 180.0 {
        draw_resource_strip(
            state,
            theme,
            colors,
            Rect::new(resource_x, top_y, resource_w, top_h),
        );
    }

    let build_label = if !state.focus_mode && state.desktop_panel == DesktopPanel::Build {
        "CLOSE"
    } else {
        "BUILD"
    };
    let info_label = if !state.focus_mode && state.desktop_panel == DesktopPanel::Info {
        "CLOSE"
    } else {
        "INFO"
    };
    let labels = [build_label, info_label, "RESEARCH", "SHIP", "MAP", "MENU"];
    let widths = [54.0, 50.0, 72.0, 48.0, 48.0, 54.0];
    let gap = 6.0;
    let total = widths.iter().sum::<f32>() + gap * 5.0;
    let scale = (action_w / total).min(1.0);
    let mut x = action_x + (action_w - total * scale) * 0.5;
    let mut action = PlanetaryAction::None;
    for (index, label) in labels.iter().enumerate() {
        let width = widths[index] * scale;
        let rect = Rect::new(x, top_y + 5.0, width, (top_h - 10.0).max(28.0));
        if draw_hud_button(theme, rect, label) {
            match index {
                0 => toggle_panel(state, DesktopPanel::Build),
                1 => toggle_panel(state, DesktopPanel::Info),
                2 => action = PlanetaryAction::OpenResearch,
                3 => action = PlanetaryAction::OpenSeedShip,
                4 => action = PlanetaryAction::OpenInterplanetary,
                _ => action = PlanetaryAction::OpenMenu,
            }
        }
        x += width + gap * scale;
    }

    super::notices::draw(state, screen_w, metrics.top_bar_height, theme, colors);

    action
}

fn toggle_panel(state: &mut PlanetState, panel: DesktopPanel) {
    if !state.focus_mode && state.desktop_panel == panel {
        state.desktop_panel = DesktopPanel::None;
        state.focus_mode = true;
    } else {
        state.desktop_panel = panel;
        state.focus_mode = false;
    }
}

fn draw_swarm_mark(x: f32, y: f32, primary: Color, secondary: Color) {
    draw_line(x - 10.0, y, x, y - 9.0, 1.5, primary);
    draw_line(x - 10.0, y, x, y + 9.0, 1.5, primary);
    draw_line(x + 10.0, y, x, y - 9.0, 1.5, secondary);
    draw_line(x + 10.0, y, x, y + 9.0, 1.5, secondary);
    draw_circle(x, y, 3.5, primary);
}

fn draw_resource_strip(state: &PlanetState, theme: &UiTheme, colors: &PanelColors, rect: Rect) {
    draw_hud_panel(theme, rect, None);
    let mut resources = vec![
        (
            ResourceType::Energy,
            "PWR",
            state.resources.energy,
            state.power_balance,
        ),
        (
            ResourceType::Minerals,
            "ORE",
            state.resources.minerals,
            state.mineral_output_rate(),
        ),
        (
            ResourceType::Data,
            "DATA",
            state.resources.data,
            state.data_generation_rate(),
        ),
        (
            ResourceType::Components,
            "PARTS",
            state.resources.components,
            state.observed_components_rate(),
        ),
    ];
    if rect.w >= 500.0 {
        resources.insert(
            3,
            (
                ResourceType::Biomass,
                "BIO",
                state.resources.biomass,
                state.biomass_harvest_rate(),
            ),
        );
        resources.insert(
            4,
            (
                ResourceType::Alloy,
                "ALY",
                state.resources.alloy,
                state.observed_alloy_rate(),
            ),
        );
    }
    let cell_w = rect.w / resources.len() as f32;
    let text = colors.text;
    for (index, (kind, label, value, rate)) in resources.into_iter().enumerate() {
        let x = rect.x + index as f32 * cell_w;
        if index > 0 {
            draw_line(
                x,
                rect.y + 8.0,
                x,
                rect.y + rect.h - 8.0,
                1.0,
                color_from_rgba(&theme.colors.border),
            );
        }
        let accent = crate::ui::resource_color(theme, kind);
        draw_resource_icon(kind, Rect::new(x + 8.0, rect.y + 11.0, 18.0, 18.0), accent);
        draw_ui_text(label, x + 32.0, rect.y + 16.0, 8.0, colors.dim);
        draw_ui_text(&format!("{value:.0}"), x + 32.0, rect.y + 31.0, 13.0, text);
        draw_ui_text(
            &format!("{rate:+.1}/s"),
            x + 32.0,
            rect.y + rect.h - 5.0,
            7.0,
            accent,
        );
    }
}
