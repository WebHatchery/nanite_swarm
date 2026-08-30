use super::mobile_info_height;

#[test]
fn short_phone_dossier_stays_above_the_footer() {
    let list_bottom = 156.0 + 44.0 + 6.0 * 24.0;
    let dossier_bottom = list_bottom + 10.0 + mobile_info_height(480.0);

    assert_eq!(mobile_info_height(480.0), 96.0);
    assert!(dossier_bottom <= 450.0);
}

#[test]
fn regular_phone_dossier_keeps_the_full_world_summary() {
    assert_eq!(mobile_info_height(640.0), 228.0);
}
