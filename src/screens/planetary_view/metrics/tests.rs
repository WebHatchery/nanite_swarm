use super::*;

#[test]
fn focus_mode_returns_both_sidebar_widths_to_the_map() {
    let theme = UiTheme::default();
    let normal = HudMetrics::for_screen(&theme, 1280.0, 720.0, Camera::default(), false);
    let focused = HudMetrics::for_screen(&theme, 1280.0, 720.0, Camera::default(), true);

    let normal_view = normal.viewport(1280.0, 720.0);
    let focused_view = focused.viewport(1280.0, 720.0);
    assert!(focused_view.0 > normal_view.0 + 500.0);
    assert_eq!(focused.left_panel_width, 0.0);
    assert_eq!(focused.right_panel_width, 0.0);
    assert_eq!(focused.base_offset_x(), focused.panel_gap * 2.0);
}
