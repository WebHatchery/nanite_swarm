use super::PlanetState;
use crate::data::GameConfig;
use crate::engine::BuildingType;

fn state() -> PlanetState {
    PlanetState::new(2, 7, GameConfig::default())
}

fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() <= eps
}

#[test]
fn offline_simulation_uses_hibernation_rate() {
    let mut state = PlanetState {
        battery_seconds: 4.0 * 60.0 * 60.0,
        ..Default::default()
    };

    let offline = 6.0 * 60.0 * 60.0;
    state.apply_offline_progress(offline);

    let expected_sim = (4.0 * 60.0 * 60.0) + (2.0 * 60.0 * 60.0) * 0.1;
    assert!(approx_eq(state.last_offline_simulated, expected_sim, 0.5));
    assert!(approx_eq(state.last_offline_seconds, offline, 0.5));
    assert!(state.battery_seconds <= 0.0);
    assert!(state.offline_notice_timer > 0.0);
}

#[test]
fn apply_offline_progress_is_noop_for_zero_or_negative_duration() {
    let mut state = PlanetState::default();
    state.apply_offline_progress(0.0);
    assert_eq!(state.last_offline_seconds, 0.0);
    assert_eq!(state.last_offline_simulated, 0.0);
}

#[test]
fn core_is_always_unlocked_others_require_explicit_unlock() {
    let mut state = PlanetState::default();
    assert!(state.is_building_unlocked(BuildingType::Core));
    assert!(!state.is_building_unlocked(BuildingType::Conduit));
    state.unlock_building(BuildingType::Conduit);
    assert!(state.is_building_unlocked(BuildingType::Conduit));
}

#[test]
fn unlock_building_does_not_duplicate_entries() {
    let mut state = PlanetState::default();
    state.unlock_building(BuildingType::Storage);
    state.unlock_building(BuildingType::Storage);
    assert_eq!(
        state
            .unlocked_buildings
            .iter()
            .filter(|b| **b == BuildingType::Storage)
            .count(),
        1
    );
}

/// Every tech that claims a number in `research.json` has to move one here.
/// These are the five that shipped as no-ops.
#[test]
fn researching_storage_optimization_raises_the_mineral_cap() {
    let mut state = PlanetState::default();
    let before = state.mineral_capacity();
    state
        .research
        .unlocked_techs
        .push("storage_optimization".into());
    state.refresh_stats();
    assert_eq!(state.mineral_capacity(), before + 50.0);
}

#[test]
fn researching_power_efficiency_lowers_consumption() {
    let mut state = PlanetState::default();
    let core = state.grid.find_core().unwrap();
    let pos = crate::engine::GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    let before = state.power_consumption();
    assert!(before > 0.0);
    state
        .research
        .unlocked_techs
        .push("power_efficiency".into());
    state.refresh_stats();
    assert!(approx_eq(state.power_consumption(), before * 0.75, 1e-4));
}

#[test]
fn researching_drone_capacity_grows_existing_drones_too() {
    let mut state = PlanetState::default();
    let core = state.grid.find_core().unwrap();
    let pos = crate::engine::GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);
    let before = state.drones.drone_capacity;

    state.research.unlocked_techs.push("drone_capacity".into());
    state.refresh_stats();

    assert_eq!(state.drones.drone_capacity, before * 2.0);
    assert_eq!(state.drones.drones()[0].capacity, before * 2.0);
}

#[test]
fn researching_efficient_drills_and_advanced_research_speeds_production() {
    let mut plain = PlanetState::default();
    let mut upgraded = PlanetState::default();
    upgraded
        .research
        .unlocked_techs
        .extend(["efficient_drills".into(), "advanced_research".into()]);
    upgraded.refresh_stats();

    for state in [&mut plain, &mut upgraded] {
        let core = state.grid.find_core().unwrap();
        let pos = crate::engine::GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);
        state.resources.minerals = 0.0;
        // Both runs would otherwise clamp at the storage cap.
        state.config.resources.base_mineral_cap = 100_000.0;
        for _ in 0..300 {
            state.step(0.1, false);
        }
    }

    assert!(upgraded.resources.minerals > plain.resources.minerals);
    assert!(upgraded.resources.data > plain.resources.data);
}

#[test]
fn mineral_capacity_grows_with_storage_buildings() {
    let mut state = PlanetState::default();
    let base = state.mineral_capacity();
    let core = state.grid.find_core().unwrap();
    let pos = crate::engine::GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::Storage);
    state.select_building(BuildingType::Storage);
    state.try_place_building(pos);
    assert!(state.mineral_capacity() > base);
}

#[test]
fn battery_time_left_converts_seconds_to_hours_and_minutes() {
    let state = PlanetState {
        battery_seconds: 3661.0, // 1h 1m 1s
        ..Default::default()
    };
    assert_eq!(state.battery_time_left(), (1, 1));
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
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    let (unlocked_after, total_after) = state.achievements_progress();
    assert!(unlocked_after > unlocked_before);
    assert_eq!(total_after, total);
}
