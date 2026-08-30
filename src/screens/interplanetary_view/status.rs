//! Header status copy for interplanetary launch readiness.

use super::InterplanetaryAction;

pub(super) fn driver_operation_status(built: usize, online: usize, world_name: &str) -> String {
    if online > 0 {
        format!(
            "{} driver{} online on {}",
            online,
            if online == 1 { "" } else { "s" },
            world_name
        )
    } else if built > 0 {
        format!("Mass Driver offline on {}", world_name)
    } else {
        format!("No Mass Driver on {world_name}")
    }
}

pub(super) fn landing_pad_operation_status(
    built: usize,
    online: usize,
    world_name: &str,
) -> String {
    if online > 0 {
        format!(
            "{} landing pad{} online on {}",
            online,
            if online == 1 { "" } else { "s" },
            world_name
        )
    } else if built > 0 {
        format!("Landing Pad offline on {}", world_name)
    } else {
        format!("No Landing Pad on {}", world_name)
    }
}

pub(super) fn launch_action(index: usize, confirming: bool) -> InterplanetaryAction {
    if confirming {
        InterplanetaryAction::LaunchSeedShip(index)
    } else {
        InterplanetaryAction::RequestLaunch(index)
    }
}

pub(super) fn launch_prompt(confirming: bool) -> &'static str {
    if confirming {
        "Tap CONFIRM row to launch Seed Ship"
    } else {
        "Tap world row to prepare launch"
    }
}

pub(super) fn mass_driver_status_label(
    researched: bool,
    built: usize,
    online: usize,
) -> &'static str {
    if online > 0 {
        "Mass Driver: ONLINE"
    } else if !researched {
        "Mass Driver: LOCKED"
    } else if built > 0 {
        "Mass Driver: OFFLINE"
    } else {
        "Mass Driver: NOT BUILT"
    }
}

pub(super) fn seed_ship_status_label(
    ready: bool,
    drivers_built: usize,
    drivers_online: usize,
) -> &'static str {
    if !ready {
        "Seed Ship: UNDER CONSTRUCTION"
    } else if drivers_online > 0 {
        "Seed Ship: READY"
    } else if drivers_built > 0 {
        "Seed Ship: READY / DRIVER OFFLINE"
    } else {
        "Seed Ship: READY / DRIVER NEEDED"
    }
}
