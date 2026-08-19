use super::*;
use crate::data::GameConfig;
use crate::engine::{BuildingType, GridPos};

fn state() -> PlanetState {
    PlanetState::new(2, 42, GameConfig::default())
}

#[test]
fn every_shipped_condition_is_one_the_game_can_measure() {
    for def in &crate::data::game_data().achievements {
        assert!(
            Milestone::from_id(&def.condition.kind).is_some(),
            "{} asks for \"{}\"",
            def.id,
            def.condition.kind
        );
        assert!(!def.name.is_empty());
        assert!(!def.description.is_empty());
    }
}

#[test]
fn a_world_that_has_done_nothing_has_earned_almost_nothing() {
    let mut state = state();
    state.power_balance = -1.0;
    state.update_achievements();
    let (unlocked, total) = state.achievements_progress();
    assert_eq!(unlocked, 0, "something fired on an untouched world");
    assert!(total >= 10, "the shipped set is still tiny: {}", total);
}

#[test]
fn the_starting_core_does_not_earn_power_surplus() {
    let mut state = state();
    assert_eq!(state.power_balance, 4.0);
    state.update_achievements();
    assert!(!state.achievements.is_unlocked("power_surplus"));

    state.power_balance = 5.0;
    state.update_achievements();
    assert!(state.achievements.is_unlocked("power_surplus"));
}

#[test]
fn holding_the_ore_a_declared_achievement_asks_for_earns_it() {
    let mut state = state();
    state.config.resources.base_mineral_cap = 100_000.0;
    state.resources.minerals = 500.0;

    state.update_achievements();

    assert!(state.achievements.is_unlocked("stockpile"));
    // And one it has not reached stays locked.
    assert!(!state.achievements.is_unlocked("refinery"));
}

#[test]
fn precision_parts_reward_both_stockpiling_and_a_stable_line() {
    let mut state = state();
    state.resources.components = 25.0;
    state.graph_samples.push(crate::state::GraphSample {
        components_produced: 0.5,
        ..crate::state::GraphSample::default()
    });

    state.update_achievements();

    assert!(state.achievements.is_unlocked("first_principles"));
    assert!(state.achievements.is_unlocked("unbroken_chain"));
}

#[test]
fn scaled_processor_modes_have_records_payoffs() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    for (step, kind) in [
        BuildingType::Smelter,
        BuildingType::Smelter,
        BuildingType::Assembler,
    ]
    .into_iter()
    .enumerate()
    {
        let pos = GridPos::new(core.x + step as i32 + 1, core.y);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        assert!(state.grid.place_building(pos, kind));
        let building = state.grid.get_mut(pos).unwrap().building.as_mut().unwrap();
        building.overclocked = true;
        building.input_priority = true;
        building.standby = true;
        state.output_buffers.insert((pos.x, pos.y), 20.0);
    }

    state.update_achievements();

    assert!(state.achievements.is_unlocked("redline_cluster"));
    assert!(state.achievements.is_unlocked("freight_yard"));
    assert!(state.achievements.is_unlocked("command_lattice"));
    assert!(state.achievements.is_unlocked("dark_shift"));
}

#[test]
fn laying_network_is_counted_by_the_tiles_that_carry_drones() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 24);
    for step in 1..=20 {
        let pos = GridPos::new(core.x + step % 10, core.y + 1 + step / 10);
        if let Some(tile) = state.grid.get_mut(pos) {
            tile.terrain = crate::engine::TerrainType::Empty;
        }
        state.grid.place_building(pos, BuildingType::Conduit);
    }

    state.update_achievements();

    assert!(state.achievements.is_unlocked("network"));
}

#[test]
fn a_manual_achievement_never_fires_by_itself() {
    let mut state = state();
    // Everything a measured condition could want, several times over.
    state.config.resources.base_mineral_cap = 100_000.0;
    state.resources.minerals = 100_000.0;
    state.resources.data = 100_000.0;
    state.resources.alloy = 100_000.0;
    state.power_balance = 100.0;

    state.update_achievements();

    assert!(!state.achievements.is_unlocked("seed_ship"));
    assert!(!state.achievements.is_unlocked("system_consumed"));
    // It still fires when the code that owns it says so.
    state.announce_achievement("seed_ship");
    assert!(state.achievements.is_unlocked("seed_ship"));
}

#[test]
fn what_the_swarm_is_told_outlives_the_toast_that_said_it() {
    let mut state = state();
    state.config.resources.base_mineral_cap = 100_000.0;
    state.resources.minerals = 500.0;
    state.update_achievements();

    // Long enough that nothing is left on screen.
    state.notifications.update(60.0);
    assert!(state.notifications.get_notifications().is_empty());
    assert!(
        state
            .notifications
            .history()
            .iter()
            .any(|entry| entry.message.contains("Stockpile")),
        "the announcement was the only copy and it is gone"
    );
}

fn record(state: &PlanetState, name: &str) -> AchievementRecord {
    state
        .achievement_records()
        .into_iter()
        .find(|record| record.name == name)
        .expect("the shipped set still has this one")
}

#[test]
fn the_records_cover_the_declared_set_in_declaration_order() {
    let records = state().achievement_records();
    let declared = &crate::data::game_data().achievements;
    assert_eq!(records.len(), declared.len());
    for (record, def) in records.iter().zip(declared) {
        assert_eq!(record.name, def.name);
        assert!(!record.unlocked, "nothing is earned on an untouched world");
    }
}

#[test]
fn a_locked_record_says_how_far_along_it_is() {
    let mut state = state();
    state.config.resources.base_mineral_cap = 100_000.0;
    state.resources.minerals = 250.0;

    let stockpile = record(&state, "Stockpile");
    assert!(!stockpile.unlocked);
    assert!(stockpile.countable, "500 minerals is worth counting");
    assert_eq!(stockpile.progress, Some(250.0));
    assert_eq!(stockpile.target, 500.0);
    assert!((stockpile.fraction() - 0.5).abs() < 0.001);
}

#[test]
fn an_earned_record_stays_full_however_the_world_moves_on() {
    let mut state = state();
    state.config.resources.base_mineral_cap = 100_000.0;
    state.resources.minerals = 500.0;
    state.update_achievements();
    // Spent again, right back down to nothing.
    state.resources.minerals = 0.0;

    let stockpile = record(&state, "Stockpile");
    assert!(stockpile.unlocked);
    assert_eq!(stockpile.fraction(), 1.0);
}

#[test]
fn a_record_with_nothing_to_count_does_not_pretend_otherwise() {
    let state = state();
    // Announced by code, so there is no running total behind it.
    let manual = record(&state, "Seed Ship");
    assert!(!manual.countable);
    assert_eq!(manual.fraction(), 0.0);
    // The declared five-power threshold has useful progress.
    let surplus = record(&state, "Power Surplus");
    assert!(surplus.countable);
    assert_eq!(surplus.progress, Some(4.0));
}
