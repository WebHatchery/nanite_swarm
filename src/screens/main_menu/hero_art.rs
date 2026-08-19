//! Procedural title-screen tableau: one world becoming a machine.

use crate::engine::ResourceType;
use crate::ui::{draw_resource_icon, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::draw_ui_text;

pub(super) fn draw(screen_w: f32, screen_h: f32, time: f32) {
    draw_space_field(screen_w, screen_h, time);

    let radius = (screen_w.min(screen_h) * 0.245).clamp(108.0, 186.0);
    let center = vec2(screen_w * 0.56, screen_h * 0.52);
    draw_orbits(center, radius, time);
    draw_machine_world(center, radius, time);
    draw_swarm_streams(center, radius, time);
    draw_seed_ship(center, radius, time);

    draw_ui_text(
        "PLANETARY FOUNDRY // ACTIVE",
        center.x - radius * 0.72,
        center.y + radius + 32.0,
        10.0,
        with_alpha(Colors::PRIMARY_SOFT, 0.72),
    );
}

fn draw_space_field(screen_w: f32, screen_h: f32, time: f32) {
    draw_rectangle(
        0.0,
        0.0,
        screen_w,
        screen_h,
        Color::new(0.012, 0.02, 0.027, 1.0),
    );
    draw_circle(
        screen_w * 0.55,
        screen_h * 0.54,
        screen_h * 0.54,
        Color::new(0.0, 0.18, 0.24, 0.055),
    );
    draw_circle(
        screen_w * 0.12,
        screen_h * 0.92,
        screen_h * 0.38,
        Color::new(0.0, 0.3, 0.34, 0.035),
    );

    for index in 0..74 {
        let x = hash01(index * 17 + 3) * screen_w;
        let y = hash01(index * 31 + 11) * screen_h;
        let pulse = 0.3 + (time * 1.7 + index as f32 * 0.61).sin().abs() * 0.55;
        let size = if index % 11 == 0 { 1.6 } else { 0.8 };
        draw_circle(x, y, size, with_alpha(Colors::PRIMARY_SOFT, pulse));
    }

    // A faint fabrication grid makes the negative space feel designed without
    // competing with the interactive panels that sit over it.
    let grid = 44.0;
    let offset = (time * 2.0) % grid;
    let grid_color = Color::new(0.0, 0.55, 0.68, 0.028);
    let mut x = -grid + offset;
    while x < screen_w {
        draw_line(x, 0.0, x, screen_h, 1.0, grid_color);
        x += grid;
    }
    let mut y = -grid + offset;
    while y < screen_h {
        draw_line(0.0, y, screen_w, y, 1.0, grid_color);
        y += grid;
    }
}

fn draw_orbits(center: Vec2, radius: f32, time: f32) {
    draw_ellipse_lines(
        center.x,
        center.y,
        radius * 1.42,
        radius * 0.52,
        -14.0,
        1.0,
        with_alpha(Colors::PRIMARY_SOFT, 0.25),
    );
    draw_ellipse_lines(
        center.x,
        center.y,
        radius * 1.18,
        radius * 0.34,
        24.0,
        1.0,
        with_alpha(Colors::ACCENT, 0.18),
    );

    for (index, resource) in [
        ResourceType::Minerals,
        ResourceType::Alloy,
        ResourceType::Components,
    ]
    .into_iter()
    .enumerate()
    {
        let angle = time * (0.12 + index as f32 * 0.025) + index as f32 * 2.1;
        let pos = orbit_position(center, radius * 1.42, radius * 0.52, angle, -14.0);
        let color = match resource {
            ResourceType::Minerals => Color::new(0.14, 0.7, 1.0, 1.0),
            ResourceType::Alloy => Color::new(0.85, 0.58, 0.3, 1.0),
            _ => Color::new(0.78, 0.55, 0.95, 1.0),
        };
        draw_circle(pos.x, pos.y, 9.0, with_alpha(Colors::BACKGROUND, 0.94));
        draw_resource_icon(
            resource,
            Rect::new(pos.x - 7.0, pos.y - 7.0, 14.0, 14.0),
            color,
        );
    }
}

fn draw_machine_world(center: Vec2, radius: f32, time: f32) {
    for layer in (1..=5).rev() {
        draw_circle(
            center.x,
            center.y,
            radius + layer as f32 * 12.0,
            Color::new(0.0, 0.58, 0.72, 0.008 * (6 - layer) as f32),
        );
    }
    draw_circle(
        center.x,
        center.y,
        radius,
        Color::new(0.018, 0.07, 0.085, 1.0),
    );
    draw_circle(
        center.x - radius * 0.16,
        center.y - radius * 0.18,
        radius * 0.82,
        Color::new(0.025, 0.13, 0.15, 0.72),
    );
    draw_circle_lines(
        center.x,
        center.y,
        radius,
        2.0,
        with_alpha(Colors::PRIMARY, 0.66),
    );

    // Latitude and longitude traces stay inside the sphere, giving it both
    // planetary curvature and the feeling of an engineered shell.
    for scale in [0.34, 0.68] {
        draw_ellipse_lines(
            center.x,
            center.y,
            radius * scale,
            radius,
            0.0,
            1.0,
            with_alpha(Colors::PRIMARY_SOFT, 0.18),
        );
        draw_ellipse_lines(
            center.x,
            center.y,
            radius,
            radius * scale,
            0.0,
            1.0,
            with_alpha(Colors::PRIMARY_SOFT, 0.16),
        );
    }

    let nodes = [
        vec2(-0.54, -0.15),
        vec2(-0.27, -0.5),
        vec2(0.12, -0.4),
        vec2(0.5, -0.12),
        vec2(0.38, 0.34),
        vec2(0.02, 0.52),
        vec2(-0.42, 0.34),
        vec2(-0.05, 0.02),
    ];
    let links = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 5),
        (5, 6),
        (6, 0),
        (0, 7),
        (2, 7),
        (4, 7),
        (6, 7),
    ];
    for (from, to) in links {
        let a = center + nodes[from] * radius;
        let b = center + nodes[to] * radius;
        draw_line(a.x, a.y, b.x, b.y, 2.0, with_alpha(Colors::PRIMARY, 0.45));
        let packet = ((time * 0.25 + from as f32 * 0.13).fract()).clamp(0.0, 1.0);
        let p = a.lerp(b, packet);
        draw_circle(p.x, p.y, 2.3, Colors::ACCENT);
    }
    for (index, offset) in nodes.into_iter().enumerate() {
        let p = center + offset * radius;
        let node_radius = if index == 7 { 13.0 } else { 7.0 };
        draw_poly(p.x, p.y, 6, node_radius, 30.0, Colors::BACKGROUND);
        draw_poly_lines(
            p.x,
            p.y,
            6,
            node_radius,
            30.0,
            1.5,
            if index == 7 {
                Colors::PRIMARY
            } else {
                Colors::PRIMARY_SOFT
            },
        );
        draw_circle(
            p.x,
            p.y,
            node_radius * 0.35,
            if index == 7 {
                Colors::ACCENT
            } else {
                Colors::PRIMARY
            },
        );
    }
}

