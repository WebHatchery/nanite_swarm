//! Display and input settings tab.

use super::SettingsLayout;
use crate::ui::{draw_button_sized, Colors};
use macroquad::prelude::Rect;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::{draw_ui_text, stepper_row, toggle_row, truncate_text_to_width};

pub(super) fn draw(
    settings: &mut GameSettings,
    screen_w: f32,
    display_x: f32,
    row_w: f32,
    layout: SettingsLayout,
) {
    let compact_mobile = screen_w < 340.0;
    let content_y = layout.panel_y + if compact_mobile { 68.0 } else { 78.0 };
    let row_h = layout.row_h;
    let scale_row = Rect::new(display_x + 16.0, content_y, row_w, row_h);
    let scale_step = stepper_row(
        scale_row,
        "UI Scale",
        &format!("{:.2}x", settings.ui_text_scale),
    );
    if scale_step != 0 {
        settings.ui_text_scale =
            (settings.ui_text_scale + scale_step as f32 * 0.05).clamp(0.75, 1.5);
    }

    let fps_gap = if compact_mobile { 4.0 } else { layout.row_gap };
    let fps_row = Rect::new(display_x + 16.0, content_y + row_h + fps_gap, row_w, row_h);
    if compact_mobile {
        toggle_row(fps_row, "Show FPS", &mut settings.show_fps);
    } else {
        let column_gap = 8.0;
        let fps_w = (row_w * 0.36).clamp(88.0, 120.0);
        toggle_row(
            Rect::new(fps_row.x, fps_row.y, fps_w, fps_row.h),
            "FPS",
            &mut settings.show_fps,
        );
        toggle_row(
            Rect::new(
                fps_row.x + fps_w + column_gap,
                fps_row.y,
                row_w - fps_w - column_gap,
                fps_row.h,
            ),
            "Fullscreen",
            &mut settings.fullscreen,
        );
    }

    let motion_row = Rect::new(
        display_x + 16.0,
        content_y + (row_h + fps_gap) * 2.0,
        row_w,
        row_h,
    );
    toggle_row(motion_row, "Reduced motion", &mut settings.reduced_motion);
    if !compact_mobile {
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
    }
    let section_gap = if compact_mobile {
        4.0
    } else {
        layout.section_gap
    };

    let key_row = Rect::new(
        display_x + 16.0,
        motion_row.y + row_h + section_gap,
        row_w,
        row_h,
    );
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

    let speed_row = Rect::new(
        display_x + 16.0,
        key_row.y + row_h + section_gap,
        row_w,
        row_h,
    );
    let scales = ["0.5x", "1x", "2x", "4x", "8x MAX"];
    let speed_index = settings.default_speed.clamp(0, 4) as usize;
    if stepper_row(speed_row, "Initial game speed", scales[speed_index]) != 0 {
        settings.default_speed = if settings.default_speed >= 4 {
            0
        } else {
            settings.default_speed + 1
        };
    }
    if compact_mobile {
        toggle_row(
            Rect::new(speed_row.x, speed_row.y + row_h + 4.0, row_w, row_h),
            "Fullscreen",
            &mut settings.fullscreen,
        );
    }
}
