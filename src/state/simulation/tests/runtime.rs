//! Upkeep, clock, collapse, and tutorial simulation checks.

use super::*;

#[test]
fn the_dust_rate_is_data_rather_than_a_constant() {
    let mut slow = GameConfig::default();
    slow.upkeep.dust_rate = 0.05;
    let mut fast = GameConfig::default();
    fast.upkeep.dust_rate = 0.5;

    let settled = |config: GameConfig| {
        let mut state = PlanetState::new(2, 42, config);
        let core = state.grid.find_core().unwrap();
        for _ in 0..300 {
            state.step(TICK_SECONDS, false);
        }
        state
            .grid
            .get(core)
            .and_then(|tile| tile.building.as_ref())
            .map(|building| building.dust)
            .unwrap_or(0.0)
    };

    assert!(
        settled(fast) > settled(slow),
        "a bigger declared dust rate should settle more dust"
    );
}

#[test]
fn a_sweepers_reach_is_the_one_the_config_declares() {
    let mut config = GameConfig::default();
    config.upkeep.sweeper_radius = 9;
    let state = PlanetState::new(2, 42, config);
    assert_eq!(state.coverage_radius(BuildingType::Sweeper), Some(9));
    assert_eq!(
        state.coverage_radius(BuildingType::ShieldGenerator),
        Some(state.config.upkeep.hazard_counter_radius)
    );
}

#[test]
fn dust_accumulates_on_powered_buildings_over_time() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    state.update_dust(10.0);
    let dust = state.grid.get(pos).unwrap().building.as_ref().unwrap().dust;
    assert!(dust > 0.0);
}

/// Dust on a drill with an optional intact forest placed at a known distance.
fn drill_dust_with_forest_at(distance: Option<i32>) -> f32 {
    let mut state = state();
    state.config.upkeep.forest_buffer_radius = 2;
    state.config.upkeep.forest_buffer_multiplier = 0.5;
    // The default temperate world has forests of its own. Clear the fixture
    // first so the comparison names exactly one preserved tile.
    for (_, tile) in state.grid.iter_tiles_mut() {
        if tile.terrain == TerrainType::Forest {
            tile.terrain = TerrainType::Empty;
            tile.biomass_amount = 0.0;
        }
    }

    let core = state.grid.find_core().unwrap();
    let drill = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(drill, 4);
    state.grid.get_mut(drill).unwrap().terrain = TerrainType::Empty;
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));

    if let Some(distance) = distance {
        let forest = GridPos::new(drill.x, drill.y + distance);
        let tile = state.grid.get_mut(forest).unwrap();
        tile.terrain = TerrainType::Forest;
        tile.filter = false;
        tile.forest_cleared = false;
    }

    state.update_dust(10.0);
    state
        .grid
        .get(drill)
        .and_then(|tile| tile.building.as_ref())
        .map(|building| building.dust)
        .unwrap_or(0.0)
}

#[test]
fn an_intact_forest_buffers_nearby_dust() {
    let bare = drill_dust_with_forest_at(None);
    let buffered = drill_dust_with_forest_at(Some(1));

    assert!(
        buffered < bare,
        "nearby forest left {buffered} dust versus bare ground at {bare}"
    );
}

#[test]
fn a_preserved_forest_stops_buffering_at_its_declared_radius() {
    let bare = drill_dust_with_forest_at(None);
    let outside = drill_dust_with_forest_at(Some(3));

    assert!(
        (outside - bare).abs() < 0.001,
        "forest outside the radius changed dust from {bare} to {outside}"
    );
}

#[test]
fn advance_runs_whole_ticks_and_banks_the_remainder() {
    let mut state = state();
    assert_eq!(state.advance(TICK_SECONDS * 2.5, false), 2);
    assert!((state.sim_accumulator - TICK_SECONDS * 0.5).abs() < 1e-4);
    assert!((state.time_played - (TICK_SECONDS * 2.0) as f64).abs() < 1e-4);

    // The banked half tick means the next one needs only half a tick more.
    assert_eq!(state.advance(TICK_SECONDS * 0.6, false), 1);
}

