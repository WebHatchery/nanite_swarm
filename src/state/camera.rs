//! Where the player is looking at a planet from.
//!
//! Pan is a logical-pixel offset added to the grid's origin; zoom scales the
//! tile size. Everything that draws or hit-tests the grid goes through those
//! two numbers, so the view and the mouse can never disagree about which tile
//! is under the cursor.

use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::camera::{CameraBounds, CameraBoundsPolicy, CameraTransform};
use serde::{Deserialize, Serialize};

use crate::engine::GridPos;

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
    // The saved pan/zoom schema stays unchanged. The shared pure transform owns
    // screen-space operations; the game chooses its map visibility policy.
    fn transform(&self, viewport: Rect) -> Option<CameraTransform> {
        CameraTransform::new(
            (viewport.center() - vec2(self.pan_x, self.pan_y)) / self.zoom,
            self.zoom,
        )
        .ok()
    }

    fn apply_transform(&mut self, viewport: Rect, transform: CameraTransform) {
        let pan = viewport.center() - transform.target() * transform.zoom();
        self.pan_x = pan.x;
        self.pan_y = pan.y;
        self.zoom = transform.zoom();
    }

    pub fn pan_by(&mut self, dx: f32, dy: f32) {
        let viewport = Rect::new(-0.5, -0.5, 1.0, 1.0);
        if let Some(mut transform) = self.transform(viewport) {
            if transform.pan_screen(vec2(dx, dy)) {
                self.apply_transform(viewport, transform);
            }
        }
    }

    /// Center a grid tile in the available map viewport, then apply the same
    /// visibility clamp used after manual panning.
    pub fn center_on_grid(
        &mut self,
        position: GridPos,
        viewport: (f32, f32),
        tile_size: f32,
        grid_size: (f32, f32),
    ) {
        if !tile_size.is_finite()
            || tile_size <= 0.0
            || !viewport.0.is_finite()
            || !viewport.1.is_finite()
        {
            return;
        }
        self.pan_x = viewport.0 * 0.5 - (position.x as f32 + 0.5) * tile_size;
        self.pan_y = viewport.1 * 0.5 - (position.y as f32 + 0.5) * tile_size;
        self.clamp_to_viewport(viewport, grid_size);
    }

    /// Zoom by `notches` wheel steps, keeping whatever is under the cursor
    /// under the cursor.
    ///
    /// `cursor` is the cursor position relative to the grid's unpanned origin.
    pub fn zoom_by(&mut self, notches: f32, cursor: (f32, f32)) {
        if notches == 0.0 {
            return;
        }
        self.zoom_by_scale(ZOOM_STEP.powf(notches), cursor);
    }

    /// Zoom by a direct scale, as reported by a two-finger pinch.
    pub fn zoom_by_scale(&mut self, scale: f32, cursor: (f32, f32)) {
        if !scale.is_finite() || scale <= 0.0 || scale == 1.0 {
            return;
        }
        if (self.zoom * scale).clamp(MIN_ZOOM, MAX_ZOOM) == self.zoom {
            return;
        }
        let viewport = Rect::new(-0.5, -0.5, 1.0, 1.0);
        if let Some(mut transform) = self.transform(viewport) {
            if transform.zoom_at(
                viewport,
                vec2(cursor.0, cursor.1),
                scale,
                (MIN_ZOOM, MAX_ZOOM),
            ) {
                self.apply_transform(viewport, transform);
            }
        }
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
        let viewport = Rect::new(0.0, 0.0, viewport.0, viewport.1);
        if let Some(mut transform) = self.transform(viewport) {
            let bounds = CameraBounds {
                min: (viewport.center() - vec2(max_x, max_y)) / self.zoom,
                max: (viewport.center() - vec2(min_x, min_y)) / self.zoom,
            };
            if transform.constrain(viewport, bounds, CameraBoundsPolicy::TargetInside) {
                self.apply_transform(viewport, transform);
            }
        }
    }
}

#[cfg(test)]
mod tests;
