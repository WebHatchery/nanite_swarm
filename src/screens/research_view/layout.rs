use crate::engine::ResearchNode;
use macroquad::prelude::Rect;

const MAX_NODE_RADIUS: f32 = 25.0;
const MIN_NODE_RADIUS: f32 = 13.0;
const GRID_SCALE: f32 = 100.0;
const COMPACT_BREAKPOINT: f32 = 960.0;

pub(super) struct ResearchLayout {
    pub compact: bool,
    pub intel: Rect,
    pub legend: Rect,
    pub tree: Rect,
}

impl ResearchLayout {
    pub fn for_screen(screen_w: f32, screen_h: f32, header_h: f32) -> Self {
        let compact = screen_w < COMPACT_BREAKPOINT;
        if compact {
            let mobile = screen_w < 600.0;
            let drawer_h = if mobile {
                if screen_h < 460.0 {
                    (screen_h * 0.30).clamp(112.0, 140.0)
                } else {
                    (screen_h * 0.30).clamp(172.0, 190.0)
                }
            } else {
                184.0_f32.min((screen_h * 0.32).max(144.0))
            };
            let intel = Rect::new(16.0, screen_h - drawer_h - 28.0, screen_w - 32.0, drawer_h);
            let tree_y = if mobile {
                header_h + 100.0
            } else {
                header_h + 38.0
            };
            return Self {
                compact,
                intel,
                legend: Rect::new(0.0, 0.0, 0.0, 0.0),
                tree: Rect::new(
                    if mobile { 12.0 } else { 16.0 },
                    tree_y,
                    if mobile {
                        screen_w - 24.0
                    } else {
                        screen_w - 32.0
                    },
                    (intel.y - tree_y - if mobile { 10.0 } else { 8.0 }).max(1.0),
                ),
            };
        }

        let intel = Rect::new(16.0, header_h + 12.0, 280.0, screen_h - header_h - 92.0);
        let legend = Rect::new(
            screen_w - 276.0,
            header_h + 12.0,
            260.0,
            screen_h - header_h - 92.0,
        );
        Self {
            compact,
            tree: Rect::new(
                intel.x + intel.w + 16.0,
                header_h + 12.0,
                legend.x - (intel.x + intel.w) - 32.0,
                screen_h - header_h - 72.0,
            ),
            intel,
            legend,
        }
    }
}

pub(super) fn compact_info_offset(mobile: bool, intel_height: f32) -> f32 {
    if mobile && intel_height < 150.0 {
        40.0
    } else {
        56.0
    }
}

/// Maps the authored research coordinates into the available field.
pub(super) struct TreeLayout {
    origin_x: f32,
    origin_y: f32,
    scale: f32,
    center: (f32, f32),
    zoom: f32,
    pan: (f32, f32),
}

impl TreeLayout {
    pub fn fit(nodes: &[ResearchNode], area: Rect) -> Self {
        Self::fit_with_min_scale(nodes, area, 1.0)
    }

    pub fn fit_with_min_scale(nodes: &[ResearchNode], area: Rect, min_scale: f32) -> Self {
        let mut min = (f32::MAX, f32::MAX);
        let mut max = (f32::MIN, f32::MIN);
        for node in nodes {
            min = (min.0.min(node.position.0), min.1.min(node.position.1));
            max = (max.0.max(node.position.0), max.1.max(node.position.1));
        }
        if nodes.is_empty() {
            return Self {
                origin_x: area.x + area.w * 0.5,
                origin_y: area.y + area.h * 0.5,
                scale: GRID_SCALE,
                center: (area.x + area.w * 0.5, area.y + area.h * 0.5),
                zoom: 1.0,
                pan: (0.0, 0.0),
            };
        }

        let margin = MAX_NODE_RADIUS + 24.0;
        let span = ((max.0 - min.0).max(0.001), (max.1 - min.1).max(0.001));
        let scale = ((area.w - margin * 2.0) / span.0)
            .min((area.h - margin * 2.0) / span.1)
            .clamp(min_scale, GRID_SCALE);
        let used = (span.0 * scale, span.1 * scale);
        Self {
            origin_x: area.x + (area.w - used.0) * 0.5 - min.0 * scale,
            origin_y: area.y + (area.h - used.1) * 0.5 + max.1 * scale,
            scale,
            center: (area.x + area.w * 0.5, area.y + area.h * 0.5),
            zoom: 1.0,
            pan: (0.0, 0.0),
        }
    }

    pub fn with_view(mut self, zoom: f32, pan: (f32, f32)) -> Self {
        self.zoom = zoom;
        self.pan = pan;
        self
    }

    pub fn to_screen(&self, position: (f32, f32)) -> (f32, f32) {
        let x = self.origin_x + position.0 * self.scale;
        let y = self.origin_y - position.1 * self.scale;
        (
            self.center.0 + (x - self.center.0) * self.zoom + self.pan.0,
            self.center.1 + (y - self.center.1) * self.zoom + self.pan.1,
        )
    }

    pub fn node_radius(&self) -> f32 {
        ((MAX_NODE_RADIUS * self.scale / GRID_SCALE).clamp(MIN_NODE_RADIUS, MAX_NODE_RADIUS)
            * self.zoom)
            .min(MAX_NODE_RADIUS)
    }
}

#[cfg(test)]
mod tests;
