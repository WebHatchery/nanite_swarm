//! Core progression labels shown in the structure inspector.

use crate::state::{Milestone, PlanetState};

pub(super) fn core_stage_status_text(state: &PlanetState) -> String {
    let stages = state.core_stages();
    if stages.is_empty() {
        return "CORE // NO STAGE DATA".to_string();
    }

    let current_index = state.core_stage_index();
    let stage_count = stages.len();
    if stages.get(current_index + 1).is_none() {
        return format!("CORE {stage_count}/{stage_count} // FULLY EVOLVED");
    }

    let requirements = state
        .core_requirement_progress()
        .iter()
        .map(|progress| {
            format!(
                "{}{}/{}",
                compact_milestone_label(progress.milestone),
                compact_requirement_amount(progress.current),
                compact_requirement_amount(progress.target)
            )
        })
        .collect::<Vec<_>>()
        .join("+");
    if requirements.is_empty() {
        format!("CORE {}/{stage_count} // READY", current_index + 1)
    } else {
        format!("CORE {}/{stage_count} / {requirements}", current_index + 1)
    }
}

fn compact_milestone_label(milestone: Milestone) -> &'static str {
    match milestone {
        Milestone::Structures => "STR",
        Milestone::NetworkTiles => "NET",
        Milestone::Technologies => "TECH",
        Milestone::SeedShipStages => "SHIP",
        _ => milestone.id(),
    }
}

fn compact_requirement_amount(amount: f32) -> String {
    if amount.fract().abs() > 0.05 {
        format!("{amount:.1}")
    } else {
        format!("{amount:.0}")
    }
}

pub(super) fn core_next_stage_label(state: &PlanetState) -> String {
    state
        .core_stages()
        .get(state.core_stage_index() + 1)
        .map(|stage| stage.name.clone())
        .unwrap_or_else(|| "FULLY EVOLVED".to_string())
}
