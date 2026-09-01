#![allow(dead_code)]

use crate::state::{PlanetState, UpkeepAlertCause};

use super::ClockAction;

pub(super) fn next_event_code(label: &str) -> &'static str {
    match label {
        "Power recovery" => "PWR",
        "Collapse recovery" => "COOL",
        "Research unlock" => "DATA",
        "Mass Driver ready" => "DRVR",
        _ => "NEXT",
    }
}

pub(super) fn has_pending_interaction(state: &PlanetState) -> bool {
    state.selected_building.is_some()
        || state.demolish_mode
        || state.relocation_source.is_some()
        || state.box_select_mode
        || state.purge_armed.is_some()
        || state.bulk_purge_armed
        || state.drag_last_pos.is_some()
        || state.smart_drag_source.is_some()
}

pub(super) fn alert_button(
    save_failed: bool,
    restored_from_backup: bool,
    flow_alert: bool,
    flow_overlay: bool,
    focusable_alert: bool,
) -> Option<(&'static str, ClockAction)> {
    if save_failed {
        Some(("RETRY", ClockAction::RetrySave))
    } else if restored_from_backup {
        Some(("SAVE NOW", ClockAction::RetrySave))
    } else if flow_alert {
        if focusable_alert {
            Some(("JUMP", ClockAction::FocusAlert))
        } else {
            Some((
                if flow_overlay { "FLOW ON" } else { "FLOW" },
                ClockAction::ToggleFlow,
            ))
        }
    } else {
        None
    }
}

/// Name the same recovery the JUMP button will visit first.
///
/// Keeping this priority beside the button policy prevents the banner from
/// promising an upkeep repair while JUMP opens a more urgent production
/// blocker instead.
pub(super) fn alert_text(
    state: &PlanetState,
    stalled: usize,
    starved: usize,
    blocked: usize,
    upkeep: usize,
    congested: usize,
) -> &'static str {
    if state.save_failed {
        "SAVE FAILED"
    } else if state.restored_from_backup {
        "BACKUP RECOVERED"
    } else if state.power_collapse_shutdown > 0.0 {
        "POWER COLLAPSE"
    } else if state.battery_seconds <= 0.0 {
        "LOW BATTERY"
    } else if state.power_balance < 0.0 {
        "NEGATIVE POWER"
    } else if stalled > 0 {
        "ROUTE SEVERED"
    } else if starved > 0 {
        "INPUT STARVED"
    } else if blocked > 0 {
        "OUTPUT PAD FULL"
    } else if upkeep > 0 {
        upkeep_alert_text(state)
    } else if congested > 0 {
        "TRAFFIC SATURATED"
    } else {
        "SYSTEM NOMINAL"
    }
}

pub(super) fn upkeep_alert_text(state: &PlanetState) -> &'static str {
    match state.upkeep_alert_cause() {
        Some(UpkeepAlertCause::Dust) => "UPKEEP DUST",
        Some(UpkeepAlertCause::Acid) => "UPKEEP ACID",
        Some(UpkeepAlertCause::Mixed) => "UPKEEP MIXED",
        None => "UPKEEP DEGRADED",
    }
}

pub(super) fn has_flow_alert(state: &PlanetState) -> bool {
    state.power_collapse_shutdown > 0.0
        || state.power_balance < 0.0
        || state.battery_seconds <= 0.0
        || state.congested_tiles() > 0
        || !state.starved_factories().is_empty()
        || !state.blocked_factories().is_empty()
        || state.stalled_drone_count() > 0
        || !state.upkeep_alert_positions().is_empty()
}
