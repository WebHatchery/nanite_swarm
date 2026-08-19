use macroquad::prelude::set_fullscreen;
use macroquad_toolkit::settings::GameSettings;

/// Push display settings only when they changed. Reapplying windowed mode on
/// every settings frame can shrink the client area on Windows.
pub fn apply_display_settings(settings: &GameSettings, previous: Option<&GameSettings>) {
    let scale_changed = previous.is_none_or(|old| old.ui_text_scale != settings.ui_text_scale);
    if scale_changed {
        macroquad_toolkit::ui::set_ui_text_scale(settings.ui_text_scale);
    }

    let fullscreen_changed = match previous {
        Some(old) => old.fullscreen != settings.fullscreen,
        None => settings.fullscreen,
    };
    if fullscreen_changed {
        set_fullscreen(settings.fullscreen);
    }
}
