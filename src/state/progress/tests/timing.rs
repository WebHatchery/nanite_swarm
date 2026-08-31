use super::*;

#[test]
fn battery_time_left_converts_seconds_to_hours_and_minutes() {
    let state = PlanetState {
        battery_seconds: 3661.0, // 1h 1m 1s
        ..Default::default()
    };
    assert_eq!(state.battery_time_left(), (1, 1));
}

#[test]
fn next_event_names_the_timer_that_will_be_reached_first() {
    let mut state = state();
    state.power_collapse_shutdown = 12.0;
    state.research_lock_timer = 8.0;
    state.export_cooldown = 20.0;

    let event = state.next_interesting_event().expect("an event is pending");
    assert_eq!(event.label, "Research unlock");
    assert_eq!(event.seconds, 8.0);
}

#[test]
fn fast_forward_to_next_event_stops_at_the_selected_timer() {
    let mut state = state();
    state.export_cooldown = 2.5;
    let before = state.time_played;

    assert!(state.fast_forward_to_next_event());
    assert!((state.time_played - before - 2.5).abs() < 0.001);
    assert!(state.export_cooldown < 0.001);
    assert!(state.next_interesting_event().is_none());
}

#[test]
fn research_modifies_the_declared_dust_response_thresholds() {
    let mut bare = state();
    let mut researched = state();
    researched
        .research
        .unlocked_techs
        .push("self_cleaning_servos".to_string());
    researched.refresh_stats();

    let base = bare.resolved_dust_response();
    let improved = researched.resolved_dust_response();
    assert!(improved.efficiency_threshold > base.efficiency_threshold);
    assert!(improved.speed_threshold > base.speed_threshold);
    assert!(researched
        .stat_sources(crate::engine::StatId::DustSpeedThreshold)
        .iter()
        .any(|source| source == "Self-Cleaning Servos"));
    bare.config.upkeep.dust_response.efficiency_threshold = 80.0;
    assert!(bare.resolved_dust_response().efficiency_threshold > 0.0);
}

#[test]
fn an_achievement_announces_itself_once_and_not_again() {
    let mut state = PlanetState::default();
    assert!(state.notifications.is_empty());

    state.announce_achievement("first_drill");
    assert_eq!(state.notifications.count(), 1);
    let message = state.notifications.get_notifications()[0].message.clone();
    assert!(message.contains("First Drill"), "{message}");

    // Ticking on does not re-announce something already unlocked.
    state.announce_achievement("first_drill");
    assert_eq!(state.notifications.count(), 1);
}

#[test]
fn toasts_fade_in_real_time_even_while_the_world_is_paused() {
    let mut state = PlanetState::default();
    state.announce_achievement("first_drill");
    // `opacity` only moves in the toast's last second, so the timer itself
    // is the thing to watch.
    let before = state.notifications.get_notifications()[0].progress();

    state.toggle_pause();
    for _ in 0..20 {
        assert_eq!(state.advance(0.1, false), 0, "the world moved while paused");
    }

    let after = state.notifications.get_notifications()[0].progress();
    assert!(
        after > before,
        "the toast froze with the world: {before} then {after}"
    );
}

#[test]
fn achievements_progress_unlocks_first_drill_and_power_surplus() {
    let mut state = PlanetState::default();
    let (unlocked_before, total) = state.achievements_progress();
    assert_eq!(unlocked_before, 0);

    let core = state.grid.find_core().unwrap();
    let pos = crate::engine::GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(crate::engine::BuildingType::Drill);
    state.try_place_building(pos);

    let (unlocked_after, total_after) = state.achievements_progress();
    assert!(unlocked_after > unlocked_before);
    assert_eq!(total_after, total);
}
