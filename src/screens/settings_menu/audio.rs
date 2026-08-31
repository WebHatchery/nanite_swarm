//! Audio settings tab and volume-step behavior.

use crate::ui::{draw_button_sized, Colors};
use macroquad::prelude::Rect;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::{draw_ui_text, stepper_row};

pub(super) fn draw(
    settings: &mut GameSettings,
    display_x: f32,
    panel_y: f32,
    row_w: f32,
    row_h: f32,
    row_gap: f32,
) -> bool {
    draw_ui_text(
        "Tap +/- to adjust. Changes apply immediately.",
        display_x + 16.0,
        panel_y + 78.0,
        10.0,
        Colors::TEXT_DIM,
    );
    let content_y = panel_y + 90.0;
    adjust_volume_row(
        Rect::new(display_x + 16.0, content_y, row_w, row_h),
        "Master volume",
        &mut settings.master_volume,
    );
    adjust_volume_row(
        Rect::new(display_x + 16.0, content_y + row_h + row_gap, row_w, row_h),
        "Sound effects",
        &mut settings.sfx_volume,
    );
    adjust_volume_row(
        Rect::new(
            display_x + 16.0,
            content_y + (row_h + row_gap) * 2.0,
            row_w,
            row_h,
        ),
        "Music",
        &mut settings.music_volume,
    );

    let test_y = content_y + (row_h + row_gap) * 3.0;
    draw_button_sized(display_x + 16.0, test_y, row_w, row_h, "Test signal")
}

fn adjust_volume_row(rect: Rect, label: &str, volume: &mut f32) {
    let direction = stepper_row(rect, label, &format!("{:.0}%", *volume * 100.0));
    if direction != 0 {
        *volume = step_volume(*volume, direction);
    }
}

pub(super) fn step_volume(volume: f32, direction: i32) -> f32 {
    let current = (volume * 10.0).round() as i32;
    ((current + direction).clamp(0, 10) as f32) / 10.0
}
