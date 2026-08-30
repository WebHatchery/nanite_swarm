//! Settings menu screen

use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::{draw_ui_text, stepper_row, toggle_row, truncate_text_to_width};

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

    let mobile = screen_w < 600.0;
    let panel_w = if mobile {
        (screen_w - 24.0).max(240.0)
    } else {
        320.0
    };
    let panel_h = if mobile { 324.0 } else { 400.0 };
    let panel_y = if mobile { 84.0 } else { screen_h * 0.3 };
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

    let motion_row = Rect::new(
        display_x + 16.0,
        panel_y + 54.0 + (row_h + row_gap) * 2.0,
        row_w,
        row_h,
    );
    toggle_row(motion_row, "Reduced motion", &mut settings.reduced_motion);
    draw_ui_text(
        &truncate_text_to_width(
            "Stops pulses and impact particles; rules still run.",
            row_w,
            10.0,
        ),
        display_x + 16.0,
        motion_row.y + row_h + 12.0,
        10.0,
        Colors::TEXT_DIM,
    );

    let key_row = Rect::new(display_x + 16.0, motion_row.y + row_h + 30.0, row_w, row_h);
    if draw_button_sized(
        key_row.x,
        key_row.y,
        key_row.w,
        key_row.h,
        &format!(
            "Pause key: {} (tap to cycle)",
            settings
                .key_bindings
                .get("pause")
                .map(String::as_str)
                .unwrap_or("Unassigned")
        ),
    ) {
        let next = match settings.key_bindings.get("pause").map(String::as_str) {
            Some("Space") => "P",
            Some("P") => "Enter",
            _ => "Space",
        };
        settings
            .key_bindings
            .insert("pause".to_string(), next.to_string());
    }

    let speed_row = Rect::new(display_x + 16.0, key_row.y + row_h + 30.0, row_w, row_h);
    let scales = ["0.5x", "1x", "2x", "4x", "8x MAX"];
    let speed_index = settings.default_speed.clamp(0, 4) as usize;
    if stepper_row(speed_row, "Initial game speed", scales[speed_index]) != 0 {
        settings.default_speed = if settings.default_speed >= 4 {
            0
        } else {
            settings.default_speed + 1
        };
    }

    if mobile {
        draw_ui_text(
            "Tap Back to return.",
            20.0,
            screen_h - 32.0,
            11.0,
            Colors::TEXT_DIM,
        );
        draw_ui_text(
            &truncate_text_to_width(
                "Initial speed is separate from each world's pause state.",
                screen_w - 40.0,
                10.0,
            ),
            20.0,
            screen_h - 16.0,
            10.0,
            Colors::TEXT_DIM,
        );
    } else {
        draw_ui_text(
            "Tap Back to return. Initial speed is separate from each world's pause state.",
            20.0,
            screen_h - 20.0,
            Dimensions::FONT_SIZE_SMALL,
            Colors::TEXT_DIM,
        );
    }

    if is_key_pressed(KeyCode::Escape) {
        return SettingsAction::Back;
    }

    SettingsAction::None
}