#[test]
fn a_paused_world_does_not_move() {
    let mut state = state();
    state.toggle_pause();
    assert!(state.paused);

    assert_eq!(state.advance(10.0, false), 0);
    assert_eq!(state.time_played, 0.0);
    assert_eq!(state.sim_accumulator, 0.0);

    // And starts again exactly where it stopped.
    state.toggle_pause();
    assert!(state.advance(TICK_SECONDS, false) > 0);
    assert!(state.time_played > 0.0);
}

#[test]
fn speed_buys_more_world_time_for_the_same_real_time() {
    let mut normal = state();
    let mut fast = state();
    fast.change_speed(true);
    assert_eq!(fast.time_scale, 2.0);

    // One second of wall clock each, in frames.
    for _ in 0..60 {
        normal.advance(1.0 / 60.0, false);
        fast.advance(1.0 / 60.0, false);
    }

    let ratio = fast.time_played / normal.time_played;
    assert!(
        (ratio - 2.0).abs() < 0.1,
        "double speed simulated {ratio} times as much"
    );
}

#[test]
fn the_speed_ladder_stops_at_both_ends() {
    let mut state = state();
    for _ in 0..10 {
        state.change_speed(false);
    }
    assert_eq!(state.time_scale, TIME_SCALES[0]);
    for _ in 0..10 {
        state.change_speed(true);
    }
    assert_eq!(state.time_scale, TIME_SCALES[TIME_SCALES.len() - 1]);
}

#[test]
fn the_fastest_speed_is_not_capped_away_by_the_catch_up_limit() {
    let mut state = state();
    for _ in 0..3 {
        state.change_speed(true);
    }
    assert_eq!(state.time_scale, 8.0);

    // A slow frame at top speed still buys everything it should.
    let ticks = state.advance(1.0 / 30.0, false);
    let expected = (8.0 / 30.0 / TICK_SECONDS).floor() as u32;
    assert_eq!(ticks, expected, "fast-forward lost time to the tick cap");
}

#[test]
fn advance_ignores_a_zero_or_nonsense_delta() {
    let mut state = state();
    assert_eq!(state.advance(0.0, false), 0);
    assert_eq!(state.advance(-1.0, false), 0);
    assert_eq!(state.advance(f32::NAN, false), 0);
    assert_eq!(state.time_played, 0.0);
}

#[test]
fn advance_drops_a_long_backlog_instead_of_replaying_it() {
    let mut state = state();
    // Ten seconds of stall: run the cap, then start clean.
    assert_eq!(state.advance(10.0, false), MAX_CATCHUP_TICKS);
    assert_eq!(state.sim_accumulator, 0.0);
}

#[test]
fn frame_rate_does_not_change_how_much_world_time_passes() {
    let mut fast = state();
    let mut slow = state();

    // Two seconds of wall clock, at 120fps and at 40fps.
    let mut fast_ticks = 0;
    for _ in 0..240 {
        fast_ticks += fast.advance(1.0 / 120.0, false);
    }
    let mut slow_ticks = 0;
    for _ in 0..80 {
        slow_ticks += slow.advance(1.0 / 40.0, false);
    }

    assert!(
        (fast_ticks as i64 - slow_ticks as i64).abs() <= 1,
        "{fast_ticks} vs {slow_ticks} ticks for the same wall clock"
    );
    assert!((fast.time_played - slow.time_played).abs() <= TICK_SECONDS as f64 * 1.5);
    assert!((fast.resources.energy - slow.resources.energy).abs() <= 1.0);
}

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
    let mut bare = PlanetState::new(3, 7, GameConfig::default());
    let mut improved = PlanetState::new(3, 7, GameConfig::default());
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

#[test]
fn tutorial_advances_when_first_drill_is_placed() {
    let mut state = state();
    assert_eq!(state.tutorial_step, 0);
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    state.update_tutorial();
    assert_eq!(state.tutorial_step, 1);
}
