use super::{mobile_offline_work_label, mobile_status_text, offline_work_label, top_bar_layout};
use crate::state::OfflineReport;

#[test]
fn mobile_arrival_status_names_the_world_without_hiding_critical_failures() {
    assert_eq!(
        mobile_status_text(false, false, false, true, "", "", "Venus"),
        ("ARRIVED VENUS".to_string(), true)
    );
    assert_eq!(
        mobile_status_text(true, false, false, true, "", "", "Venus"),
        ("SAVE FAILED".to_string(), false)
    );
    assert_eq!(
        mobile_status_text(false, false, false, false, "", "ACID RAIN", "Venus"),
        ("ACID RAIN".to_string(), false)
    );
}

#[test]
fn offline_work_status_is_short_enough_for_the_mobile_header() {
    let report = OfflineReport {
        research_completed: true,
        seed_ship_stages_completed: 2,
        ..OfflineReport::default()
    };

    assert_eq!(mobile_offline_work_label(report), "RESEARCH DONE");
    assert_eq!(
        offline_work_label(OfflineReport {
            seed_ship_stages_completed: 2,
            ..OfflineReport::default()
        })
        .as_deref(),
        Some("Seed Ship +2 stages")
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