fn draw_swarm_streams(center: Vec2, radius: f32, time: f32) {
    for stream in 0..3 {
        let start = vec2(
            center.x - radius * 1.8,
            center.y + radius * (0.85 + stream as f32 * 0.18),
        );
        let end = center + vec2(-radius * 0.72, radius * (0.34 - stream as f32 * 0.16));
        let control = vec2(
            center.x - radius * 1.05,
            center.y + radius * (1.15 - stream as f32 * 0.42),
        );
        for particle in 0..15 {
            let t = (time * (0.09 + stream as f32 * 0.012) + particle as f32 / 15.0).fract();
            let p = quadratic_bezier(start, control, end, t);
            let alpha = (t * std::f32::consts::PI).sin().max(0.08) * 0.72;
            draw_poly(
                p.x,
                p.y,
                4,
                1.5 + (particle % 3) as f32 * 0.45,
                45.0,
                with_alpha(
                    if stream == 1 {
                        Colors::ACCENT
                    } else {
                        Colors::PRIMARY
                    },
                    alpha,
                ),
            );
        }
    }
}

fn draw_seed_ship(center: Vec2, radius: f32, time: f32) {
    let bob = (time * 0.8).sin() * 3.0;
    let origin = center + vec2(radius * 1.04, -radius * 0.78 + bob);
    let nose = origin + vec2(22.0, -24.0);
    let left = origin + vec2(-10.0, 18.0);
    let right = origin + vec2(16.0, 12.0);
    draw_triangle(nose, left, right, with_alpha(Colors::PRIMARY_SOFT, 0.28));
    draw_triangle_lines(nose, left, right, 1.5, Colors::PRIMARY);
    draw_line(
        origin.x + 1.0,
        origin.y + 11.0,
        origin.x - 10.0,
        origin.y + 31.0,
        3.0,
        with_alpha(Colors::ACCENT, 0.62),
    );
    draw_line(
        origin.x + 7.0,
        origin.y + 10.0,
        origin.x,
        origin.y + 28.0,
        1.5,
        with_alpha(Colors::PRIMARY, 0.72),
    );
}

fn quadratic_bezier(start: Vec2, control: Vec2, end: Vec2, t: f32) -> Vec2 {
    let inverse = 1.0 - t;
    start * inverse * inverse + control * 2.0 * inverse * t + end * t * t
}

fn orbit_position(center: Vec2, rx: f32, ry: f32, angle: f32, rotation: f32) -> Vec2 {
    let local = vec2(angle.cos() * rx, angle.sin() * ry);
    let rotation = rotation.to_radians();
    center
        + vec2(
            local.x * rotation.cos() - local.y * rotation.sin(),
            local.x * rotation.sin() + local.y * rotation.cos(),
        )
}

fn hash01(seed: usize) -> f32 {
    let value = (seed as u32)
        .wrapping_mul(747_796_405)
        .wrapping_add(2_891_336_453);
    ((value ^ (value >> 16)) & 0xffff) as f32 / 65_535.0
}

#[cfg(test)]
mod tests;
