//! Planetary backdrop effects and late-Core ambience.

use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::math::{lerp, pulse01_at};

use super::super::format::hash01;
use super::super::metrics::HudMetrics;
use super::core::draw_core_stage_ambient;

pub(super) fn draw_core_stage_ambient_for_map(state: &PlanetState, metrics: HudMetrics, time: f32) {
    draw_core_stage_ambient(state, metrics, time);
}

/// Project the final Core stage into the planetary backdrop. The map ring is
/// the local interaction anchor; this quieter ring belongs to the planet that
/// the Core has begun to encircle.
pub(super) fn draw_core_stage_planetary_backdrop(
    state: &PlanetState,
    screen_w: f32,
    screen_h: f32,
    time: f32,
) {
    if state.core_stage_index() < 4 {
        return;
    }

    let center = vec2(screen_w * 0.82, screen_h * 0.85);
    let diameter = (screen_w.min(screen_h) * 0.72).clamp(300.0, 760.0);
    let height = (diameter * 0.30).clamp(96.0, 180.0);
    let pulse = if macroquad_toolkit::settings::reduced_motion_enabled() {
        0.0
    } else {
        pulse01_at(time as f64, 2.4)
    };
    let rotation = -12.0 + (time * 0.04).sin() * pulse * 1.5;

    draw_ellipse_lines(
        center.x,
        center.y,
        diameter,
        height,
        rotation,
        2.0,
        with_alpha(Colors::PRIMARY_SOFT, 0.28 + pulse * 0.08),
    );
    draw_ellipse_lines(
        center.x,
        center.y,
        diameter * 0.92,
        height * 0.60,
        rotation + 3.0,
        1.0,
        with_alpha(Colors::ACCENT, 0.18 + pulse * 0.06),
    );

    for (index, angle) in [0.35, 1.95, 3.55, 5.15].into_iter().enumerate() {
        let position = ellipse_point(center, diameter * 0.5, height * 0.5, angle, rotation);
        let color = if index % 2 == 0 {
            Colors::PRIMARY
        } else {
            Colors::ACCENT
        };
        draw_circle(
            position.x,
            position.y,
            3.0,
            with_alpha(Colors::BACKGROUND, 0.82),
        );
        draw_circle(position.x, position.y, 1.8 + pulse * 0.7, color);
    }
}

fn ellipse_point(center: Vec2, radius_x: f32, radius_y: f32, angle: f32, rotation: f32) -> Vec2 {
    let angle = angle + rotation.to_radians();
    vec2(
        center.x + angle.cos() * radius_x,
        center.y + angle.sin() * radius_y,
    )
}

pub(super) fn draw_planetary_background(
    screen_w: f32,
    screen_h: f32,
    time: f32,
    state: &PlanetState,
) {
    // Subtle star field
    for i in 0..80u32 {
        let star_x = hash01(i) * screen_w;
        let star_y = hash01(i + 17) * screen_h;
        let twinkle = lerp(0.5, 1.0, pulse01_at(time as f64 + 2.0 * i as f64, 0.5));
        let color = Color::new(0.6, 0.7, 0.8, 0.15 + twinkle * 0.2);
        draw_circle(star_x, star_y, 1.0 + hash01(i + 31), color);
    }

    // Planet glow
    let planet_x = screen_w * 0.82;
    let planet_y = screen_h * 0.85;
    let glow = lerp(0.25, 0.35, pulse01_at(time as f64, 0.6));
    draw_circle(planet_x, planet_y, 220.0, Color::new(0.0, 0.2, 0.35, 0.12));
    draw_circle(planet_x, planet_y, 170.0, Color::new(0.0, 0.3, 0.45, glow));
    draw_circle(planet_x, planet_y, 120.0, Color::new(0.0, 0.25, 0.4, 0.25));
    let planet = state.data().planet(state.planet_index);
    let atmosphere = if state.acid_strength() > 0.0 {
        Color::new(0.8, 0.28, 0.14, 0.16)
    } else if state.freeze_strength() > 0.0 {
        Color::new(0.35, 0.65, 1.0, 0.16)
    } else {
        Color::new(planet.color[0], planet.color[1], planet.color[2], 0.10)
    };
    draw_circle_lines(planet_x, planet_y, 132.0, 3.0, atmosphere);
    draw_circle_lines(planet_x, planet_y, 184.0, 1.0, with_alpha(atmosphere, 0.55));
}

pub(super) fn draw_collapse_shake(state: &PlanetState, screen_w: f32, screen_h: f32, time: f32) {
    if state.collapse_notice_timer <= 0.0 || macroquad_toolkit::settings::reduced_motion_enabled() {
        return;
    }
    let offset = (time * 48.0).sin() * 3.0;
    draw_rectangle_lines(
        offset,
        0.0,
        screen_w - offset * 2.0,
        screen_h,
        3.0,
        with_alpha(Colors::ERROR, 0.65),
    );
}
