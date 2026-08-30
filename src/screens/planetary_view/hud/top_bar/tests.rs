use super::{
    mobile_offline_report_lines, mobile_offline_work_label, mobile_research_status,
    mobile_status_text, offline_work_label, top_bar_layout,
};
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
fn mobile_header_can_report_active_research_progress() {
    let data = crate::data::test_data();

    assert_eq!(
        mobile_research_status(Some("adaptive_clocking"), 42.0, data.data()).as_deref(),
        Some("R 42/110 DATA")
    );
    assert!(mobile_research_status(Some("missing"), 42.0, data.data()).is_none());
    assert!(mobile_research_status(None, 42.0, data.data()).is_none());
}

#[test]
fn offline_report_keeps_data_spent_and_ship_stages_visible() {
    let report = OfflineReport {
        elapsed_seconds: 21_600.0,
        capped_seconds: 14_400.0,
        minerals_gained: 740.0,
        alloy_gained: 120.0,
        components_gained: 64.0,
        data_gained: 360.0,
        power_gained: 18.0,
        research_data_spent: 110.0,
        research_completed: true,
        seed_ship_stages_completed: 1,
        ..OfflineReport::default()
    };
    assert_eq!(
        offline_work_label(report).as_deref(),
        Some("research complete (-110 data) | Seed Ship +1 stage")
    );
    let lines = mobile_offline_report_lines(report, 21_600.0, 14_400.0);
    assert_eq!(lines[0], "Away 6h 0m | Sim 4h 0m | CAPPED");
    assert!(lines[1].contains("+360 data"));
    assert!(lines[2].contains("-110 data"));
    assert!(lines[2].contains("Seed Ship +1 stage"));
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
