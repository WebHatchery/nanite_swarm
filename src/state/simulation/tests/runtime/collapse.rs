//! Power-collapse lifecycle coverage.

use super::state;
use crate::data::GameConfig;
use crate::engine::{BuildingType, DroneState, GridPos};

#[test]
fn power_collapse_triggers_after_sustained_negative_power() {
    let mut state = state();
    // Force a persistent deficit and drive the simulation past the 60s threshold.
    state.resources.energy = 1_000_000.0;
    state.config.resources.max_energy = 1_000_000.0;
    state.resources.minerals = 1_000_000.0;
    // A Server Bank placed adjacent to the Core is powered directly (Core
    // transmits power to neighbors) and consumes more than the Core generates.
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::ServerBank);
    state.select_building(BuildingType::ServerBank);
    assert!(state.try_place_building(pos));

    assert!(state.net_power() < 0.0);

    // Just past the trigger: the shutdown is short for a base this small,
    // so running well past it would only prove it had already lifted.
    for _ in 0..61 {
        state.step(1.0, false);
    }

    assert!(state.power_collapse_cooldown > 0.0);
    assert!(state.power_collapse_shutdown > 0.0);
}

#[test]
fn trigger_power_collapse_drops_drone_cargo_and_corrupts_progress() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);
    state.drones.drones_mut()[0].carrying = 5.0;
    state.resources.data = 100.0;
    state.research.research_progress = 100.0;
    state
        .research
        .parked_progress
        .insert("paused_research".to_string(), 100.0);

    state.trigger_power_collapse();

    assert_eq!(state.drones.drones()[0].carrying, 0.0);
    assert_eq!(state.drones.drones()[0].state, DroneState::Error);
    // How much is lost scales with the size of the base, so this pins the
    // ends rather than one number that moves with the balance.
    let collapse = &GameConfig::default().collapse;
    let kept = state.resources.data;
    assert!(
        kept < 100.0 * (1.0 - collapse.min_data_loss) + 0.001
            && kept > 100.0 * (1.0 - collapse.max_data_loss),
        "a two-building base kept {} of 100 Data",
        kept
    );
    assert_eq!(state.research.research_progress, kept);
    assert_eq!(
        state.research.parked_progress.get("paused_research"),
        Some(&kept)
    );
    assert_eq!(state.power_collapse_cooldown, collapse.cooldown_seconds);
}

#[test]
fn research_reduces_collapse_shutdown_and_data_loss_deterministically() {
    let mut bare = crate::state::PlanetState::new(3, 7, GameConfig::default());
    let mut improved = crate::state::PlanetState::new(3, 7, GameConfig::default());
    improved
        .research
        .unlocked_techs
        .extend(["thermal_sinks".to_string(), "advanced_research".to_string()]);
    improved.refresh_stats();

    bare.resources.data = 100.0;
    improved.resources.data = 100.0;
    bare.trigger_power_collapse();
    improved.trigger_power_collapse();
    assert!(improved.power_collapse_shutdown < bare.power_collapse_shutdown);
    assert!(improved.resources.data > bare.resources.data);
}
