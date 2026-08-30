use super::*;

#[test]
fn narrow_populated_menu_keeps_every_command_inside_the_view() {
    let layout = mobile_menu_layout(240.0, 400.0, 8);
    let first_y = layout.panel_y + 34.0;
    let last_y = first_y + 7.0 * layout.button_spacing;

    assert!(layout.briefing_y + layout.briefing_h <= layout.panel_y);
    assert!(layout.panel_y + layout.panel_h <= 400.0);
    assert!(last_y + layout.button_h <= 400.0);
    assert!((layout.button_h - 20.0).abs() < 0.001);
    assert!((layout.button_spacing - 21.0).abs() < 0.001);
}

#[test]
fn roomy_phone_menu_keeps_the_existing_spacing() {
    let layout = mobile_menu_layout(360.0, 640.0, 7);

    assert!((layout.briefing_y - 150.0).abs() < 0.001);
    assert!((layout.briefing_h - 58.0).abs() < 0.001);
    assert!((layout.panel_y - 218.0).abs() < 0.001);
    assert!((layout.button_h - 28.0).abs() < 0.001);
    assert!((layout.button_spacing - 34.0).abs() < 0.001);
}
