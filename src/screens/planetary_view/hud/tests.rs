use super::RightStackLayout;
use crate::data::UiTheme;
use crate::screens::planetary_view::metrics::HudMetrics;
use crate::state::Camera;

fn assert_stack_fits(layout: &RightStackLayout, screen_h: f32, metrics: HudMetrics) {
    let panels = [layout.inspector, layout.power, layout.ops, layout.directive];
    for pair in panels.windows(2) {
        assert!(pair[1].y >= pair[0].y + pair[0].h);
    }
    let bottom = layout.directive.y + layout.directive.h;
    assert!((bottom - (screen_h - metrics.bottom_bar_height - metrics.panel_gap)).abs() < 0.01);
}

#[test]
fn compact_right_stack_stays_above_the_bottom_bar() {
    let theme = UiTheme::default();
    let metrics = HudMetrics::for_screen(&theme, 640.0, 480.0, Camera::default(), false);
    let layout = RightStackLayout::compute(640.0, 480.0, metrics);

    assert!(layout.compact);
    assert!(layout.inspector.h >= 110.0);
    assert!(layout.directive.h >= 80.0);
    assert_stack_fits(&layout, 480.0, metrics);
}

#[test]
fn regular_right_stack_keeps_full_utility_panels() {
    let theme = UiTheme::default();
    let metrics = HudMetrics::for_screen(&theme, 1280.0, 720.0, Camera::default(), false);
    let layout = RightStackLayout::compute(1280.0, 720.0, metrics);

    assert!(!layout.compact);
    assert_stack_fits(&layout, 720.0, metrics);
}

#[test]
fn very_short_right_stack_never_creates_a_negative_panel() {
    let theme = UiTheme::default();
    let metrics = HudMetrics::for_screen(&theme, 640.0, 300.0, Camera::default(), false);
    let layout = RightStackLayout::compute(640.0, 300.0, metrics);

    assert!(layout.power.h >= 0.0);
    assert!(layout.ops.h >= 0.0);
    assert!(layout.directive.h >= 0.0);
    assert_stack_fits(&layout, 300.0, metrics);
}
