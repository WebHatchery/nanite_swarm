//! The launch, drawn. A pure view over `LaunchSequence`: it reads the beat and
//! how far into it, and returns whether the player asked to cut it short.

use crate::state::{LaunchBeat, LaunchSequence};
use crate::ui::{color_from_rgba, draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::math::lerp;
use macroquad_toolkit::ui::draw_ui_text;

/// What the player did to the launch sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchAction {
    None,
    /// Cut to the end.
    Skip,
}

/// Render one frame of the sequence.
pub fn render_launch_view(sequence: &LaunchSequence, arrival_line: &str) -> LaunchAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let beat = sequence.beat();
    let fraction = sequence.beat_fraction();

    draw_starfield(screen_w, screen_h);

    let planets = &crate::data::game_data().planets;
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
    draw_world(origin, origin_radius, origin_color);
    draw_world(target, target_radius, target_color);

    match beat {
        Some(LaunchBeat::Countdown) => {
            draw_ship(origin + vec2(0.0, -origin_radius - 14.0), 0.0);
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
            draw_exhaust(position, origin + vec2(0.0, -origin_radius), origin_color);
            draw_ship(position, 0.0);
        }
        Some(LaunchBeat::Transit) => {
            let position = origin.lerp(target, fraction);
            draw_trail(origin, position);
            draw_ship(
                position,
                (target - origin).to_angle() + std::f32::consts::FRAC_PI_2,
            );
        }
        Some(LaunchBeat::Arrival) | None => {}
    }

    if let Some(beat) = beat {
        draw_caption(sequence, beat, arrival_line, screen_w, screen_h);
    }

    let hint = LaunchSequence::skip_hint();
    if !hint.is_empty() {
        draw_ui_text(hint, 18.0, screen_h - 18.0, 12.0, Colors::TEXT_DIM);
    }

    let skipped = is_mouse_button_pressed(MouseButton::Left)
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
    let band_y = screen_h * 0.60;
    let fade = if beat == LaunchBeat::Arrival {
        (sequence.beat_fraction() * 3.0).min(1.0)
    } else {
        1.0
    };
    let band_h = if beat == LaunchBeat::Arrival {
        96.0
    } else {
        62.0
    };
    draw_panel(0.0, band_y - 34.0, screen_w, band_h);

    if beat == LaunchBeat::Arrival {
        let name = crate::data::game_data().planets[sequence.target()]
            .name
            .as_str();
        draw_centered(
            name,
            screen_w,
            band_y,
            22.0,
            Color::new(
                Colors::PRIMARY.r,
                Colors::PRIMARY.g,
                Colors::PRIMARY.b,
                fade,
            ),
        );
        draw_centered(
            &sequence.line(arrival_line),
            screen_w,
            band_y + 30.0,
            14.0,
            Color::new(Colors::TEXT.r, Colors::TEXT.g, Colors::TEXT.b, fade),
        );
    } else {
        draw_centered(
            &sequence.line(arrival_line),
            screen_w,
            band_y + 4.0,
            14.0,
            Colors::TEXT,
        );
    }
}

fn draw_centered(text: &str, screen_w: f32, y: f32, size: f32, color: Color) {
    let width = measure_text(text, None, size as u16, 1.0).width;
    draw_ui_text(text, (screen_w - width) * 0.5, y, size, color);
}

/// A world as a lit disc with a thin limb, seen from close enough that most of
/// it is off screen.
fn draw_world(centre: Vec2, radius: f32, color: Color) {
    draw_circle(
        centre.x,
        centre.y,
        radius,
        Color::new(color.r * 0.35, color.g * 0.35, color.b * 0.4, 1.0),
    );
    draw_circle_lines(centre.x, centre.y, radius, 2.0, color);
    draw_circle_lines(
        centre.x,
        centre.y,
        radius * 1.06,
        1.0,
        Color::new(color.r, color.g, color.b, 0.25),
    );
}

/// The ship: a wedge, pointing along `angle` (zero is straight up).
fn draw_ship(position: Vec2, angle: f32) {
    let (sin, cos) = angle.sin_cos();
    let rotate = |offset: Vec2| {
        vec2(
            position.x + offset.x * cos - offset.y * sin,
            position.y + offset.x * sin + offset.y * cos,
        )
    };
    let nose = rotate(vec2(0.0, -24.0));
    let left = rotate(vec2(-12.0, 15.0));
    let right = rotate(vec2(12.0, 15.0));
    draw_triangle(nose, left, right, Colors::PRIMARY);
    draw_triangle_lines(nose, left, right, 1.5, Color::new(0.8, 0.95, 1.0, 1.0));
}

/// Everything the ship is throwing away to leave, drawn as a widening plume
/// back to the ground it left.
fn draw_exhaust(ship: Vec2, pad: Vec2, world_color: Color) {
    let span = (pad.y - ship.y).max(1.0);
    for step in 0..18 {
        let t = step as f32 / 17.0;
        let y = ship.y + span * t;
        let width = 5.0 + t * 26.0;
        let alpha = (1.0 - t) * 0.5;
        draw_rectangle(
            ship.x - width * 0.5,
            y,
            width,
            span / 18.0 + 1.0,
            Color::new(1.0, 0.6 + world_color.r * 0.2, 0.3, alpha),
        );
    }
}

/// The line the ship has already covered between worlds.
fn draw_trail(from: Vec2, to: Vec2) {
    draw_line(
        from.x,
        from.y,
        to.x,
        to.y,
        1.5,
        Color::new(0.4, 0.8, 1.0, 0.28),
    );
}

/// The same deterministic scatter every frame, so the sky does not boil.
fn draw_starfield(screen_w: f32, screen_h: f32) {
    for i in 0..180u32 {
        let x = (i as f32 * 71.3).sin().abs() * screen_w;
        let y = (i as f32 * 41.9).cos().abs() * screen_h;
        draw_circle(
            x,
            y,
            0.6 + (i % 3) as f32 * 0.4,
            Color::new(0.7, 0.85, 1.0, 0.10 + (i % 5) as f32 * 0.03),
        );
    }
}
