//! Factory-deck status, progression, and resource readouts.

use crate::data::UiTheme;
use crate::engine::ResourceType;
use crate::state::{FactoryFocus, FactoryFocusReadout, Milestone, PlanetState};
use crate::ui::color_from_rgba;
use macroquad::prelude::Color;

pub(super) fn focus_status_line(
    focus: FactoryFocus,
    readout: FactoryFocusReadout,
) -> (String, bool) {
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

pub(super) fn narrow_focus_description(focus: FactoryFocus) -> &'static str {
    match focus {
        FactoryFocus::Balanced => "All decks share output.",
        FactoryFocus::Extraction => "Drills +25%; draw +15%.",
        FactoryFocus::Refining => "Smelters +25%; draw +15%.",
        FactoryFocus::Assembly => "Servers +25%; draw +15%.",
    }
}

pub(super) fn core_progress_text(state: &PlanetState, narrow: bool) -> String {
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

pub(super) fn resource_chip_color(resource: ResourceType, theme: &UiTheme) -> Color {
    match resource {
        ResourceType::Minerals => color_from_rgba(&theme.colors.minerals),
        ResourceType::Alloy => color_from_rgba(&theme.colors.alloy),
        ResourceType::Components => color_from_rgba(&theme.colors.components),
        _ => color_from_rgba(&theme.colors.text),
    }
}
