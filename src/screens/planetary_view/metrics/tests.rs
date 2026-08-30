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

#[test]
fn portrait_metrics_keep_the_map_open_and_drawers_full_width() {
    let theme = UiTheme::default();
    let metrics = HudMetrics::for_screen(&theme, 360.0, 640.0, Camera::default(), false);
    let map_view = metrics.viewport(360.0, 640.0);
    let build = metrics.mobile_build_panel(360.0);
    let info = metrics.mobile_info_panel(360.0);

    assert!(metrics.mobile);
    assert_eq!(metrics.left_panel_width, 0.0);
    assert_eq!(metrics.right_panel_width, 0.0);
    assert!(map_view.0 > 300.0);
    assert!(build.left_panel_width > 340.0);
    assert_eq!(info.left_panel_width, 0.0);
    assert!(info.right_panel_width > 330.0);
}

#[test]
fn portrait_drawers_claim_their_overlay_but_leave_the_map_touchable() {
    let theme = UiTheme::default();
    let metrics = HudMetrics::for_screen(&theme, 360.0, 640.0, Camera::default(), false);

    assert!(is_cursor_over_ui(
        20.0,
        100.0,
        360.0,
        640.0,
        metrics,
        MobilePanel::Map,
    ));
    assert!(!is_cursor_over_ui(
        180.0,
        240.0,
        360.0,
        640.0,
        metrics,
        MobilePanel::Map,
    ));
    assert!(is_cursor_over_ui(
        180.0,
        240.0,
        360.0,
        640.0,
        metrics,
        MobilePanel::Build,
    ));
}
