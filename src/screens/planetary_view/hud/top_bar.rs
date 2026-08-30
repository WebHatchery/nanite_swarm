//! Top bar: brand, resource metric cards, navigation buttons, and status banners

use crate::data::UiTheme;
use crate::engine::ResourceType;
use crate::state::{OfflineReport, PlanetState};
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel, draw_metric_card};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

use super::super::format::{format_hours_minutes, format_power_delta};
use super::super::metrics::HudMetrics;
use super::super::PlanetaryAction;
use super::PanelColors;

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

fn mobile_status_text(
    save_failed: bool,
    power_collapse: bool,
    hibernating: bool,
    arriving: bool,
    offline_work: &str,
    hazard: &str,
    world_name: &str,
) -> (String, bool) {
    if save_failed {
        ("SAVE FAILED".to_string(), false)
    } else if power_collapse {
        ("POWER COLLAPSE".to_string(), false)
    } else if hibernating {
        ("HIBERNATION".to_string(), false)
    } else if arriving {
        (format!("ARRIVED {}", world_name.to_uppercase()), true)
    } else if !offline_work.is_empty() {
        (offline_work.to_string(), true)
    } else {
        (hazard.to_string(), false)
    }
}

fn mobile_research_status(
    current_research: Option<&str>,
    progress: f32,
    data: &crate::data::GameData,
) -> Option<String> {
    let id = current_research?;
    let node = data.research.nodes.iter().find(|node| node.id == id)?;
    Some(format!(
        "R {:.0}/{:.0} DATA",
        progress.clamp(0.0, node.data_cost),
        node.data_cost
    ))
}

fn resource_rate_label(state: &PlanetState, resource: ResourceType, rate: &str) -> String {
    if state.resource_is_full(resource) && rate.starts_with('+') {
        "FULL".to_string()
    } else {
        rate.to_string()
    }
}

fn resource_card_color(
    state: &PlanetState,
    resource: ResourceType,
    normal: Color,
    warning: Color,
) -> Color {
    if state.resource_is_full(resource) {
        warning
    } else {
        normal
    }
}

fn compact_resource_readout(state: &PlanetState) -> String {
    let compact = |resource: ResourceType| {
        let value = state.resources.get(resource);
        let capacity = state.resource_capacity(resource);
        if capacity >= 1_000.0 {
            format!("{:.0}k/{:.0}k", value / 1_000.0, capacity / 1_000.0)
        } else {
            format!("{:.0}/{:.0}", value, capacity)
        }
    };
    format!(
        "E{} M{} D{} C{}",
        compact(ResourceType::Energy),
        compact(ResourceType::Minerals),
        compact(ResourceType::Data),
        compact(ResourceType::Components)
    )
}

fn offline_work_label(report: OfflineReport) -> Option<String> {
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

fn mobile_offline_work_label(report: OfflineReport) -> &'static str {
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

fn mobile_offline_report_lines(
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
            "Gains +{:.0} ore +{:.0} alloy +{:.0} parts +{:.0} data P {:+.0}",
            report.minerals_gained,
            report.alloy_gained,
            report.components_gained,
            report.data_gained,
            report.power_gained
        ),
        format!("Work: {work}"),
    ]
}

pub(super) fn draw_mobile_offline_report(
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

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    state: &PlanetState,
    screen_w: f32,
    metrics: HudMetrics,
    theme: &UiTheme,
    colors: &PanelColors,
) -> PlanetaryAction {
    if metrics.mobile {
        return draw_mobile(state, screen_w, metrics, theme, colors);
    }
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

    let mut notice_y = metrics.top_bar_height + 6.0;

    // Offline progress banner
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
        if let Some(work_note) = work_note {
            draw_ui_text(
                &format!("Work: {work_note}"),
                banner_x + 16.0,
                banner_y + 54.0,
                10.0,
                primary_soft,
            );
        }
        notice_y += banner_h + 4.0;
    }

    if state.arrival_notice_timer > 0.0 {
        let banner_w = 620.0;
        let banner_h = 36.0;
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
            banner_y + 24.0,
            13.0,
            primary_soft,
        );
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
        draw_ui_text(&notice, banner_x + 16.0, banner_y + 24.0, 13.0, error);
    }

    ui_action
}

fn draw_mobile(
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
    if let Some(research_status) = mobile_research_status(
        state.research.current_research.as_deref(),
        state.research.research_progress,
        state.data(),
    ) {
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

#[cfg(test)]
mod tests;
