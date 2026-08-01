//! Where the player is looking at a planet from.
//!
//! Pan is a logical-pixel offset added to the grid's origin; zoom scales the
//! tile size. Everything that draws or hit-tests the grid goes through those
//! two numbers, so the view and the mouse can never disagree about which tile
//! is under the cursor.

use serde::{Deserialize, Serialize};

pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 2.5;
/// Multiplier per wheel notch.
const ZOOM_STEP: f32 = 1.12;
/// Grid edge that must stay inside the viewport, in logical pixels, so the map
/// can never be dragged completely out of sight.
const KEEP_VISIBLE: f32 = 64.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub fn pan_by(&mut self, dx: f32, dy: f32) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    /// Zoom by `notches` wheel steps, keeping whatever is under the cursor
    /// under the cursor.
    ///
    /// `cursor` is the cursor position relative to the grid's unpanned origin.
    pub fn zoom_by(&mut self, notches: f32, cursor: (f32, f32)) {
        if notches == 0.0 {
            return;
        }
        let target = (self.zoom * ZOOM_STEP.powf(notches)).clamp(MIN_ZOOM, MAX_ZOOM);
        if target == self.zoom {
            return;
        }

        // The tile under the cursor sits at (cursor - pan) / (base * zoom) in
        // grid units; solving for the pan that keeps it there gives:
        let ratio = target / self.zoom;
        self.pan_x = cursor.0 - (cursor.0 - self.pan_x) * ratio;
        self.pan_y = cursor.1 - (cursor.1 - self.pan_y) * ratio;
        self.zoom = target;
    }

    /// Keep at least a corner of the map inside the viewport.
    ///
    /// `viewport` is the width and height of the area the grid is drawn into,
    /// `grid_size` the map's on-screen size at the current zoom, both measured
    /// from the grid's unpanned origin.
    pub fn clamp_to_viewport(&mut self, viewport: (f32, f32), grid_size: (f32, f32)) {
        let min_x = -(grid_size.0 - KEEP_VISIBLE).max(0.0);
        let max_x = (viewport.0 - KEEP_VISIBLE).max(0.0);
        let min_y = -(grid_size.1 - KEEP_VISIBLE).max(0.0);
        let max_y = (viewport.1 - KEEP_VISIBLE).max(0.0);
        self.pan_x = self.pan_x.clamp(min_x, max_x);
        self.pan_y = self.pan_y.clamp(min_y, max_y);
    }
}

#[cfg(test)]
mod tests {
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
}
