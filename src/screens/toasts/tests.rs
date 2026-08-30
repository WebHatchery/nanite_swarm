use super::*;

#[test]
fn compact_view_uses_a_narrower_stack_and_keeps_newest_three() {
    assert!(compact_screen(800.0, 600.0));
    assert_eq!(toast_width_for_viewport(800.0, 600.0), 200.0);
    assert_eq!(max_visible_toasts(800.0, 600.0), 3);
}

#[test]
fn roomy_view_keeps_the_full_notification_stack() {
    assert!(!compact_screen(1280.0, 720.0));
    assert_eq!(toast_width_for_viewport(1280.0, 720.0), 260.0);
    assert_eq!(max_visible_toasts(1280.0, 720.0), usize::MAX);
}
