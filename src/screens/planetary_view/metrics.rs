//! Screen layout metrics and screen/grid coordinate conversions

use crate::data::UiTheme;
use crate::engine::GridPos;
use crate::state::{Camera, DesktopPanel, MobilePanel};
use macroquad::prelude::{vec2, Rect};

#[derive(Debug, Clone, Copy)]
pub(super) struct HudMetrics {
    /// Tile size on screen, i.e. the themed size scaled by the camera.
    pub(super) tile_size: f32,
    pan_x: f32,
    pan_y: f32,
    pub(super) top_bar_height: f32,
    pub(super) left_panel_width: f32,
    pub(super) right_panel_width: f32,
    pub(super) bottom_bar_height: f32,
    pub(super) panel_gap: f32,
    pub(super) panel_padding: f32,
    pub(super) build_row_height: f32,
    pub(super) mobile: bool,
}

impl HudMetrics {
    #[allow(dead_code)]
    pub(super) fn for_screen(
        theme: &UiTheme,
        screen_w: f32,
        screen_h: f32,
        camera: Camera,
        focus_mode: bool,
    ) -> Self {
        let compact_height = screen_h < 760.0;
        let compact_width = screen_w < 1260.0;
        let mobile = screen_w < 600.0;
        Self {
            tile_size: theme.layout.tile_size * camera.zoom,
            pan_x: camera.pan_x,
            pan_y: camera.pan_y,
            top_bar_height: if compact_height {
                76.0
            } else {
                theme.layout.top_bar_height
            },
            left_panel_width: if focus_mode || mobile {
                0.0
            } else if compact_width {
                260.0
            } else {
                theme.layout.left_panel_width
            },
            right_panel_width: if focus_mode || mobile {
                0.0
            } else if compact_width {
                292.0
            } else {
                theme.layout.right_panel_width
            },
            bottom_bar_height: if compact_height {
                64.0
            } else {
                theme.layout.bottom_bar_height
            },
            panel_gap: theme.layout.panel_gap,
            panel_padding: theme.layout.panel_padding,
            build_row_height: if compact_height {
                68.0
            } else {
                theme.layout.build_row_height
            },
            mobile,
        }
    }

    /// Compute the world-facing layout. Desktop drawers consume only the edge
    /// they are opened on; with no drawer open the map owns the full width.
    pub(super) fn for_screen_with_panel(
        theme: &UiTheme,
        screen_w: f32,
        screen_h: f32,
        camera: Camera,
        focus_mode: bool,
        desktop_panel: DesktopPanel,
    ) -> Self {
        let compact_height = screen_h < 760.0;
        let compact_width = screen_w < 1260.0;
        let mobile = screen_w < 600.0;
        let active_panel = if focus_mode {
            DesktopPanel::None
        } else {
            desktop_panel
        };
        Self {
            tile_size: theme.layout.tile_size * camera.zoom,
            pan_x: camera.pan_x,
            pan_y: camera.pan_y,
            top_bar_height: if mobile {
                if compact_height {
                    76.0
                } else {
                    theme.layout.top_bar_height
                }
            } else if compact_height {
                58.0
            } else {
                60.0
            },
            left_panel_width: if mobile || active_panel != DesktopPanel::Build {
                0.0
            } else if compact_width {
                260.0
            } else {
                theme.layout.left_panel_width
            },
            right_panel_width: if mobile || active_panel != DesktopPanel::Info {
                0.0
            } else if compact_width {
                292.0
            } else {
                theme.layout.right_panel_width
            },
            bottom_bar_height: if mobile {
                if compact_height {
                    64.0
                } else {
                    theme.layout.bottom_bar_height
                }
            } else if compact_height {
                54.0
            } else {
                58.0
            },
            panel_gap: theme.layout.panel_gap,
            panel_padding: theme.layout.panel_padding,
            build_row_height: if compact_height {
                68.0
            } else {
                theme.layout.build_row_height
            },
            mobile,
        }
    }

    pub(super) fn mobile_build_panel(self, screen_w: f32) -> Self {
        Self {
            left_panel_width: (screen_w - 5.0).max(0.0),
            right_panel_width: 0.0,
            mobile: false,
            ..self
        }
    }

