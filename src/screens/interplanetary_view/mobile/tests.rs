use super::{mobile_info_height, mobile_status_text, mobile_worlds_layout};

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
fn very_short_worlds_layout_keeps_the_dossier_action_visible() {
    let layout = mobile_worlds_layout(400.0, crate::state::PLANET_COUNT);
    let info_y = 156.0 + layout.list_h + layout.info_gap;
    let dossier_bottom = info_y + mobile_info_height(400.0, true, false);

    assert!(layout.compact);
    assert!((layout.row_height - 19.0).abs() < 0.001);
    assert!((layout.list_h - 129.0).abs() < 0.001);
    assert!((info_y - 291.0).abs() < 0.001);
    assert!((dossier_bottom - 371.0).abs() < 0.001);
    assert!(dossier_bottom < 400.0 - 18.0);
}

#[test]
fn regular_phone_worlds_layout_keeps_the_existing_rows() {
    let layout = mobile_worlds_layout(480.0, crate::state::PLANET_COUNT);

    assert!(!layout.compact);
    assert!((layout.row_height - 24.0).abs() < 0.001);
    assert!((layout.list_h - 164.0).abs() < 0.001);
    assert!((layout.info_gap - 10.0).abs() < 0.001);
}

#[test]
fn regular_phone_dossier_keeps_the_full_world_summary() {
    assert_eq!(mobile_info_height(640.0, true, false), 250.0);
    assert_eq!(mobile_info_height(640.0, true, true), 282.0);
}
