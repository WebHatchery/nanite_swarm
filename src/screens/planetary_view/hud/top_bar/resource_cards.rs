//! Desktop resource cards for the planetary top bar.

use crate::data::UiTheme;
use crate::engine::ResourceType;
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_metric_card};
use macroquad::prelude::{Color, Rect};

use super::super::super::format::format_power_delta;
use super::readouts::{resource_card_color, resource_rate_label};
use super::TopBarLayout;

pub(super) fn draw(
    state: &PlanetState,
    layout: &TopBarLayout,
    panel_gap: f32,
    theme: &UiTheme,
    warning: Color,
) {
    let cards_x = layout.brand_width + panel_gap;
    let cards_y = 9.0;
    let card_gap = layout.card_gap;
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
}