    pub(super) fn mobile_info_panel(self, screen_w: f32) -> Self {
        Self {
            left_panel_width: 0.0,
            right_panel_width: (screen_w - 20.0).max(0.0),
            mobile: false,
            ..self
        }
    }

    /// Where the grid's top-left corner sits with the camera at rest. Zooming
    /// is measured from here, so it has to stay free of the pan.
    pub(super) fn base_offset_x(&self) -> f32 {
        self.left_panel_width + self.panel_gap * 2.0
    }

    pub(super) fn base_offset_y(&self) -> f32 {
        self.top_bar_height + self.panel_gap
    }

    pub(super) fn grid_offset_x(&self) -> f32 {
        self.base_offset_x() + self.pan_x
    }

    pub(super) fn grid_offset_y(&self) -> f32 {
        self.base_offset_y() + self.pan_y
    }

    /// The area the grid is drawn into, between the panels.
    pub(super) fn viewport(&self, screen_w: f32, screen_h: f32) -> (f32, f32) {
        (
            (screen_w - self.right_panel_width - self.base_offset_x()).max(0.0),
            (screen_h - self.bottom_bar_height - self.base_offset_y()).max(0.0),
        )
    }
}

#[cfg(test)]
mod tests;

/// Convert screen position to grid position
pub(super) fn screen_to_grid(screen_x: f32, screen_y: f32, metrics: HudMetrics) -> Option<GridPos> {
    let grid_x = ((screen_x - metrics.grid_offset_x()) / metrics.tile_size).floor() as i32;
    let grid_y = ((screen_y - metrics.grid_offset_y()) / metrics.tile_size).floor() as i32;

    if grid_x >= 0 && grid_y >= 0 {
        Some(GridPos::new(grid_x, grid_y))
    } else {
        None
    }
}

/// Convert grid position to screen position
pub(super) fn grid_to_screen(pos: GridPos, metrics: HudMetrics) -> (f32, f32) {
    (
        metrics.grid_offset_x() + pos.x as f32 * metrics.tile_size,
        metrics.grid_offset_y() + pos.y as f32 * metrics.tile_size,
    )
}

#[allow(dead_code)]
pub(super) fn is_cursor_over_ui(
    screen_x: f32,
    screen_y: f32,
    screen_w: f32,
    screen_h: f32,
    metrics: HudMetrics,
    mobile_panel: MobilePanel,
) -> bool {
    is_cursor_over_ui_with_help(
        screen_x,
        screen_y,
        screen_w,
        screen_h,
        metrics,
        mobile_panel,
        false,
    )
}

pub(super) fn is_cursor_over_ui_with_help(
    screen_x: f32,
    screen_y: f32,
    screen_w: f32,
    screen_h: f32,
    metrics: HudMetrics,
    mobile_panel: MobilePanel,
    show_help: bool,
) -> bool {
    if screen_y <= metrics.top_bar_height || screen_y >= screen_h - metrics.bottom_bar_height {
        return true;
    }
    if screen_x <= metrics.left_panel_width {
        return true;
    }
    if screen_x >= screen_w - metrics.right_panel_width {
        return true;
    }
    if show_help && !metrics.mobile {
        let help_rect = Rect::new(screen_w - 380.0, 90.0, 360.0, 200.0);
        if help_rect.contains(vec2(screen_x, screen_y)) {
            return true;
        }
    }
    if metrics.mobile {
        let panel_rect = Rect::new(
            8.0,
            metrics.top_bar_height + 8.0,
            (screen_w - 16.0).max(0.0),
            (screen_h - metrics.top_bar_height - metrics.bottom_bar_height - 16.0).max(0.0),
        );
        if mobile_panel != MobilePanel::Map {
            return panel_rect.contains(vec2(screen_x, screen_y));
        }
        let tutorial_rect = Rect::new(
            panel_rect.x,
            panel_rect.y,
            panel_rect.w,
            super::hud::MOBILE_TUTORIAL_HEIGHT,
        );
        if tutorial_rect.contains(vec2(screen_x, screen_y)) {
            return true;
        }
    }
    false
}
