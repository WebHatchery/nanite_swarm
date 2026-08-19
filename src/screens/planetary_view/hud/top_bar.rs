//! Top bar: brand, resource metric cards, navigation buttons, and status banners

use crate::data::UiTheme;
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel, draw_metric_card};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::format::{format_hours_minutes, format_power_delta};
use super::super::metrics::HudMetrics;
use super::super::PlanetaryAction;
use super::PanelColors;

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    state: &PlanetState,
    screen_w: f32,
    metrics: HudMetrics,
    theme: &UiTheme,
    colors: &PanelColors,
) -> PlanetaryAction {
    let mut ui_action = PlanetaryAction::None;
    let dim = colors.dim;
    let primary = colors.primary;
    let primary_soft = colors.primary_soft;
    let success = colors.success;
    let warning = colors.warning;
    let error = colors.error;
    let power_color = colors.power;

    draw_hud_panel(
        theme,
        Rect::new(0.0, 0.0, screen_w, metrics.top_bar_height),
        None,
    );
    let compact_top = screen_w < 1180.0 || metrics.top_bar_height < 80.0;
    let title_size = if compact_top {
        theme.typography.title - 3.0
    } else {
        theme.typography.title
    };
    let brand_w = (screen_w * 0.18).clamp(188.0, 318.0);
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
        if compact_top {
            theme.typography.small
        } else {
            theme.typography.body
        },
        if hazard.is_empty() { dim } else { warning },
    );
    draw_line(24.0, 24.0, 36.0, 14.0, 1.5, primary);
    draw_line(24.0, 24.0, 36.0, 34.0, 1.5, primary);
    draw_line(48.0, 24.0, 36.0, 14.0, 1.5, primary_soft);
    draw_line(48.0, 24.0, 36.0, 34.0, 1.5, primary_soft);
    draw_circle(36.0, 24.0, 4.0, primary);

    let cards_x = brand_w + metrics.panel_gap;
    let cards_y = 9.0;
    let card_gap = if compact_top { 6.0 } else { 10.0 };
    let actions_w = (screen_w * 0.25).clamp(304.0, 448.0);
    let action_x = screen_w - actions_w - 8.0;
    let card_area_w = (action_x - cards_x - metrics.panel_gap).max(0.0);
    let card_w = ((card_area_w - card_gap * 5.0) / 6.0).max(52.0);
    let card_h = if compact_top { 62.0 } else { 68.0 };
    let energy_value = format!("{:.0}", state.resources.energy);
    let energy_rate = format_power_delta(state.power_balance);
    let minerals_value = format!("{:.0}", state.resources.minerals);
    let minerals_rate = format!("+{:.1}/s", state.drill_output_rate());
    let minerals_cap = format!("/ {:.0}", state.mineral_capacity());
    let data_value = format!("{:.0}", state.resources.data);
    let data_rate = format!("+{:.2}/s", state.config.resources.core_data_rate);
    let biomass_value = format!("{:.0}", state.resources.biomass);
    let biomass_rate = format!("+{:.1}/s", state.biomass_power_bonus.max(0.0));
    let alloy_value = format!("{:.0}", state.resources.alloy);
    let alloy_rate = format!("+{:.1}/s", state.observed_alloy_rate());
    let components_value = format!("{:.0}", state.resources.components);
    let components_rate = format!("+{:.1}/s", state.observed_components_rate());

    draw_metric_card(
        theme,
        Rect::new(cards_x, cards_y, card_w, card_h),
        "energy",
        "ENERGY",
        &energy_value,
        &energy_rate,
        None,
        color_from_rgba(&theme.colors.energy),
    );
    draw_metric_card(
        theme,
        Rect::new(cards_x + (card_w + card_gap), cards_y, card_w, card_h),
        "minerals",
        "MINERALS",
        &minerals_value,
        &minerals_rate,
        Some(&minerals_cap),
        color_from_rgba(&theme.colors.minerals),
    );
    draw_metric_card(
        theme,
        Rect::new(cards_x + 2.0 * (card_w + card_gap), cards_y, card_w, card_h),
        "data",
        "DATA",
        &data_value,
        &data_rate,
        None,
        color_from_rgba(&theme.colors.data),
    );
    draw_metric_card(
        theme,
        Rect::new(cards_x + 3.0 * (card_w + card_gap), cards_y, card_w, card_h),
        "biomass",
        "BIOMASS",
        &biomass_value,
        &biomass_rate,
        Some("/ 1000"),
        color_from_rgba(&theme.colors.biomass),
    );
    draw_metric_card(
        theme,
        Rect::new(cards_x + 4.0 * (card_w + card_gap), cards_y, card_w, card_h),
        "alloy",
        "ALLOY",
        &alloy_value,
        &alloy_rate,
        Some("/ 1000"),
        color_from_rgba(&theme.colors.alloy),
    );
    draw_metric_card(
        theme,
        Rect::new(cards_x + 5.0 * (card_w + card_gap), cards_y, card_w, card_h),
        "components",
        if compact_top { "PARTS" } else { "COMPONENTS" },
        &components_value,
        &components_rate,
        Some("/ 1000"),
        color_from_rgba(&theme.colors.components),
    );

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
    if state.battery_seconds <= 0.0 {
        draw_ui_text(
            "HIBERNATION",
            action_x + actions_w * 0.7,
            62.0,
            theme.typography.small,
            warning,
        );
    }

    // Offline progress banner
    if state.offline_notice_timer > 0.0 && state.last_offline_seconds > 0.0 {
        let (off_h, off_m) = format_hours_minutes(state.last_offline_seconds);
        let (sim_h, sim_m) = format_hours_minutes(state.last_offline_simulated);
        let banner_w = 520.0;
        let banner_h = 48.0;
        let banner_x = (screen_w - banner_w) * 0.5;
        let banner_y = metrics.top_bar_height + 6.0;
        draw_hud_panel(
            theme,
            Rect::new(banner_x, banner_y, banner_w, banner_h),
            None,
        );
        let capped_note = if state.last_offline_report.capped_seconds
            < state.last_offline_report.elapsed_seconds
        {
            " | capped"
        } else {
            ""
        };
        let clock_note = if state.last_offline_report.tamper_guarded {
            " | clock check"
        } else {
            ""
        };
        let banner_text = format!(
            "Offline {}h {}m | Simulated {}h {}m{}{}",
            off_h, off_m, sim_h, sim_m, capped_note, clock_note
        );
        draw_ui_text(
            &banner_text,
            banner_x + 16.0,
            banner_y + 18.0,
            12.0,
            success,
        );
        draw_ui_text(
            &format!(
                "Gains: +{:.0} ore  +{:.0} alloy  +{:.0} parts  +{:.0} data  power {:+.0}",
                state.last_offline_report.minerals_gained,
                state.last_offline_report.alloy_gained,
                state.last_offline_report.components_gained,
                state.last_offline_report.data_gained,
                state.last_offline_report.power_gained
            ),
            banner_x + 16.0,
            banner_y + 36.0,
            11.0,
            success,
        );
    }

    if state.arrival_notice_timer > 0.0 {
        let banner_w = 620.0;
        let banner_h = 36.0;
        let banner_x = (screen_w - banner_w) * 0.5;
        let banner_y = metrics.top_bar_height + 6.0;
        draw_hud_panel(
            theme,
            Rect::new(banner_x, banner_y, banner_w, banner_h),
            None,
        );
        draw_ui_text(
            state.arrival_line(),
            banner_x + 16.0,
            banner_y + 24.0,
            13.0,
            primary_soft,
        );
    }

    if state.collapse_notice_timer > 0.0 {
        let banner_w = 520.0;
        let banner_h = 36.0;
        let banner_x = (screen_w - banner_w) * 0.5;
        let banner_y = metrics.top_bar_height + 46.0;
        draw_hud_panel(
            theme,
            Rect::new(banner_x, banner_y, banner_w, banner_h),
            None,
        );
        // The shutdown scales with the size of what collapsed, so the banner
        // has to say how long rather than leave the player guessing.
        let source = state
            .latest_collapse_source()
            .map(|source| format!(" | {}", source))
            .unwrap_or_default();
        let notice = if state.power_collapse_shutdown > 0.0 {
            format!(
                "POWER COLLAPSE: drones offline for {:.0}s, data corrupted, research locked{}",
                state.power_collapse_shutdown, source
            )
        } else {
            format!("POWER COLLAPSE: data corrupted, research locked{}", source)
        };
        draw_ui_text(&notice, banner_x + 16.0, banner_y + 24.0, 13.0, error);
    }

    ui_action
}
