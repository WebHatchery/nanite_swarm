use super::{mobile_info_height, mobile_status_text};

#[test]
fn mobile_dossier_status_keeps_the_remote_alert_reason_visible() {
    assert_eq!(
        mobile_status_text(true, None),
        "Status: COLONIZED".to_string()
    );
    assert_eq!(
        mobile_status_text(true, Some("MAINT DUST")),
        "Status: COLONIZED / MAINT DUST".to_string()
    );
    assert_eq!(
        mobile_status_text(false, None),
        "Status: UNEXPLORED".to_string()
    );
}

#[test]
fn short_phone_dossier_stays_above_the_footer() {
    let list_bottom = 156.0 + 44.0 + crate::state::PLANET_COUNT as f32 * 24.0;
    let dossier_bottom = list_bottom + 10.0 + mobile_info_height(480.0, true, false);

    assert_eq!(mobile_info_height(480.0, false, false), 96.0);
    assert_eq!(mobile_info_height(480.0, true, false), 124.0);
    assert!(dossier_bottom <= 454.0);
}

#[test]
fn regular_phone_dossier_keeps_the_full_world_summary() {
    assert_eq!(mobile_info_height(640.0, true, false), 250.0);
    assert_eq!(mobile_info_height(640.0, true, true), 282.0);
}
