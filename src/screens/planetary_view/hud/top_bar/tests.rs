use super::{mobile_status_text, top_bar_layout};

#[test]
fn mobile_arrival_status_names_the_world_without_hiding_critical_failures() {
    assert_eq!(
        mobile_status_text(false, false, false, true, "", "Venus"),
        ("ARRIVED VENUS".to_string(), true)
    );
    assert_eq!(
        mobile_status_text(true, false, false, true, "", "Venus"),
        ("SAVE FAILED".to_string(), false)
    );
    assert_eq!(
        mobile_status_text(false, false, false, false, "ACID RAIN", "Venus"),
        ("ACID RAIN".to_string(), false)
    );
}

#[test]
fn narrow_top_bar_keeps_resource_cards_before_navigation_buttons() {
    let layout = top_bar_layout(640.0, 76.0, 10.0);
    let cards_x = layout.brand_width + 10.0;
    let cards_right = cards_x
        + layout.card_width * layout.card_count as f32
        + layout.card_gap * (layout.card_count.saturating_sub(1)) as f32;
    let action_x = 640.0 - layout.actions_width - 8.0;
    let button_width = (layout.actions_width - 6.0 * 3.0) / 4.0;

    assert!(layout.abbreviated_labels);
    assert!(cards_right <= action_x - 10.0);
    assert!(button_width >= 50.0);
}
