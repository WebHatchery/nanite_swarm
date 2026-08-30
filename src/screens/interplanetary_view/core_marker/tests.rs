use super::normalized_stage;

#[test]
fn stage_progress_keeps_player_facing_values_in_range() {
    assert_eq!(normalized_stage(2, 5), (2, 5));
    assert_eq!(normalized_stage(0, 5), (1, 5));
    assert_eq!(normalized_stage(9, 5), (5, 5));
}

#[test]
fn malformed_stage_counts_still_produce_a_visible_marker() {
    assert_eq!(normalized_stage(0, 0), (1, 1));
    assert_eq!(normalized_stage(4, 0), (1, 1));
}
