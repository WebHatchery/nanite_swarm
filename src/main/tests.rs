use super::save_failure_notice;

#[test]
fn save_failure_notice_names_the_visible_menu_recovery_action() {
    let notice = save_failure_notice();

    assert!(notice.starts_with("Save failed:"));
    assert!(notice.contains("progress is in memory"));
    assert!(notice.contains("Tap SAVE to retry"));
}
