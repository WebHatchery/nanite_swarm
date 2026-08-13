use super::*;

#[test]
fn a_fresh_camera_shows_the_map_unscaled_from_its_origin() {
    let camera = Camera::default();
    assert_eq!(camera.pan_x, 0.0);
    assert_eq!(camera.pan_y, 0.0);
    assert_eq!(camera.zoom, 1.0);
}

#[test]
fn panning_accumulates() {
    let mut camera = Camera::default();
    camera.pan_by(10.0, -5.0);
    camera.pan_by(2.0, 1.0);
    assert_eq!((camera.pan_x, camera.pan_y), (12.0, -4.0));
}

#[test]
fn zoom_stays_within_its_limits() {
    let mut camera = Camera::default();
    camera.zoom_by(100.0, (0.0, 0.0));
    assert_eq!(camera.zoom, MAX_ZOOM);
    camera.zoom_by(-100.0, (0.0, 0.0));
    assert_eq!(camera.zoom, MIN_ZOOM);
}

#[test]
fn zooming_keeps_the_point_under_the_cursor_in_place() {
    let base_tile = 28.0;
    let cursor = (300.0, 180.0);
    let mut camera = Camera::default();

    // Grid coordinate under the cursor before the zoom.
    let before = (
        (cursor.0 - camera.pan_x) / (base_tile * camera.zoom),
        (cursor.1 - camera.pan_y) / (base_tile * camera.zoom),
    );

    camera.zoom_by(3.0, cursor);

    let after = (
        (cursor.0 - camera.pan_x) / (base_tile * camera.zoom),
        (cursor.1 - camera.pan_y) / (base_tile * camera.zoom),
    );
    assert!((before.0 - after.0).abs() < 1e-3, "{before:?} vs {after:?}");
    assert!((before.1 - after.1).abs() < 1e-3, "{before:?} vs {after:?}");
    assert!(camera.zoom > 1.0);
}

#[test]
fn zooming_out_keeps_the_point_under_the_cursor_too() {
    let base_tile = 28.0;
    let cursor = (420.0, 90.0);
    let mut camera = Camera {
        pan_x: -60.0,
        pan_y: 30.0,
        zoom: 1.8,
    };
    let before = (
        (cursor.0 - camera.pan_x) / (base_tile * camera.zoom),
        (cursor.1 - camera.pan_y) / (base_tile * camera.zoom),
    );

    camera.zoom_by(-2.0, cursor);

    let after = (
        (cursor.0 - camera.pan_x) / (base_tile * camera.zoom),
        (cursor.1 - camera.pan_y) / (base_tile * camera.zoom),
    );
    assert!((before.0 - after.0).abs() < 1e-3);
    assert!((before.1 - after.1).abs() < 1e-3);
    assert!(camera.zoom < 1.8);
}

#[test]
fn pinch_scale_keeps_its_center_over_the_same_grid_point() {
    let cursor = (240.0, 160.0);
    let mut camera = Camera::default();
    let before = (
        (cursor.0 - camera.pan_x) / camera.zoom,
        (cursor.1 - camera.pan_y) / camera.zoom,
    );

    camera.zoom_by_scale(1.5, cursor);

    let after = (
        (cursor.0 - camera.pan_x) / camera.zoom,
        (cursor.1 - camera.pan_y) / camera.zoom,
    );
    assert!((before.0 - after.0).abs() < 1e-3);
    assert!((before.1 - after.1).abs() < 1e-3);
    assert_eq!(camera.zoom, 1.5);
}

#[test]
fn nonsense_pinch_scales_do_not_move_the_camera() {
    let original = Camera {
        pan_x: 12.0,
        pan_y: -8.0,
        zoom: 1.0,
    };
    for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let mut camera = original;
        camera.zoom_by_scale(scale, (200.0, 200.0));
        assert_eq!(camera, original);
    }
}

#[test]
fn a_zoom_that_changes_nothing_leaves_the_pan_alone() {
    let mut camera = Camera {
        pan_x: 12.0,
        pan_y: -8.0,
        zoom: MAX_ZOOM,
    };
    camera.zoom_by(5.0, (200.0, 200.0));
    assert_eq!((camera.pan_x, camera.pan_y), (12.0, -8.0));

    camera.zoom_by(0.0, (200.0, 200.0));
    assert_eq!((camera.pan_x, camera.pan_y), (12.0, -8.0));
}

#[test]
fn the_map_cannot_be_dragged_out_of_sight() {
    let viewport = (800.0, 600.0);
    let grid_size = (672.0, 672.0);

    let mut far_right = Camera {
        pan_x: 5_000.0,
        pan_y: 0.0,
        zoom: 1.0,
    };
    far_right.clamp_to_viewport(viewport, grid_size);
    assert_eq!(far_right.pan_x, viewport.0 - KEEP_VISIBLE);

    let mut far_left = Camera {
        pan_x: -5_000.0,
        pan_y: -5_000.0,
        zoom: 1.0,
    };
    far_left.clamp_to_viewport(viewport, grid_size);
    assert_eq!(far_left.pan_x, -(grid_size.0 - KEEP_VISIBLE));
    assert_eq!(far_left.pan_y, -(grid_size.1 - KEEP_VISIBLE));
}

#[test]
fn clamping_leaves_a_camera_that_is_already_looking_at_the_map() {
    let mut camera = Camera {
        pan_x: -100.0,
        pan_y: 40.0,
        zoom: 1.0,
    };
    let before = camera;
    camera.clamp_to_viewport((800.0, 600.0), (672.0, 672.0));
    assert_eq!(camera, before);
}
