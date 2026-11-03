//! Settings menu screen

use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

mod audio;
mod display;

#[cfg(test)]
use audio::step_volume;

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
        SettingsTab::Display => display::draw(settings, screen_w, display_x, row_w, layout),
        SettingsTab::Audio => {
            if audio::draw(
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

#[cfg(test)]
mod tests;
