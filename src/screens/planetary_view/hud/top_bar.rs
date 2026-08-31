//! Top bar: brand, resource metric cards, navigation buttons, and status banners

use crate::data::UiTheme;
use crate::state::PlanetState;
use crate::ui::{draw_hud_button, draw_hud_panel};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::metrics::HudMetrics;
use super::super::PlanetaryAction;
use super::PanelColors;

mod mobile;
mod notices;
mod readouts;
mod resource_cards;

pub(super) use notices::{draw_mobile_offline_report, mobile_offline_work_label};
use readouts::{
    compact_hidden_resource_readout, compact_resource_readout_for_screen,
    mobile_arrival_guidance_label, mobile_research_status, mobile_status_text,
};
#[cfg(test)]
use readouts::{compact_resource_amount, compact_resource_readout, resource_rate_label};

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

    let actions_w = layout.actions_width;
    let action_x = screen_w - actions_w - 8.0;
    resource_cards::draw(state, &layout, metrics.panel_gap, theme, warning);

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
