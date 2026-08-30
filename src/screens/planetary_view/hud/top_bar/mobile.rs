//! Compact top-bar renderer for touch-sized planetary views.

use crate::data::UiTheme;
use crate::state::PlanetState;
use crate::ui::{draw_hud_button, draw_hud_panel};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

use super::super::super::metrics::HudMetrics;
use super::super::super::PlanetaryAction;
use super::super::PanelColors;
use super::{
    compact_resource_readout, mobile_arrival_guidance_label, mobile_offline_work_label,
    mobile_research_status, mobile_status_text,
};

pub(super) fn draw(
    state: &PlanetState,
    screen_w: f32,
    metrics: HudMetrics,
    theme: &UiTheme,
    colors: &PanelColors,
) -> PlanetaryAction {
    draw_hud_panel(
        theme,
        Rect::new(0.0, 0.0, screen_w, metrics.top_bar_height),
        None,
    );
    draw_line(24.0, 24.0, 36.0, 14.0, 1.5, colors.primary);
    draw_line(24.0, 24.0, 36.0, 34.0, 1.5, colors.primary);
    draw_line(48.0, 24.0, 36.0, 14.0, 1.5, colors.primary_soft);
    draw_line(48.0, 24.0, 36.0, 34.0, 1.5, colors.primary_soft);
    draw_circle(36.0, 24.0, 4.0, colors.primary);
    draw_ui_text("NANITE SWARM", 58.0, 18.0, 11.0, colors.primary);
    draw_ui_text(
        &compact_resource_readout(state),
        58.0,
        31.0,
        8.0,
        colors.dim,
    );
    let (status, arrival_status) = mobile_status_text(
        state.save_failed,
        state.power_collapse_shutdown > 0.0,
        state.battery_seconds <= 0.0,
        state.arrival_notice_timer > 0.0,
        if state.offline_notice_timer > 0.0 && state.last_offline_seconds > 0.0 {
            mobile_offline_work_label(state.last_offline_report)
        } else {
            ""
        },
        &state.hazard_label(),
        &state.name,
    );
    if !status.is_empty() {
        draw_ui_text(
            &status,
            screen_w - 108.0,
            20.0,
            8.0,
            if arrival_status {
                colors.primary_soft
            } else {
                colors.warning
            },
        );
    }
    let research_status = mobile_research_status(
        state.research.current_research.as_deref(),
        state.research.research_progress,
        state.data(),
    );
    if let Some(research_status) = research_status {
        draw_ui_text(
            &truncate_text_to_width(&research_status, 108.0, 8.0),
            screen_w - 108.0,
            31.0,
            8.0,
            if state.research_lock_timer > 0.0 {
                colors.warning
            } else {
                colors.primary_soft
            },
        );
    } else if state.arrival_notice_timer > 0.0 {
        if let Some(guidance) = state.arrival_guidance() {
            draw_ui_text(
                mobile_arrival_guidance_label(&guidance),
                screen_w - 108.0,
                31.0,
                8.0,
                colors.warning,
            );
        }
    }

    let gap = 6.0;
    let button_w = ((screen_w - 20.0 - gap * 3.0) / 4.0).max(48.0);
    let labels = ["RESEARCH", "SHIP", "MAP", "MENU"];
    let mut action = PlanetaryAction::None;
    for (index, label) in labels.iter().enumerate() {
        if draw_hud_button(
            theme,
            Rect::new(10.0 + (button_w + gap) * index as f32, 38.0, button_w, 30.0),
            label,
        ) {
            action = match index {
                0 => PlanetaryAction::OpenResearch,
                1 => PlanetaryAction::OpenSeedShip,
                2 => PlanetaryAction::OpenInterplanetary,
                _ => PlanetaryAction::OpenMenu,
            };
        }
    }
    action
}
