//! Settings menu screen

use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::{draw_ui_text, stepper_row, toggle_row, truncate_text_to_width};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsAction {
    None,
    Back,
    PreviewAudio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Display,
    Audio,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SettingsLayout {
    panel_w: f32,
    panel_h: f32,
    panel_y: f32,
    row_h: f32,
    row_gap: f32,
    section_gap: f32,
    compact_mobile: bool,
    compact_desktop: bool,
}

/// Render the settings menu and return any action taken
pub fn render_settings_menu(
    settings: &mut GameSettings,
    selected_tab: &mut SettingsTab,
) -> SettingsAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let header_height = 72.0;

    draw_panel(0.0, 0.0, screen_w, header_height);
    draw_ui_text("Settings", 18.0, 30.0, 18.0, Colors::PRIMARY);
    draw_ui_text(
        "Tune the command shell.",
        18.0,
        52.0,
        12.0,
        Colors::TEXT_DIM,
    );

    if draw_button_sized(screen_w - 110.0, 18.0, 80.0, 34.0, "Back") {
        return SettingsAction::Back;
    }

    let mobile = screen_w < 600.0;
    let layout = settings_layout(screen_w, screen_h);
    let display_x = screen_w * 0.5 - layout.panel_w * 0.5;
    let row_w = layout.panel_w - 32.0;

    draw_panel(display_x, layout.panel_y, layout.panel_w, layout.panel_h);
    let tab_y = layout.panel_y + 40.0;
    let tab_gap = 8.0;
    let tab_w = (row_w - tab_gap) * 0.5;
    if draw_button_sized(display_x + 16.0, tab_y, tab_w, 26.0, "Display") {
        *selected_tab = SettingsTab::Display;
    }
    if draw_button_sized(
        display_x + 16.0 + tab_w + tab_gap,
        tab_y,
        tab_w,
        26.0,
        "Audio",
    ) {
        *selected_tab = SettingsTab::Audio;
    }
    draw_ui_text(
        match *selected_tab {
            SettingsTab::Display => "Display",
            SettingsTab::Audio => "Audio",
        },
        display_x + 16.0,
        layout.panel_y + 28.0,
        16.0,
        Colors::PRIMARY,
    );

    match *selected_tab {
        SettingsTab::Display => draw_display_settings(
            settings,
            display_x,
            layout.panel_y,
            row_w,
            layout.row_h,
            layout.row_gap,
            layout.section_gap,
        ),
        SettingsTab::Audio => {
            if draw_audio_settings(
                settings,
                display_x,
                layout.panel_y,
                row_w,
                layout.row_h,
                layout.row_gap,
            ) {
                return SettingsAction::PreviewAudio;
            }
        }
    }

    if mobile {
        let footer_offset = if layout.compact_mobile { 42.0 } else { 32.0 };
        let detail_offset = if layout.compact_mobile { 22.0 } else { 16.0 };
        draw_ui_text(
            "Tap Back to return.",
            20.0,
            screen_h - footer_offset,
            11.0,
            Colors::TEXT_DIM,
        );
        draw_ui_text(
            &truncate_text_to_width(
                match *selected_tab {
                    SettingsTab::Display => {
                        "Initial speed is separate from each world's pause state."
                    }
                    SettingsTab::Audio => "Volume changes are saved immediately.",
                },
                screen_w - 40.0,
                10.0,
            ),
            20.0,
            screen_h - detail_offset,
            10.0,
            Colors::TEXT_DIM,
        );
    } else {
        let footer = match *selected_tab {
            SettingsTab::Display => {
                "Tap Back to return. Initial speed is separate from each world's pause state."
            }
            SettingsTab::Audio => "Tap Back to return. Volume changes are saved immediately.",
        };
        draw_ui_text(
            footer,
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

fn settings_layout(screen_w: f32, screen_h: f32) -> SettingsLayout {
    let mobile = screen_w < 600.0;
    let compact_mobile = mobile && screen_h < 440.0;
    let compact_desktop = !mobile && screen_h < 600.0;
    let panel_w = if mobile {
        (screen_w - 24.0).max(240.0)
    } else {
        320.0
    };
    let panel_y = if compact_mobile || compact_desktop {
        76.0
    } else if mobile {
        84.0
    } else {
        screen_h * 0.3
    };
    let panel_h = if compact_mobile {
        (screen_h - panel_y - 16.0).max(280.0)
    } else if compact_desktop {
        (screen_h - panel_y - 32.0).clamp(280.0, 400.0)
    } else if mobile {
        332.0
    } else {
        400.0
    };

    SettingsLayout {
        panel_w,
        panel_h,
        panel_y,
        row_h: 30.0,
        row_gap: if compact_mobile {
            6.0
        } else if compact_desktop {
            8.0
        } else if mobile {
            12.0
        } else {
            18.0
        },
        section_gap: if compact_mobile || compact_desktop {
            18.0
        } else {
            30.0
        },
        compact_mobile,
        compact_desktop,
    }
}

fn draw_display_settings(
    settings: &mut GameSettings,
    display_x: f32,
    panel_y: f32,
    row_w: f32,
    row_h: f32,
    row_gap: f32,
    section_gap: f32,
) {
    let content_y = panel_y + 78.0;
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

    let fps_row = Rect::new(display_x + 16.0, content_y + row_h + row_gap, row_w, row_h);
    toggle_row(fps_row, "Show FPS", &mut settings.show_fps);

    let motion_row = Rect::new(
        display_x + 16.0,
        content_y + (row_h + row_gap) * 2.0,
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
}

fn draw_audio_settings(
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

fn step_volume(volume: f32, direction: i32) -> f32 {
    let current = (volume * 10.0).round() as i32;
    ((current + direction).clamp(0, 10) as f32) / 10.0
}

#[cfg(test)]
mod tests;
