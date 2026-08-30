use super::*;

#[test]
fn short_desktop_yard_reserves_space_for_a_fixed_action_footer() {
    let layout = compact_yard_geometry(800.0, 400.0, 4, true);
    let footer_y = layout.footer_y.expect("desktop compact footer");

    assert_eq!(layout.view.y, HEADER_HEIGHT + 12.0);
    assert!(layout.view.bottom() + 20.0 <= footer_y);
    assert!(footer_y + Dimensions::BUTTON_HEIGHT + 14.0 <= 400.0 - 18.0);
    assert_eq!(layout.content_h, layout.stages_h);
    assert!(layout.content_h > layout.view.h);
}

#[test]
fn phone_yard_keeps_actions_in_the_scrollable_content() {
    let layout = compact_yard_geometry(390.0, 844.0, 4, false);

    assert!(layout.footer_y.is_none());
    assert_eq!(layout.content_h, layout.stages_h + 92.0);
    assert!(layout.view.h > layout.stages_h);
}
