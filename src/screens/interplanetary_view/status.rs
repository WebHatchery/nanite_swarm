//! Header status copy for interplanetary launch readiness.

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
