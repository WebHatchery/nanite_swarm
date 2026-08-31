use super::mobile_panel_geometry;

#[test]
fn short_phone_completion_report_keeps_both_choices_inside_the_view() {
    let (y, height) = mobile_panel_geometry(480.0);

    assert_eq!(y, 36.0);
    assert_eq!(height, 428.0);
    assert!(y + height <= 464.0);
}

#[test]
fn very_short_phone_completion_report_stays_inside_the_view() {
    let (y, height) = mobile_panel_geometry(320.0);

    assert_eq!((y, height), (36.0, 268.0));
    assert!(y + height <= 304.0);
}

#[test]
fn regular_phone_completion_report_keeps_its_spacious_layout() {
    assert_eq!(mobile_panel_geometry(640.0), (86.0, 520.0));
}
