//! Fixed-step clock and speed-control coverage.

use super::super::super::MAX_CATCHUP_TICKS;
use super::state;
use crate::state::{TICK_SECONDS, TIME_SCALES};

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
