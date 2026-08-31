//! Top bar: brand, resource metric cards, navigation buttons, and status banners

use crate::data::UiTheme;
use crate::engine::ResourceType;
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel, draw_metric_card};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::format::format_power_delta;
use super::super::metrics::HudMetrics;
use super::super::PlanetaryAction;
use super::PanelColors;

mod mobile;
mod notices;
mod readouts;

pub(super) use notices::{draw_mobile_offline_report, mobile_offline_work_label};
use readouts::{
    compact_hidden_resource_readout, compact_resource_readout_for_screen,
    mobile_arrival_guidance_label, mobile_research_status, mobile_status_text, resource_card_color,
    resource_rate_label,
};
#[cfg(test)]
use readouts::{compact_resource_amount, compact_resource_readout};

#[derive(Debug, Clone, Copy, PartialEq)]
struct TopBarLayout {
    compact: bool,
    abbreviated_labels: bool,
    brand_width: f32,
    actions_width: f32,
    card_gap: f32,
    card_width: f32,
    card_count: usize,
}

fn top_bar_layout(screen_width: f32, top_bar_height: f32, panel_gap: f32) -> TopBarLayout {
    let compact = screen_width < 1180.0 || top_bar_height < 80.0;
    let narrow = screen_width < 720.0;
    let brand_width = if narrow {
        150.0
    } else {
        (screen_width * 0.18).clamp(188.0, 318.0)
    };
    let actions_width = if narrow {
        236.0
    } else {
        (screen_width * 0.25).clamp(304.0, 448.0)
    };
    let card_gap = if compact { 6.0 } else { 10.0 };
    let card_count = if compact { 4 } else { 6 };
    let cards_x = brand_width + panel_gap;
    let action_x = screen_width - actions_width - 8.0;
    let card_area_width = (action_x - cards_x - panel_gap).max(0.0);
    let card_width = ((card_area_width - card_gap * (card_count - 1) as f32) / card_count as f32)
        .max(if narrow { 42.0 } else { 52.0 });
    TopBarLayout {
        compact,
        abbreviated_labels: narrow,
        brand_width,
        actions_width,
        card_gap,
        card_width,
        card_count,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    state: &PlanetState,
    screen_w: f32,
    metrics: HudMetrics,
    theme: &UiTheme,
    colors: &PanelColors,
) -> PlanetaryAction {
    if metrics.mobile {
        return mobile::draw(state, screen_w, metrics, theme, colors);
    }
    let mut ui_action = PlanetaryAction::None;
    let dim = colors.dim;
    let primary = colors.primary;
    let primary_soft = colors.primary_soft;
    let warning = colors.warning;
    let power_color = colors.power;

    draw_hud_panel(
        theme,
        Rect::new(0.0, 0.0, screen_w, metrics.top_bar_height),
        None,
    );
    let layout = top_bar_layout(screen_w, metrics.top_bar_height, metrics.panel_gap);
    let title_size = if layout.compact {
        theme.typography.title - 3.0
    } else {
        theme.typography.title
    };
    draw_ui_text("NANITE SWARM", 66.0, 31.0, title_size, primary);
    let hazard = state.hazard_label();
    let subtitle = if hazard.is_empty() {
        format!("{} SECTOR 7-B", state.name.to_uppercase())
    } else {
        format!("{} SECTOR 7-B - {}", state.name.to_uppercase(), hazard)
    };
    draw_ui_text(
        &subtitle,
        66.0,
        55.0,
        if layout.compact {
            theme.typography.small
        } else {
            theme.typography.body
        },
        if hazard.is_empty() { dim } else { warning },
    );
    if layout.compact {
        draw_ui_text(
            &compact_hidden_resource_readout(state),
            66.0,
            70.0,
            8.0,
            dim,
        );
    }
    draw_line(24.0, 24.0, 36.0, 14.0, 1.5, primary);
    draw_line(24.0, 24.0, 36.0, 34.0, 1.5, primary);
    draw_line(48.0, 24.0, 36.0, 14.0, 1.5, primary_soft);
    draw_line(48.0, 24.0, 36.0, 34.0, 1.5, primary_soft);
    draw_circle(36.0, 24.0, 4.0, primary);

    let cards_x = layout.brand_width + metrics.panel_gap;
    let cards_y = 9.0;
    let card_gap = layout.card_gap;
    let actions_w = layout.actions_width;
    let action_x = screen_w - actions_w - 8.0;
    let card_w = layout.card_width;
    let card_h = if layout.compact { 62.0 } else { 68.0 };
    let (energy_label, minerals_label, data_label, parts_label) = if layout.abbreviated_labels {
        ("ENG", "MIN", "DATA", "PARTS")
    } else {
        ("ENERGY", "MINERALS", "DATA", "COMPONENTS")
    };
    let energy_value = format!("{:.0}", state.resources.energy);
    let energy_rate = resource_rate_label(
        state,
        ResourceType::Energy,
        &format_power_delta(state.power_balance),
    );
    let minerals_value = format!("{:.0}", state.resources.minerals);
    let minerals_rate = resource_rate_label(
        state,
        ResourceType::Minerals,
        &format!("+{:.1}/s", state.mineral_output_rate()),
    );
    let minerals_cap = format!("/ {:.0}", state.resource_capacity(ResourceType::Minerals));
    let data_value = format!("{:.0}", state.resources.data);
    let data_rate = resource_rate_label(
        state,
        ResourceType::Data,
        &format!("+{:.2}/s", state.data_generation_rate()),
    );
    let biomass_value = format!("{:.0}", state.resources.biomass);
    let biomass_rate = resource_rate_label(
        state,
        ResourceType::Biomass,
        &format!("+{:.1}/s", state.biomass_harvest_rate()),
    );
    let alloy_value = format!("{:.0}", state.resources.alloy);
    let alloy_rate = resource_rate_label(
        state,
        ResourceType::Alloy,
        &format!("+{:.1}/s", state.observed_alloy_rate()),
    );
    let components_value = format!("{:.0}", state.resources.components);
    let components_rate = resource_rate_label(
        state,
        ResourceType::Components,
        &format!("+{:.1}/s", state.observed_components_rate()),
    );
    let energy_cap = format!("/ {:.0}", state.resource_capacity(ResourceType::Energy));
    let data_cap = format!("/ {:.0}", state.resource_capacity(ResourceType::Data));
    let biomass_cap = format!("/ {:.0}", state.resource_capacity(ResourceType::Biomass));
    let alloy_cap = format!("/ {:.0}", state.resource_capacity(ResourceType::Alloy));
    let components_cap = format!("/ {:.0}", state.resource_capacity(ResourceType::Components));

    draw_metric_card(
        theme,
        Rect::new(cards_x, cards_y, card_w, card_h),
        "energy",
        energy_label,
        &energy_value,
        &energy_rate,
        Some(&energy_cap),
        resource_card_color(
            state,
            ResourceType::Energy,
            color_from_rgba(&theme.colors.energy),
            warning,
        ),
    );
    draw_metric_card(
        theme,
        Rect::new(cards_x + (card_w + card_gap), cards_y, card_w, card_h),
        "minerals",
        minerals_label,
        &minerals_value,
        &minerals_rate,
        Some(&minerals_cap),
        resource_card_color(
            state,
            ResourceType::Minerals,
            color_from_rgba(&theme.colors.minerals),
            warning,
        ),
    );
    draw_metric_card(
        theme,
        Rect::new(cards_x + 2.0 * (card_w + card_gap), cards_y, card_w, card_h),
        "data",
        data_label,
        &data_value,
        &data_rate,
        Some(&data_cap),
        resource_card_color(
            state,
            ResourceType::Data,
            color_from_rgba(&theme.colors.data),
            warning,
        ),
    );
    if layout.compact {
        draw_metric_card(
            theme,
            Rect::new(cards_x + 3.0 * (card_w + card_gap), cards_y, card_w, card_h),
            "components",
            parts_label,
            &components_value,
            &components_rate,
            Some(&components_cap),
            resource_card_color(
                state,
                ResourceType::Components,
                color_from_rgba(&theme.colors.components),
                warning,
            ),
        );
    } else {
        draw_metric_card(
            theme,
            Rect::new(cards_x + 3.0 * (card_w + card_gap), cards_y, card_w, card_h),
            "biomass",
            "BIOMASS",
            &biomass_value,
            &biomass_rate,
            Some(&biomass_cap),
            resource_card_color(
                state,
                ResourceType::Biomass,
                color_from_rgba(&theme.colors.biomass),
                warning,
            ),
        );
        draw_metric_card(
            theme,
            Rect::new(cards_x + 4.0 * (card_w + card_gap), cards_y, card_w, card_h),
            "alloy",
            "ALLOY",
            &alloy_value,
            &alloy_rate,
            Some(&alloy_cap),
            resource_card_color(
                state,
                ResourceType::Alloy,
                color_from_rgba(&theme.colors.alloy),
                warning,
            ),
        );
        draw_metric_card(
            theme,
            Rect::new(cards_x + 5.0 * (card_w + card_gap), cards_y, card_w, card_h),
            "components",
            "COMPONENTS",
            &components_value,
            &components_rate,
            Some(&components_cap),
            resource_card_color(
                state,
                ResourceType::Components,
                color_from_rgba(&theme.colors.components),
                warning,
            ),
        );
    }

    let button_y = 10.0;
    let button_gap = 6.0;
    let button_w = (actions_w - button_gap * 3.0) / 4.0;
    let button_slot = |index: f32| action_x + (button_w + button_gap) * index;
    if draw_hud_button(
        theme,
        Rect::new(button_slot(0.0), button_y, button_w, 34.0),
        "RESEARCH",
    ) {
        ui_action = PlanetaryAction::OpenResearch;
    }
    if draw_hud_button(
        theme,
        Rect::new(button_slot(1.0), button_y, button_w, 34.0),
        "SHIP",
    ) {
        ui_action = PlanetaryAction::OpenSeedShip;
    }
    if draw_hud_button(
        theme,
        Rect::new(button_slot(2.0), button_y, button_w, 34.0),
        "MAP",
    ) {
        ui_action = PlanetaryAction::OpenInterplanetary;
    }
    if draw_hud_button(
        theme,
        Rect::new(button_slot(3.0), button_y, button_w, 34.0),
        "MENU",
    ) {
        ui_action = PlanetaryAction::OpenMenu;
    }

    draw_ui_text(
        &format!("Power {:+.0}/s", state.power_balance),
        action_x + 2.0,
        62.0,
        theme.typography.small,
        power_color,
    );
    let (hours, minutes) = state.battery_time_left();
    draw_ui_text(
        &format!("Battery {}h {}m", hours, minutes),
        action_x + actions_w * 0.36,
        62.0,
        theme.typography.small,
        primary_soft,
    );
    draw_ui_text(
        &format!(
            "{} / {}",
            state.factory_depth_label(),
            state.factory_focus.short_label()
        ),
        action_x + actions_w * 0.62,
        62.0,
        theme.typography.small,
        if state.factory_focus == crate::state::FactoryFocus::Balanced {
            dim
        } else {
            primary
        },
    );
    if state.battery_seconds <= 0.0 {
        draw_ui_text(
            "HIBERNATION",
            action_x + actions_w * 0.7,
            62.0,
            theme.typography.small,
            warning,
        );
    }

    notices::draw(state, screen_w, metrics.top_bar_height, theme, colors);

    ui_action
}

#[cfg(test)]
mod tests;
