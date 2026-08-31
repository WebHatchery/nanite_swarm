//! Status notices shared by desktop and touch-sized planetary top bars.

use crate::data::UiTheme;
use crate::state::{OfflineReport, PlanetState};
use crate::ui::draw_hud_panel;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

use super::super::super::format::format_hours_minutes;
use super::super::super::metrics::HudMetrics;
use super::PanelColors;

pub(crate) fn offline_work_label(report: OfflineReport) -> Option<String> {
    let mut work = Vec::new();
    if report.research_completed {
        if report.research_data_spent > 0.001 {
            work.push(format!(
                "research complete (-{:.0} data)",
                report.research_data_spent
            ));
        } else {
            work.push("research complete".to_string());
        }
    } else if report.research_data_spent > 0.001 {
        work.push(format!("research -{:.0} data", report.research_data_spent));
    }
    if report.seed_ship_stages_completed > 0 {
        let suffix = if report.seed_ship_stages_completed == 1 {
            "stage"
        } else {
            "stages"
        };
        work.push(format!(
            "Seed Ship +{} {suffix}",
            report.seed_ship_stages_completed
        ));
    }
    (!work.is_empty()).then(|| work.join(" | "))
}

pub(crate) fn mobile_offline_work_label(report: OfflineReport) -> &'static str {
    if report.research_completed {
        "RESEARCH DONE"
    } else if report.seed_ship_stages_completed > 0 {
        "SHIP ADVANCED"
    } else if report.research_data_spent > 0.001 {
        "RESEARCHING"
    } else {
        ""
    }
}

pub(crate) fn mobile_offline_report_lines(
    report: OfflineReport,
    elapsed_seconds: f32,
    simulated_seconds: f32,
) -> [String; 3] {
    let (off_h, off_m) = format_hours_minutes(elapsed_seconds);
    let (sim_h, sim_m) = format_hours_minutes(simulated_seconds);
    let capped_note = if report.capped_seconds < report.elapsed_seconds {
        " | CAPPED"
    } else {
        ""
    };
    let clock_note = if report.tamper_guarded {
        " | CLOCK CHECK"
    } else {
        ""
    };
    let work = offline_work_label(report).unwrap_or_else(|| "no queued work".to_string());
    [
        format!("Away {off_h}h {off_m}m | Sim {sim_h}h {sim_m}m{capped_note}{clock_note}"),
        format!(
            "Gains +{:.0} ore +{:.0} bio +{:.0} alloy +{:.0} parts +{:.0} data P {:+.0}",
            report.minerals_gained,
            report.biomass_gained,
            report.alloy_gained,
            report.components_gained,
            report.data_gained,
            report.power_gained
        ),
        format!("Work: {work}"),
    ]
}

pub(crate) fn draw_mobile_offline_report(
    state: &PlanetState,
    screen_w: f32,
    metrics: HudMetrics,
    theme: &UiTheme,
    colors: &PanelColors,
) -> bool {
    if state.offline_notice_timer <= 0.0 || state.last_offline_seconds <= 0.0 {
        return false;
    }
    let x = 10.0;
    let y = metrics.top_bar_height + 8.0;
    let width = (screen_w - 20.0).max(0.0);
    draw_hud_panel(theme, Rect::new(x, y, width, 82.0), None);
    draw_ui_text("OFFLINE RETURN", x + 12.0, y + 18.0, 11.0, colors.primary);
    let text_width = (width - 24.0).max(40.0);
    for (index, line) in mobile_offline_report_lines(
        state.last_offline_report,
        state.last_offline_seconds,
        state.last_offline_simulated,
    )
    .iter()
    .enumerate()
    {
        draw_ui_text(
            &truncate_text_to_width(line, text_width, 8.0),
            x + 12.0,
            y + 40.0 + index as f32 * 12.0,
            8.0,
            if index == 2 {
                colors.primary_soft
            } else {
                colors.success
            },
        );
    }
    true
}

pub(super) fn draw(
    state: &PlanetState,
    screen_w: f32,
    top_bar_height: f32,
    theme: &UiTheme,
    colors: &PanelColors,
) {
    let mut notice_y = top_bar_height + 6.0;

    if state.offline_notice_timer > 0.0 && state.last_offline_seconds > 0.0 {
        let (off_h, off_m) = format_hours_minutes(state.last_offline_seconds);
        let (sim_h, sim_m) = format_hours_minutes(state.last_offline_simulated);
        let banner_w = 520.0;
        let work_note = offline_work_label(state.last_offline_report);
        let banner_h = if work_note.is_some() { 64.0 } else { 48.0 };
        let banner_x = (screen_w - banner_w) * 0.5;
        let banner_y = notice_y;
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
            colors.success,
        );
        draw_ui_text(
            &format!(
                "Gains: +{:.0} ore  +{:.0} biomass  +{:.0} alloy  +{:.0} parts  +{:.0} data  power {:+.0}",
                state.last_offline_report.minerals_gained,
                state.last_offline_report.biomass_gained,
                state.last_offline_report.alloy_gained,
                state.last_offline_report.components_gained,
                state.last_offline_report.data_gained,
                state.last_offline_report.power_gained
            ),
            banner_x + 16.0,
            banner_y + 36.0,
            11.0,
            colors.success,
        );
        if let Some(work_note) = work_note {
            draw_ui_text(
                &format!("Work: {work_note}"),
                banner_x + 16.0,
                banner_y + 54.0,
                10.0,
                colors.primary_soft,
            );
        }
        notice_y += banner_h + 4.0;
    }

    if state.arrival_notice_timer > 0.0 {
        let banner_w = 620.0;
        let arrival_guidance = state.arrival_guidance();
        let banner_h = if arrival_guidance.is_some() {
            52.0
        } else {
            36.0
        };
        let banner_x = (screen_w - banner_w) * 0.5;
        let banner_y = notice_y;
        draw_hud_panel(
            theme,
            Rect::new(banner_x, banner_y, banner_w, banner_h),
            None,
        );
        draw_ui_text(
            state.arrival_line(),
            banner_x + 16.0,
            banner_y + 21.0,
            13.0,
            colors.primary_soft,
        );
        if let Some(guidance) = arrival_guidance {
            draw_ui_text(
                &truncate_text_to_width(&guidance, banner_w - 32.0, 10.0),
                banner_x + 16.0,
                banner_y + 41.0,
                10.0,
                colors.warning,
            );
        }
        notice_y += banner_h + 4.0;
    }

    if state.collapse_notice_timer > 0.0 {
        let banner_w = 520.0;
        let banner_h = 36.0;
        let banner_x = (screen_w - banner_w) * 0.5;
        let banner_y = notice_y;
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
        draw_ui_text(
            &notice,
            banner_x + 16.0,
            banner_y + 24.0,
            13.0,
            colors.error,
        );
    }
}
