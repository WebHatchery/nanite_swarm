//! Desktop help and controls overlay for the command bar.

use crate::data::UiTheme;
use crate::ui::draw_hud_panel;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub(super) fn draw(theme: &UiTheme, screen_w: f32, text: Color, dim: Color) {
    let help_w = 360.0;
    let help_h = 200.0;
    let help_x = screen_w - help_w - 20.0;
    let help_y = 90.0;
    draw_hud_panel(
        theme,
        Rect::new(help_x, help_y, help_w, help_h),
        Some("HELP & CONTROLS"),
    );
    draw_ui_text(
        "Tap grid: build or inspect",
        help_x + 16.0,
        help_y + 55.0,
        14.0,
        text,
    );
    draw_ui_text(
        "Drag grid: pan the map",
        help_x + 16.0,
        help_y + 75.0,
        14.0,
        text,
    );
    draw_ui_text(
        "Tap - / + or pinch grid: zoom the map",
        help_x + 16.0,
        help_y + 95.0,
        14.0,
        text,
    );
    draw_ui_text(
        "Tap inspector: harvest, filter, or sell",
        help_x + 16.0,
        help_y + 115.0,
        14.0,
        text,
    );
    draw_ui_text(
        "Tap CANCEL: leave build or demolish mode",
        help_x + 16.0,
        help_y + 135.0,
        14.0,
        text,
    );
    draw_ui_text(
        "Top buttons open Research, Ship, Map, Menu",
        help_x + 16.0,
        help_y + 155.0,
        14.0,
        text,
    );
    draw_ui_text(
        "Tap HELP again to close",
        help_x + 16.0,
        help_y + 175.0,
        14.0,
        dim,
    );
}
