//! The compact control-room overlay for the factory's deeper decks.

use crate::data::UiTheme;
use crate::engine::GridPos;
use crate::state::{FactoryFocus, FactoryFocusReadout, Milestone, PlanetState};
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel, draw_resource_icon};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width};
use std::collections::HashSet;

use super::metrics::HudMetrics;

#[derive(Debug, Clone, Copy, PartialEq)]
struct FactoryDeckLayout {
    panel: Rect,
    compact: bool,
    narrow: bool,
}

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
    if state.factory_deck_open {
        draw_focus_targets(state, metrics, theme, time);
    }
}

pub(super) fn draw_factory_depth_overlay(
    state: &mut PlanetState,
    metrics: HudMetrics,
    theme: &UiTheme,
) {
    if !state.factory_deck_open {
        return;
    }

    let layout = factory_deck_layout(screen_width(), screen_height(), metrics);
    let panel = layout.panel;
    let compact = layout.compact;
    let narrow = layout.narrow;
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
    let depth_line = if narrow {
        format!(
            "DEPTH {:02}  //  {}",
            state.factory_depth() + 1,
            state.factory_depth_label()
        )
    } else {
        format!(
            "DEPTH {:02}  {}  //  CORE STAGE {}",
            state.factory_depth() + 1,
            state.factory_depth_label(),
            state.core_stage_index() + 1
        )
    };
    draw_ui_text(&depth_line, panel.x + 18.0, panel.y + 48.0, 12.0, primary);
    draw_ui_text(
        if narrow {
            "Choose the next scheduling deck."
        } else {
            "Choose one deck to receive the next scheduling cycle."
        },
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
    let depth_labels = if narrow {
        ["SFC", "FND", "ASM", "ORB"]
    } else {
        ["SURFACE", "FOUNDRY", "ASSEMBLY", "ORBITAL"]
    };
    for (index, (number, _, color)) in DEPTHS.iter().enumerate() {
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
        let label = depth_labels[index];
        draw_ui_text(
            label,
            x - measure_ui_text(label, None, 8, 1.0).width * 0.5,
            rail_y + 26.0,
            8.0,
            node_color,
        );
    }
    let next_label = if narrow {
        match depth {
            0 => "SMELTER",
            1 => "ASSEMBLER",
            2 => "MASS DRIVER",
            _ => "ALL DECKS ONLINE",
        }
    } else {
        state.factory_depth_progress().1
    };
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
    let core_line = truncate_text_to_width(
        &core_progress_text(state, narrow),
        panel.w - 36.0,
        if narrow { 8.0 } else { 9.0 },
    );
    draw_ui_text(
        &core_line,
        panel.x + 18.0,
        panel.y + 144.0,
        if narrow { 8.0 } else { 9.0 },
        primary,
    );

    let focus_y = panel.y + 151.0;
    let focus_w = (panel.w - 44.0 - 18.0) / 4.0;
    for (index, focus) in FactoryFocus::ALL.into_iter().enumerate() {
        let x = panel.x + 22.0 + index as f32 * (focus_w + 6.0);
        let selected = state.factory_focus == focus;
        let button_color = if selected { primary } else { dim };
        let focus_label = if narrow && panel.w < 280.0 {
            match focus {
                FactoryFocus::Balanced => "BAL",
                FactoryFocus::Extraction => "ORE",
                FactoryFocus::Refining => "ALY",
                FactoryFocus::Assembly => "PRT",
            }
        } else {
            focus.short_label()
        };
        if draw_hud_button(theme, Rect::new(x, focus_y, focus_w, 30.0), focus_label) {
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
        if narrow {
            narrow_focus_description(state.factory_focus)
        } else {
            state.factory_focus.description()
        },
        panel.x + 22.0,
        panel.y + 202.0,
        10.0,
        if state.factory_focus == FactoryFocus::Balanced {
            dim
        } else {
            warning
        },
    );
    let live_readout = if narrow {
        format!(
            "DRAW {:+.1}/s | ORE {:.1} | A {:.1} | P {:.1}",
            -state.factory_focus_power_tax(),
            state.drill_output_rate(),
            state.observed_alloy_rate(),
            state.observed_components_rate()
        )
    } else {
        format!(
            "Live draw: {:+.1}/s  |  drill {:.1}/s  |  alloy {:.1}/s  |  parts {:.1}/s",
            -state.factory_focus_power_tax(),
            state.drill_output_rate(),
            state.observed_alloy_rate(),
            state.observed_components_rate()
        )
    };
    draw_ui_text(&live_readout, panel.x + 22.0, panel.y + 223.0, 9.0, text);
    let focus_readout = state.factory_focus_readout();
    let (focus_status, focus_status_color) = focus_status_line(state.factory_focus, focus_readout);
    let focus_status = truncate_text_to_width(
        &focus_status,
        panel.w - 44.0,
        if narrow { 8.0 } else { 9.0 },
    );
    draw_ui_text(
        &focus_status,
        panel.x + 22.0,
        panel.y + 240.0,
        if narrow { 8.0 } else { 9.0 },
        if focus_status_color { warning } else { success },
    );
    if compact {
        draw_ui_text(
            "FRAME: LIVE / WAIT / FULL / OFFLINE  //  TAP DECK CLOSE",
            panel.x + 22.0,
            panel.y + 260.0,
            if narrow { 8.0 } else { 9.0 },
            dim,
        );
    } else {
        draw_ui_text(
            "FRAME: LIVE / WAIT / FULL / OFFLINE  //  focus changes rates, never map wiring.",
            panel.x + 22.0,
            panel.y + 260.0,
            9.0,
            dim,
        );
    }

    // Resource chips give the deck a visual readout that is distinct from the
    // six full-width cards in the top bar. The top bar already carries these
    // values on phones, where the control-room copy needs the full width.
    if !narrow {
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
}

fn focus_status_line(focus: FactoryFocus, readout: FactoryFocusReadout) -> (String, bool) {
    if focus == FactoryFocus::Balanced {
        return ("FOCUS BALANCED // no modifier applied".to_string(), false);
    }
    if readout.targets == 0 {
        return ("FOCUS IDLE // no target machines".to_string(), true);
    }
    if readout.producing == readout.targets {
        return (
            format!(
                "FOCUS LIVE // {} {}/{} producing",
                focus.short_label(),
                readout.producing,
                readout.targets
            ),
            false,
        );
    }
    if readout.producing > 0 {
        return (
            format!(
                "FOCUS LIMITED // {} {}/{} producing",
                focus.short_label(),
                readout.producing,
                readout.targets
            ),
            true,
        );
    }
    if readout.online > 0 {
        return (
            format!(
                "FOCUS WAIT // {} online, feed or pad is blocked",
                focus.short_label()
            ),
            true,
        );
    }
    if readout.powered > 0 {
        return (
            format!(
                "FOCUS PAUSED // {} standby or dust stalled",
                focus.short_label()
            ),
            true,
        );
    }
    (
        format!(
            "FOCUS OFFLINE // {} target machines unpowered",
            focus.short_label()
        ),
        true,
    )
}

fn factory_deck_layout(screen_w: f32, screen_h: f32, metrics: HudMetrics) -> FactoryDeckLayout {
    let narrow = screen_w < 600.0;
    let compact = screen_w < 900.0 || screen_h < 650.0;
    let panel_w = if narrow {
        (screen_w - 20.0).clamp(200.0, 390.0).min(screen_w)
    } else if compact {
        390.0
    } else {
        500.0
    };
    let panel_x = ((screen_w - panel_w) * 0.5).max(0.0);
    let panel_y = (metrics.top_bar_height + 18.0).max(18.0);
    let requested_h: f32 = if narrow {
        if screen_h < 560.0 {
            300.0
        } else {
            310.0
        }
    } else if compact {
        268.0
    } else {
        302.0
    };
    let available_h = (screen_h - panel_y - metrics.bottom_bar_height - 12.0).max(180.0);
    let panel_h = requested_h.min(available_h);
    FactoryDeckLayout {
        panel: Rect::new(panel_x, panel_y, panel_w, panel_h),
        compact,
        narrow,
    }
}

fn narrow_focus_description(focus: FactoryFocus) -> &'static str {
    match focus {
        FactoryFocus::Balanced => "All decks share output.",
        FactoryFocus::Extraction => "Drills +25%; draw +15%.",
        FactoryFocus::Refining => "Smelters +25%; draw +15%.",
        FactoryFocus::Assembly => "Servers +25%; draw +15%.",
    }
}

fn core_progress_text(state: &PlanetState, narrow: bool) -> String {
    let stages = state.core_stages();
    if stages.is_empty() {
        return "CORE // NO STAGE DATA".to_string();
    }

    let current_index = state.core_stage_index();
    let current = &stages[current_index];
    let stage_count = stages.len();
    let Some(next) = stages.get(current_index + 1) else {
        return if narrow {
            format!("CORE {stage_count}/{stage_count} // FULLY EVOLVED")
        } else {
            format!(
                "CORE {stage_count:02}/{stage_count:02}  {} // FULLY EVOLVED",
                current.name.to_uppercase()
            )
        };
    };

    let requirements = next
        .requires
        .iter()
        .filter_map(|requirement| {
            let milestone = Milestone::from_id(&requirement.kind)?;
            Some(format!(
                "{} {}/{}",
                core_requirement_label(milestone, narrow),
                compact_requirement_amount(state.measure(milestone)),
                compact_requirement_amount(requirement.target)
            ))
        })
        .collect::<Vec<_>>()
        .join(" + ");
    let requirements = if requirements.is_empty() {
        "AWAITING SIGNAL".to_string()
    } else {
        requirements
    };

    if narrow {
        format!(
            "CORE {}/{} -> {} | {}",
            current_index + 1,
            stage_count,
            next.name.to_uppercase(),
            requirements
        )
    } else {
        format!(
            "CORE {:02}/{:02}  {} // NEXT {}: {}",
            current_index + 1,
            stage_count,
            current.name.to_uppercase(),
            next.name.to_uppercase(),
            requirements
        )
    }
}

fn core_requirement_label(milestone: Milestone, narrow: bool) -> &'static str {
    if narrow {
        match milestone {
            Milestone::Structures => "STR",
            Milestone::NetworkTiles => "NET",
            Milestone::Technologies => "TECH",
            Milestone::SeedShipStages => "SHIP",
            _ => milestone.id(),
        }
    } else {
        match milestone {
            Milestone::Structures => "STRUCTURES",
            Milestone::NetworkTiles => "NETWORK",
            Milestone::Technologies => "TECHNOLOGIES",
            Milestone::SeedShipStages => "SHIP STAGES",
            _ => milestone.id(),
        }
    }
}

fn compact_requirement_amount(amount: f32) -> String {
    if amount.fract().abs() > 0.05 {
        format!("{amount:.1}")
    } else {
        format!("{amount:.0}")
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

fn draw_focus_targets(state: &PlanetState, metrics: HudMetrics, theme: &UiTheme, time: f32) {
    if state.factory_focus == FactoryFocus::Balanced {
        return;
    }

    let starved: HashSet<_> = state.starved_factories().into_iter().collect();
    let blocked: HashSet<_> = state.blocked_factories().into_iter().collect();
    let pulse = 0.35 + (time * 2.6).sin().abs() * 0.18;
    for position in focus_target_positions(state) {
        let Some(building) = state
            .grid
            .get(position)
            .and_then(|tile| tile.building.as_ref())
        else {
            continue;
        };
        let color = focus_target_color(state, position, building, &starved, &blocked, theme);
        let (x, y) = super::metrics::grid_to_screen(position, metrics);
        let inset = (metrics.tile_size * 0.1).clamp(2.0, 4.0);
        let size = metrics.tile_size - inset * 2.0;
        draw_rectangle_lines(
            x + inset,
            y + inset,
            size,
            size,
            1.2,
            with_alpha(color, pulse),
        );
        let beacon = 2.0 + (time * 3.0 + position.x as f32 * 0.17).sin().abs();
        draw_circle(x + metrics.tile_size * 0.5, y + inset, beacon, color);
    }
}

fn focus_target_color(
    state: &PlanetState,
    position: GridPos,
    building: &crate::engine::Building,
    starved: &HashSet<GridPos>,
    blocked: &HashSet<GridPos>,
    theme: &UiTheme,
) -> Color {
    if !building.powered {
        color_from_rgba(&theme.colors.text_dim)
    } else if building.standby || building.is_dust_stalled_with(&state.resolved_dust_response()) {
        color_from_rgba(&theme.colors.warning)
    } else if blocked.contains(&position) {
        color_from_rgba(&theme.colors.error)
    } else if starved.contains(&position) {
        color_from_rgba(&theme.colors.warning)
    } else {
        color_from_rgba(&theme.colors.primary)
    }
}

fn focus_target_positions(state: &PlanetState) -> Vec<GridPos> {
    if state.factory_focus == FactoryFocus::Balanced {
        return Vec::new();
    }
    state
        .grid
        .iter_tiles()
        .filter_map(|(position, tile)| {
            tile.building
                .as_ref()
                .filter(|building| state.factory_focus.affects(building.building_type))
                .map(|_| position)
        })
        .collect()
}

fn resource_chip_color(resource: crate::engine::ResourceType, theme: &UiTheme) -> Color {
    match resource {
        crate::engine::ResourceType::Minerals => color_from_rgba(&theme.colors.minerals),
        crate::engine::ResourceType::Alloy => color_from_rgba(&theme.colors.alloy),
        crate::engine::ResourceType::Components => color_from_rgba(&theme.colors.components),
        _ => color_from_rgba(&theme.colors.text),
    }
}

#[cfg(test)]
mod tests;
