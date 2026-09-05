//! Pure text formatting and lookup helpers shared across the planetary view

use crate::ui::Colors;
use macroquad::prelude::*;
pub(super) use macroquad_toolkit::ui::truncate_text_to_width as fit_text_to_width;

pub(super) fn hash01(seed: u32) -> f32 {
    let noise = (seed as f32 * 12.9898).sin() * 43_758.547;
    noise.fract().abs()
}

pub(super) fn format_hours_minutes(seconds: f32) -> (i32, i32) {
    let total = seconds.max(0.0) as i32;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    (hours, minutes)
}

pub(super) fn keycode_from_hotkey(hotkey: char) -> Option<KeyCode> {
    match hotkey {
        '1' => Some(KeyCode::Key1),
        '2' => Some(KeyCode::Key2),
        '3' => Some(KeyCode::Key3),
        '4' => Some(KeyCode::Key4),
        '5' => Some(KeyCode::Key5),
        '6' => Some(KeyCode::Key6),
        '7' => Some(KeyCode::Key7),
        '8' => Some(KeyCode::Key8),
        '9' => Some(KeyCode::Key9),
        _ => None,
    }
}

pub(super) fn format_power_delta(delta: f32) -> String {
    if delta > 0.0 {
        format!("+{:.0}/s", delta)
    } else if delta < 0.0 {
        format!("{:.0}/s", delta)
    } else {
        "0/s".to_string()
    }
}

pub(super) fn dust_status(dust: f32) -> (&'static str, Color) {
    if dust >= 100.0 {
        ("Stalled", Colors::ERROR)
    } else if dust >= 75.0 {
        ("Power leakage", Colors::WARNING)
    } else if dust >= 50.0 {
        ("Drones slowed", Colors::WARNING)
    } else if dust >= 25.0 {
        ("Efficiency -10%", Colors::TEXT_DIM)
    } else {
        ("Clean", Colors::SUCCESS)
    }
}
