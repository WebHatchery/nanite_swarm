use super::*;

fn metrics(screen_w: f32, screen_h: f32) -> HudMetrics {
    HudMetrics::for_screen(
        &UiTheme::default(),
        screen_w,
        screen_h,
        crate::state::Camera::default(),
        false,
    )
}

#[test]
fn factory_deck_fits_inside_portrait_viewports() {
    for (screen_w, screen_h) in [(360.0, 640.0), (320.0, 480.0)] {
        let layout = factory_deck_layout(screen_w, screen_h, metrics(screen_w, screen_h));
        assert!(layout.narrow);
        assert!(layout.panel.x >= 0.0);
        assert!(layout.panel.right() <= screen_w);
        assert!(layout.panel.bottom() <= screen_h - 64.0 - 12.0);
    }
}

#[test]
fn factory_deck_keeps_the_wide_control_room_geometry() {
    let layout = factory_deck_layout(1280.0, 720.0, metrics(1280.0, 720.0));

    assert!(!layout.narrow);
    assert!(!layout.compact);
    assert_eq!(layout.panel.w, 500.0);
    assert_eq!(layout.panel.h, 302.0);
    assert!(layout.panel.x > 300.0);
}

#[test]
fn narrow_focus_copy_stays_short_enough_for_touch_cards() {
    for focus in FactoryFocus::ALL {
        assert!(narrow_focus_description(focus).len() <= 25);
    }
}
