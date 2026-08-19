use super::*;
use crate::data::GameConfig;
use crate::engine::{BuildingType, GridPos};
use crate::state::PlanetState;

#[test]
fn the_tier_walks_the_declared_set_in_order_and_wraps() {
    let set = &crate::data::game_data().directives.directives;
    assert!(
        set.len() > 4,
        "the shipped set is no bigger than the old one"
    );
    for (tier, def) in set.iter().enumerate() {
        let directive = pick_directive(tier as i32);
        assert_eq!(directive.kind, DirectiveKind::from_id(&def.kind).unwrap());
    }
    // And it comes back round rather than running out.
    assert_eq!(
        pick_directive(set.len() as i32).kind,
        pick_directive(0).kind
    );
}

#[test]
fn every_shipped_directive_says_what_it_wants_and_pays_for_it() {
    for (tier, _) in crate::data::game_data()
        .directives
        .directives
        .iter()
        .enumerate()
    {
        let directive = pick_directive(tier as i32);
        assert!(!directive.description.is_empty());
        assert!(
            !directive.description.contains('{'),
            "a placeholder survived: {}",
            directive.description
        );
        assert!(directive.target >= 1);
        assert!(directive.reward_data > 0.0);
    }
}

#[test]
fn a_later_lap_of_the_set_asks_for_more_and_pays_more() {
    let set_len = crate::data::game_data().directives.directives.len() as i32;
    let first = pick_directive(0);
    let second = pick_directive(set_len);
    assert_eq!(first.kind, second.kind);
    assert!(second.target > first.target, "the ask did not grow");
    assert!(
        second.reward_data > first.reward_data,
        "the pay did not grow"
    );
}

#[test]
fn holding_power_is_timed_by_its_own_number_not_by_the_threshold() {
    // The old shape used the target for both, so asking for more power
    // also asked for it to be held longer.
    let directive = pick_directive(0);
    assert_eq!(directive.kind, DirectiveKind::PowerSurplus);
    let harder = pick_directive(crate::data::game_data().directives.directives.len() as i32);
    assert!(harder.target > directive.target);
    assert_eq!(harder.hold_seconds, directive.hold_seconds);
}

#[test]
fn power_surplus_directive_progresses_only_while_target_is_met() {
    let mut directive = Directive::for_test(DirectiveKind::PowerSurplus, 5, 100.0);
    let mut state = PlanetState::new(2, 1, GameConfig::default());
    state.power_balance = 10.0;

    directive.update(&state, 1.0);
    assert_eq!(directive.progress, 1);

    state.power_balance = -5.0;
    directive.update(&state, 1.0);
    assert_eq!(directive.progress, 0);
}

#[test]
fn drill_count_directive_tracks_placed_drills_and_completes() {
    let mut directive = Directive::for_test(DirectiveKind::DrillCount, 1, 100.0);
    let mut state = PlanetState::new(2, 1, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    directive.update(&state, 0.0);
    assert_eq!(directive.progress, 1);
    assert!(directive.completed);
}

#[test]
fn completed_directive_no_longer_updates() {
    let mut directive = Directive::for_test(DirectiveKind::PowerSurplus, 1, 100.0);
    directive.completed = true;
    directive.progress = 1;
    let mut state = PlanetState::new(2, 1, GameConfig::default());
    state.power_balance = -50.0;

    directive.update(&state, 1.0);
    // A completed directive is frozen: progress doesn't regress further.
    assert_eq!(directive.progress, 1);
}

#[test]
fn duration_counts_down_but_never_below_zero() {
    let mut directive = Directive::for_test(DirectiveKind::HarvestForest, 5, 2.0);
    let state = PlanetState::new(2, 1, GameConfig::default());
    directive.update(&state, 1.5);
    assert_eq!(directive.duration, 0.5);
    directive.update(&state, 5.0);
    assert_eq!(directive.duration, 0.0);
}

#[test]
fn late_factory_directive_counts_banked_components() {
    let mut directive = Directive::for_test(DirectiveKind::ComponentStock, 8, 100.0);
    let mut state = PlanetState::new(2, 1, GameConfig::default());
    state.resources.components = 7.9;
    directive.update(&state, 0.0);
    assert_eq!(directive.progress, 7);
    assert!(!directive.completed);

    state.resources.components = 8.0;
    directive.update(&state, 0.0);
    assert!(directive.completed);
}
