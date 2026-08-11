use super::*;

fn modifier(stat: &str, op: &str, value: f32) -> ModifierDef {
    ModifierDef {
        stat: stat.to_string(),
        op: op.to_string(),
        value,
    }
}

#[test]
fn every_stat_the_game_knows_has_something_to_call_itself() {
    for stat in StatId::ALL {
        assert_ne!(
            stat.label(),
            stat.id(),
            "{} is only ever shown as its raw id",
            stat.id()
        );
    }
}

#[test]
fn a_percent_is_said_as_a_percent_and_a_count_as_a_count() {
    let percent = describe_modifier(&modifier("drill_output", "percent", 0.25)).unwrap();
    assert_eq!(percent.change, "+25%");
    let count = describe_modifier(&modifier("drones_per_drill", "add", 1.0)).unwrap();
    assert_eq!(count.change, "+1");
    assert_eq!(count.label, StatId::DronesPerDrill.label());
}

#[test]
fn less_of_a_bad_thing_reads_as_a_gain_and_more_of_it_does_not() {
    let quieter = describe_modifier(&modifier("dust_accumulation", "percent", -0.3)).unwrap();
    assert_eq!(quieter.change, "-30%");
    assert!(quieter.is_gain, "less dust is good news");

    let hungrier = describe_modifier(&modifier("power_consumption", "percent", 0.2)).unwrap();
    assert!(!hungrier.is_gain, "more power draw is a cost");

    // And the ordinary direction still holds for ordinary stats.
    let weaker = describe_modifier(&modifier("drill_output", "percent", -0.1)).unwrap();
    assert!(!weaker.is_gain);
}

#[test]
fn a_modifier_the_game_cannot_read_is_not_described() {
    assert!(describe_modifier(&modifier("not_a_stat", "add", 1.0)).is_none());
    assert!(describe_modifier(&modifier("drill_output", "multiply", 1.0)).is_none());
}

#[test]
fn stat_ids_round_trip_through_their_names() {
    for stat in StatId::ALL {
        assert_eq!(StatId::from_id(stat.id()), Some(stat));
    }
    assert_eq!(StatId::from_id("not_a_stat"), None);
}

#[test]
fn parse_modifier_names_the_unknown_stat_and_lists_the_known_ones() {
    let error = parse_modifier(&modifier("wobble", "add", 1.0)).unwrap_err();
    assert!(error.contains("wobble"), "{error}");
    assert!(error.contains("drill_output"), "{error}");
}

#[test]
fn parse_modifier_rejects_an_unknown_op() {
    let error = parse_modifier(&modifier("drill_output", "divide", 2.0)).unwrap_err();
    assert!(error.contains("divide"), "{error}");
}

#[test]
fn an_empty_sheet_leaves_every_base_value_alone() {
    let stats = Stats::default();
    for stat in StatId::ALL {
        assert_eq!(stats.apply(stat, 7.0), 7.0);
        assert_eq!(stats.multiplier(stat), 1.0);
    }
}

#[test]
fn percents_sum_before_they_apply_so_order_never_matters() {
    let mut forward = Stats::default();
    forward.push(StatId::DrillOutput, ModifierOp::Percent, 0.5);
    forward.push(StatId::DrillOutput, ModifierOp::Percent, 0.25);

    let mut backward = Stats::default();
    backward.push(StatId::DrillOutput, ModifierOp::Percent, 0.25);
    backward.push(StatId::DrillOutput, ModifierOp::Percent, 0.5);

    assert_eq!(forward, backward);
    // 10 * (1 + 0.75), not 10 * 1.5 * 1.25.
    assert_eq!(forward.apply(StatId::DrillOutput, 10.0), 17.5);
}

#[test]
fn adds_land_before_percents() {
    let mut stats = Stats::default();
    stats.push(StatId::MineralCapacity, ModifierOp::Add, 50.0);
    stats.push(StatId::MineralCapacity, ModifierOp::Percent, 1.0);
    assert_eq!(stats.apply(StatId::MineralCapacity, 100.0), 300.0);
}

#[test]
fn a_negative_percent_reduces_the_stat() {
    let mut stats = Stats::default();
    stats.push(StatId::PowerConsumption, ModifierOp::Percent, -0.25);
    assert_eq!(stats.apply(StatId::PowerConsumption, 8.0), 6.0);
}

#[test]
fn modifiers_only_count_once_their_node_is_unlocked() {
    let locked = Stats::from_unlocked(&[]);
    assert_eq!(locked.multiplier(StatId::DrillOutput), 1.0);

    let unlocked = Stats::from_unlocked(&["efficient_drills".to_string()]);
    assert!(unlocked.multiplier(StatId::DrillOutput) > 1.0);
}

#[test]
fn the_shipped_research_tree_declares_only_valid_modifiers() {
    for node in &crate::data::game_data().research.nodes {
        for modifier in &node.modifiers {
            assert!(
                parse_modifier(modifier).is_ok(),
                "node {} declares an invalid modifier: {:?}",
                node.id,
                modifier
            );
        }
    }
}
