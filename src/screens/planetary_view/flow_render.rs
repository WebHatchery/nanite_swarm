//! Optional factory-flow overlay: live recipe nodes and intended supply runs.

use crate::data::{RecipeDef, UiTheme};
use crate::engine::{BuildingType, GridPos, ResourceType};
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_panel, draw_resource_icon, resource_color, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::draw_ui_text;

use super::metrics::{grid_to_screen, HudMetrics};

#[derive(Debug, Clone)]
struct FlowLink {
    resource: ResourceType,
    path: Vec<GridPos>,
    peak_load: u32,
    capacity: f32,
}

#[derive(Debug, Clone)]
struct FlowNode {
    pos: GridPos,
    inputs: Vec<ResourceType>,
    outputs: Vec<ResourceType>,
    readiness: f32,
    powered: bool,
    blocked: bool,
    output_pressure: f32,
    priority: bool,
    standby: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct FactoryLedger {
    processors: usize,
    active: usize,
    starved: usize,
    blocked: usize,
    boosted: usize,
    priority: usize,
    standby: usize,
    congested: usize,
    peak_network_load: u32,
    network_capacity: f32,
    bottleneck: Option<ResourceType>,
    ore_rate: f32,
    alloy_rate: f32,
    alloy_capacity: f32,
    components_rate: f32,
    components_capacity: f32,
    auto_clocking: bool,
}

pub(super) fn draw(state: &PlanetState, metrics: HudMetrics, theme: &UiTheme, time: f32) {
    if !state.flow_overlay {
        return;
    }

    for link in factory_flow_links(state) {
        draw_link(&link, metrics, theme, time);
    }

    let starved: std::collections::HashSet<GridPos> =
        state.starved_factories().into_iter().collect();
    for node in factory_flow_nodes(state) {
        draw_node(&node, starved.contains(&node.pos), metrics, theme);
    }

    draw_factory_ledger(&factory_ledger(state), metrics, theme);

    draw_ui_text(
        "FACTORY FLOW // LIVE ROUTES",
        metrics.base_offset_x() + 12.0,
        metrics.base_offset_y() + 53.0,
        10.0,
        color_from_rgba(&theme.colors.primary_soft),
    );
    draw_node_key(metrics, theme);
}

fn draw_node_key(metrics: HudMetrics, theme: &UiTheme) {
    let x = metrics.base_offset_x() + 12.0;
    let y = metrics.base_offset_y() + 69.0;
    draw_ui_text(
        "NODE BARS",
        x,
        y,
        8.0,
        color_from_rgba(&theme.colors.text_dim),
    );
    draw_line(
        x + 58.0,
        y - 3.0,
        x + 72.0,
        y - 3.0,
        2.0,
        color_from_rgba(&theme.colors.success),
    );
    draw_ui_text(
        "INPUT",
        x + 77.0,
        y,
        8.0,
        color_from_rgba(&theme.colors.text_dim),
    );
    draw_line(
        x + 116.0,
        y - 10.0,
        x + 116.0,
        y,
        2.0,
        color_from_rgba(&theme.colors.components),
    );
    draw_ui_text(
        "OUTPUT",
        x + 122.0,
        y,
        8.0,
        color_from_rgba(&theme.colors.text_dim),
    );
    draw_circle(
        x + 58.0,
        y + 8.0,
        4.0,
        color_from_rgba(&theme.colors.primary),
    );
    draw_ui_text("P", x + 56.0, y + 11.0, 7.0, Colors::BACKGROUND);
    draw_ui_text(
        "PRIORITY",
        x + 65.0,
        y + 12.0,
        8.0,
        color_from_rgba(&theme.colors.text_dim),
    );
}

fn draw_factory_ledger(ledger: &FactoryLedger, metrics: HudMetrics, theme: &UiTheme) {
    let grid_room = screen_width() - metrics.base_offset_x() - metrics.right_panel_width - 14.0;
    let x = metrics.base_offset_x() + if grid_room >= 500.0 { 210.0 } else { 8.0 };
    let width = (screen_width() - x - metrics.right_panel_width - 14.0).clamp(250.0, 520.0);
    let compact = width < 400.0;
    let area = Rect::new(
        x,
        metrics.base_offset_y() + 8.0,
        width,
        if compact { 108.0 } else { 88.0 },
    );
    draw_hud_panel(theme, area, Some("FACTORY LEDGER"));

    let text = color_from_rgba(&theme.colors.text);
    let dim = color_from_rgba(&theme.colors.text_dim);
    let warning = color_from_rgba(&theme.colors.warning);
    let success = color_from_rgba(&theme.colors.success);
    let summary = format!(
        "PROC {} ACTIVE {} STARVED {} BLOCKED {} BOOST {} PRIO {} PAUSE {}",
        ledger.processors,
        ledger.active,
        ledger.starved,
        ledger.blocked,
        ledger.boosted,
        ledger.priority,
        ledger.standby
    );
    draw_ui_text(&summary, area.x + 12.0, area.y + 50.0, 10.0, text);

    let clock_mode = if ledger.auto_clocking {
        "AUTO CLOCK"
    } else {
        "MANUAL CLOCK"
    };
    let bottleneck = ledger
        .bottleneck
        .map(|resource| {
            let label = if resource == ResourceType::Minerals {
                "ORE".to_string()
            } else {
                resource.id().to_uppercase()
            };
            format!("{} // BOTTLENECK {}", clock_mode, label)
        })
        .unwrap_or_else(|| {
            if ledger.blocked > 0 {
                format!("{} // DISPATCH PAD FULL", clock_mode)
            } else if ledger.congested > 0 {
                format!(
                    "{} // NETWORK SATURATED {}/{}",
                    clock_mode, ledger.peak_network_load, ledger.network_capacity as u32
                )
            } else {
                format!("{} // FLOW STABLE", clock_mode)
            }
        });
    draw_ui_text(
        &bottleneck,
        area.x + 12.0,
        area.y + 71.0,
        9.0,
        if ledger.bottleneck.is_some() || ledger.blocked > 0 || ledger.congested > 0 {
            warning
        } else {
            success
        },
    );

    let rates = [
        (ResourceType::Minerals, "IN", ledger.ore_rate, None),
        (
            ResourceType::Alloy,
            "A",
            ledger.alloy_rate,
            Some(ledger.alloy_capacity),
        ),
        (
            ResourceType::Components,
            "C",
            ledger.components_rate,
            Some(ledger.components_capacity),
        ),
    ];
    let start_x = if compact {
        area.x + 12.0
    } else {
        area.x + area.w - 224.0
    };
    let rate_y = if compact {
        area.y + 82.0
    } else {
        area.y + 59.0
    };
    for (index, (resource, label, rate, capacity)) in rates.into_iter().enumerate() {
        let chip_x = start_x + index as f32 * 74.0;
        draw_resource_icon(
            resource,
            Rect::new(chip_x, rate_y, 10.0, 10.0),
            resource_color(theme, resource),
        );
        draw_ui_text(
            &capacity.map_or_else(
                || format!("{} {:.1}/s", label, rate),
                |rated| format!("{} {:.1}/{:.1}", label, rate, rated),
            ),
            chip_x + 14.0,
            rate_y + 10.0,
            8.0,
            dim,
        );
        if let Some(rated) = capacity {
            let utilization = if rated <= 0.001 {
                0.0
            } else {
                (rate / rated).clamp(0.0, 1.0)
            };
            draw_rectangle(
                chip_x + 14.0,
                rate_y + 14.0,
                50.0,
                1.5,
                with_alpha(Colors::TEXT_DIM, 0.22),
            );
            draw_rectangle(
                chip_x + 14.0,
                rate_y + 14.0,
                50.0 * utilization,
                1.5,
                resource_color(theme, resource),
            );
        }
    }
}

mod model;
use model::*;

#[cfg(test)]
mod tests;
