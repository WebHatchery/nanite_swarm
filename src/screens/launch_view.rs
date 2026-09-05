//! The launch, drawn. A pure view over `LaunchSequence`: it reads the beat and
//! how far into it, and returns whether the player asked to cut it short.

use crate::state::{LaunchBeat, LaunchSequence, PlanetState};
use crate::ui::{color_from_rgba, draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::math::lerp;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text as measure_text, wrap_text_ex, Pointer};

mod art;

#[derive(Debug, Clone, Copy, PartialEq)]
struct CaptionGeometry {
    band_y: f32,
    band_h: f32,
    text_size: f32,
    max_width: f32,
    title_y: f32,
    line_start_y: f32,
    line_gap: f32,
}

/// What the player did to the launch sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchAction {
    None,
    /// Cut to the end.
    Skip,
}

/// Render one frame of the sequence.
pub fn render_launch_view(
    sequence: &LaunchSequence,
    arrival_line: &str,
    origin_state: Option<&PlanetState>,
) -> LaunchAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let beat = sequence.beat();
    let fraction = sequence.beat_fraction();

    art::draw_starfield(screen_w, screen_h);

    let planets = &sequence.data().planets;
    let origin_color = color_from_rgba(&planets[sequence.origin()].color);
    let target_color = color_from_rgba(&planets[sequence.target()].color);

    // The two worlds sit at opposite corners for the whole sequence, so the
    // ship's crossing reads as a crossing rather than as drift. Both centres
    // are off screen: what shows is a limb, close up.
    let origin = vec2(screen_w * 0.20, screen_h * 1.34);
    let target = vec2(screen_w * 0.72, screen_h * -0.18);

    let (origin_radius, target_radius) = match beat {
        // Arriving, the new world swells and the old one is behind you.
        Some(LaunchBeat::Arrival) => (
            screen_h * 0.42,
            lerp(screen_h * 0.36, screen_h * 0.62, fraction.min(1.0)),
        ),
        _ => (screen_h * 0.52, screen_h * 0.34),
    };
    art::draw_world(origin, origin_radius, origin_color);
    art::draw_world(target, target_radius, target_color);
    if matches!(beat, Some(LaunchBeat::Countdown | LaunchBeat::Ascent)) {
        if let Some(state) = origin_state {
            art::draw_departure_base(origin, origin_radius, state, fraction);
        }
    }

    match beat {
        Some(LaunchBeat::Countdown) => {
            art::draw_ship(origin + vec2(0.0, -origin_radius - 14.0), 0.0);
            let count = sequence.countdown_remaining().ceil().max(1.0) as i32;
            let text = format!("{}", count);
            let size = 84.0;
            let width = measure_text(&text, None, size as u16, 1.0).width;
            draw_ui_text(
                &text,
                (screen_w - width) * 0.5,
                screen_h * 0.42,
                size,
                Colors::PRIMARY,
            );
            draw_centered(
                "IGNITION IN",
                screen_w,
                screen_h * 0.42 - 46.0,
                14.0,
                Colors::TEXT_DIM,
            );
        }
        Some(LaunchBeat::Ascent) => {
            // Off the pad and accelerating: distance covered grows with the
            // square of the time, which is what makes it read as a launch.
            let climb = fraction * fraction;
            let position = origin + vec2(0.0, -origin_radius - 14.0 - climb * screen_h * 0.75);
            art::draw_exhaust(position, origin + vec2(0.0, -origin_radius), origin_color);
            art::draw_ship(position, 0.0);
        }
        Some(LaunchBeat::Transit) => {
            let position = origin.lerp(target, fraction);
            art::draw_trail(origin, position);
            art::draw_ship(
                position,
                (target - origin).to_angle() + std::f32::consts::FRAC_PI_2,
            );
        }
        Some(LaunchBeat::Arrival) | None => {}
    }

    if let Some(beat) = beat {
        draw_caption(sequence, beat, arrival_line, screen_w, screen_h);
    }

    let hint = sequence.skip_hint();
    if !hint.is_empty() {
        draw_ui_text(hint, 18.0, screen_h - 18.0, 12.0, Colors::TEXT_DIM);
    }

    let skipped = Pointer::read(|position| position).released
        || get_last_key_pressed().is_some()
        || sequence.is_finished();
    if skipped {
        LaunchAction::Skip
    } else {
        LaunchAction::None
    }
}

/// The line for this beat, on a band across the lower third. The arrival beat
/// gets the world's name above it, because that is the moment it matters.
fn draw_caption(
    sequence: &LaunchSequence,
    beat: LaunchBeat,
    arrival_line: &str,
    screen_w: f32,
    screen_h: f32,
) {
    let geometry = caption_geometry(screen_w, screen_h, beat == LaunchBeat::Arrival);
    let fade = if beat == LaunchBeat::Arrival {
        (sequence.beat_fraction() * 3.0).min(1.0)
    } else {
        1.0
    };
    draw_panel(0.0, geometry.band_y - 34.0, screen_w, geometry.band_h);

    if beat == LaunchBeat::Arrival {
        let name = sequence.data().planets[sequence.target()].name.as_str();
        draw_centered(
            name,
            screen_w,
            geometry.title_y,
            22.0,
            Color::new(
                Colors::PRIMARY.r,
                Colors::PRIMARY.g,
                Colors::PRIMARY.b,
                fade,
            ),
        );
        draw_caption_lines(
            &sequence.line(arrival_line),
            screen_w,
            geometry,
            Color::new(Colors::TEXT.r, Colors::TEXT.g, Colors::TEXT.b, fade),
        );
    } else {
        draw_caption_lines(
            &sequence.line(arrival_line),
            screen_w,
            geometry,
            Colors::TEXT,
        );
    }
}

fn draw_caption_lines(text: &str, screen_w: f32, geometry: CaptionGeometry, color: Color) {
    for (index, line) in wrap_text_ex(text, geometry.max_width, None, geometry.text_size)
        .into_iter()
        .enumerate()
    {
        draw_centered(
            &line,
            screen_w,
            geometry.line_start_y + index as f32 * geometry.line_gap,
            geometry.text_size,
            color,
        );
    }
}

fn caption_geometry(screen_w: f32, screen_h: f32, arrival: bool) -> CaptionGeometry {
    let mobile = screen_w < 600.0;
    let band_y = screen_h * 0.60;
    let band_h = if arrival {
        if mobile {
            128.0
        } else {
            96.0
        }
    } else if mobile {
        82.0
    } else {
        62.0
    };
    let text_size = if mobile { 12.0 } else { 14.0 };
    CaptionGeometry {
        band_y,
        band_h,
        text_size,
        max_width: (screen_w - 28.0).max(80.0),
        title_y: band_y - if mobile { 4.0 } else { 0.0 },
        line_start_y: if arrival {
            band_y + if mobile { 20.0 } else { 30.0 }
        } else {
            band_y + if mobile { -4.0 } else { 4.0 }
        },
        line_gap: text_size + 4.0,
    }
}

fn draw_centered(text: &str, screen_w: f32, y: f32, size: f32, color: Color) {
    let width = measure_text(text, None, size as u16, 1.0).width;
    draw_ui_text(text, (screen_w - width) * 0.5, y, size, color);
}

#[cfg(test)]
mod tests;
