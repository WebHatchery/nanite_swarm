//! Settings menu screen

use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::{draw_ui_text, stepper_row, toggle_row};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsAction {
    None,
    Back,
}

/// Render the settings menu and return any action taken
pub fn render_settings_menu(settings: &mut GameSettings) -> SettingsAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let header_height = 72.0;

    draw_panel(0.0, 0.0, screen_w, header_height);
    draw_ui_text("Settings", 18.0, 30.0, 18.0, Colors::PRIMARY);
    draw_ui_text("Display", 18.0, 52.0, 12.0, Colors::TEXT_DIM);

    if draw_button_sized(screen_w - 110.0, 18.0, 80.0, 34.0, "Back") {
        return SettingsAction::Back;
    }

    let panel_w = 320.0;
    let panel_h = 240.0;
    let panel_y = screen_h * 0.3;
    let display_x = screen_w * 0.5 - panel_w * 0.5;
    let row_w = panel_w - 32.0;
    let row_h = 30.0;
    let row_gap = 18.0;

    draw_panel(display_x, panel_y, panel_w, panel_h);
    draw_ui_text(
        "Display",
        display_x + 16.0,
        panel_y + 28.0,
        16.0,
        Colors::PRIMARY,
    );

    let scale_row = Rect::new(display_x + 16.0, panel_y + 54.0, row_w, row_h);
    let scale_step = stepper_row(
        scale_row,
        "UI Scale",
        &format!("{:.2}x", settings.ui_text_scale),
    );
    if scale_step != 0 {
        settings.ui_text_scale =
            (settings.ui_text_scale + scale_step as f32 * 0.05).clamp(0.75, 1.5);
    }

    let fps_row = Rect::new(
        display_x + 16.0,
        panel_y + 54.0 + row_h + row_gap,
        row_w,
        row_h,
    );
    toggle_row(fps_row, "Show FPS", &mut settings.show_fps);

    draw_ui_text(
        "Tap Back to return",
        20.0,
        screen_h - 20.0,
        Dimensions::FONT_SIZE_SMALL,
        Colors::TEXT_DIM,
    );

    if is_key_pressed(KeyCode::Escape) {
        return SettingsAction::Back;
    }

    SettingsAction::None
}
