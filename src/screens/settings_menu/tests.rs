use super::*;

#[test]
fn volume_stepper_uses_ten_percent_increments_and_clamps() {
    let mut volume = 0.3;
    volume = step_volume(volume, 1);
    assert_eq!(volume, 0.4);

    volume = step_volume(volume, -1);
    assert_eq!(volume, 0.3);

    volume = step_volume(volume, -8);
    assert_eq!(volume, 0.0);
    volume = step_volume(volume, 20);
    assert_eq!(volume, 1.0);
}

#[test]
fn short_mobile_settings_keep_display_controls_above_the_footer() {
    let layout = settings_layout(240.0, 400.0);
    let content_y = layout.panel_y + 78.0;
    let motion_y = content_y + (layout.row_h + layout.row_gap) * 2.0;
    let key_y = motion_y + layout.row_h + layout.section_gap;
    let speed_y = key_y + layout.row_h + layout.section_gap;

    assert!(layout.compact_mobile);
    assert!((layout.panel_y - 76.0).abs() < 0.001);
    assert!(layout.panel_y + layout.panel_h <= 400.0);
    assert!(speed_y + layout.row_h <= 400.0 - 48.0);
    assert!((layout.row_gap - 6.0).abs() < 0.001);
    assert!((layout.section_gap - 18.0).abs() < 0.001);
}

#[test]
fn short_mobile_settings_keep_audio_test_signal_above_the_footer() {
    let layout = settings_layout(240.0, 400.0);
    let content_y = layout.panel_y + 90.0;
    let test_y = content_y + (layout.row_h + layout.row_gap) * 3.0;

    assert!(test_y + layout.row_h <= 400.0 - 48.0);
}

#[test]
fn short_mobile_settings_move_the_footer_clear_of_the_lower_rows() {
    let layout = settings_layout(240.0, 400.0);
    let footer_y = 400.0 - 42.0;
    let detail_y = 400.0 - 22.0;

    assert!(layout.compact_mobile);
    assert!(footer_y > 352.0);
    assert!(detail_y > footer_y);
}

#[test]
fn established_phone_settings_keep_the_existing_geometry() {
    let layout = settings_layout(320.0, 480.0);

    assert!(!layout.compact_mobile);
    assert!((layout.panel_y - 84.0).abs() < 0.001);
    assert!((layout.panel_h - 332.0).abs() < 0.001);
    assert!((layout.row_gap - 12.0).abs() < 0.001);
    assert!((layout.section_gap - 30.0).abs() < 0.001);
}
