use super::*;

#[test]
fn mobile_launch_caption_has_room_for_wrapped_story_lines() {
    let ascent = caption_geometry(320.0, 480.0, false);
    let arrival = caption_geometry(320.0, 480.0, true);

    assert_eq!(ascent.text_size, 12.0);
    assert_eq!(ascent.max_width, 292.0);
    assert_eq!(ascent.band_h, 82.0);
    assert_eq!(arrival.band_h, 128.0);
    assert!(arrival.band_y - 34.0 >= 0.0);
    assert!(arrival.band_y - 34.0 + arrival.band_h < 480.0 - 18.0);
}

#[test]
fn desktop_launch_caption_preserves_the_original_cinematic_band() {
    let geometry = caption_geometry(1280.0, 720.0, false);

    assert_eq!(geometry.text_size, 14.0);
    assert_eq!(geometry.max_width, 1252.0);
    assert_eq!(geometry.band_h, 62.0);
}
