use super::quick_action_layout;

#[test]
fn compact_palette_actions_fit_below_a_single_build_card() {
    let compact = quick_action_layout(310.0);
    let normal = quick_action_layout(430.0);

    assert!(compact.compact);
    assert_eq!(compact.button_height, 22.0);
    assert_eq!(compact.total_height, 154.0);
    assert!(!normal.compact);
    assert_eq!(normal.button_height, 30.0);
    assert_eq!(normal.total_height, 216.0);
}
