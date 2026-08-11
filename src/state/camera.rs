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
mod tests;
