//! Drone and particle rendering

use crate::engine::{DroneState, GridPos};
use crate::state::PlanetState;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::{multiply_alpha, with_alpha};

use super::metrics::{grid_to_screen, HudMetrics};

pub(super) fn draw_drones(state: &PlanetState, metrics: HudMetrics, time: f32) {
    for drone in state.drones.drones() {
        let (vx, vy) = drone.visual_position();
        let (dx, dy) = grid_to_screen(GridPos::new(vx as i32, vy as i32), metrics);

        let frac_x = vx - vx.floor();
        let frac_y = vy - vy.floor();
        let mut drone_x = dx + frac_x * metrics.tile_size + metrics.tile_size / 2.0 - 4.0;
        let mut drone_y = dy + frac_y * metrics.tile_size + metrics.tile_size / 2.0 - 4.0;

        let drone_color = match drone.state {
            DroneState::Idle => Colors::SECONDARY,
            DroneState::MovingToCore => Colors::SUCCESS,
            DroneState::MovingToDrill => Colors::WARNING,
            DroneState::Delivering => Colors::PRIMARY,
            DroneState::Error => Colors::ERROR,
        };

        let wobble = (time * 6.0 + drone.id as f32).sin() * 1.2;
        let float = (time * 5.0 + drone.id as f32 * 0.7).cos() * 1.0;

        if drone.state == DroneState::Idle {
            // Idle cluster wobble near drill
            drone_x += wobble * 0.6;
            drone_y += float * 0.6;
            draw_circle(drone_x, drone_y, 3.2, drone_color);
        } else if drone.state == DroneState::Error {
            // Error spin + glyph
            let spin = (time * 8.0 + drone.id as f32).sin();
            draw_circle(drone_x, drone_y, 4.0, drone_color);
            draw_line(
                drone_x - 4.0,
                drone_y - 4.0,
                drone_x + 4.0,
                drone_y + 4.0,
                1.0 + spin.abs(),
                Colors::ERROR,
            );
            draw_line(
                drone_x + 4.0,
                drone_y - 4.0,
                drone_x - 4.0,
                drone_y + 4.0,
                1.0 + spin.abs(),
                Colors::ERROR,
            );
        } else {
            draw_circle(
                drone_x + wobble * 0.2,
                drone_y + float * 0.2,
                4.0,
                drone_color,
            );
        }

        if drone.state != DroneState::Error
            && drone.path_index > 0
            && drone.path_index < drone.path.len()
        {
            let prev = drone.path[drone.path_index - 1];
            let next = drone.path[drone.path_index];
            let dir_x = (next.x - prev.x) as f32;
            let dir_y = (next.y - prev.y) as f32;
            let length = (dir_x * dir_x + dir_y * dir_y).sqrt().max(0.01);
            let norm_x = dir_x / length;
            let norm_y = dir_y / length;
            let tail_len = 10.0;
            for segment in 0..3 {
                let segment_ratio = segment as f32 / 3.0;
                let tail_x = drone_x - norm_x * tail_len * segment_ratio;
                let tail_y = drone_y - norm_y * tail_len * segment_ratio;
                let alpha = 0.4 * (1.0 - segment_ratio);
                let tail_color = with_alpha(drone_color, alpha);
                draw_circle(tail_x, tail_y, 3.0 - segment as f32 * 0.6, tail_color);
            }
        }

        if drone.carrying > 0.0 && drone.state != DroneState::Error {
            // Visible cargo packet between drill and core
            let mut cargo_x = drone_x;
            let mut cargo_y = drone_y;
            if drone.path_index > 0 && drone.path_index < drone.path.len() {
                let prev = drone.path[drone.path_index - 1];
                let next = drone.path[drone.path_index];
                let dir_x = (next.x - prev.x) as f32;
                let dir_y = (next.y - prev.y) as f32;
                let length = (dir_x * dir_x + dir_y * dir_y).sqrt().max(0.01);
                let norm_x = dir_x / length;
                let norm_y = dir_y / length;
                cargo_x += norm_x * 6.0;
                cargo_y += norm_y * 6.0;
            } else {
                cargo_y -= 6.0;
            }
            draw_rectangle(cargo_x - 2.0, cargo_y - 2.0, 4.0, 4.0, Colors::ACCENT);
            draw_circle(cargo_x, cargo_y, 2.0, Color::new(1.0, 0.8, 0.4, 0.9));
        }

        if state.power_collapse_shutdown > 0.0 {
            // Power collapse: drones sag/fall
            let fall = (1.0 - (state.power_collapse_shutdown / 20.0)).clamp(0.0, 1.0);
            draw_circle(drone_x, drone_y + fall * 6.0, 2.0, Colors::ERROR);
        }
    }
}

/// Outline the network tiles carrying more traffic than they can pass, so a
/// jam is something the player can see and route around.
pub(super) fn draw_congestion(state: &PlanetState, metrics: HudMetrics, time: f32) {
    let pulse = 0.45 + (time * 4.0).sin().abs() * 0.35;
    for (x, y) in state.traffic.keys() {
        let pos = GridPos::new(*x, *y);
        if !state.is_congested(pos) {
            continue;
        }
        let (screen_x, screen_y) = grid_to_screen(pos, metrics);
        draw_rectangle_lines(
            screen_x + 1.0,
            screen_y + 1.0,
            metrics.tile_size - 2.0,
            metrics.tile_size - 2.0,
            2.0,
            with_alpha(Colors::WARNING, pulse),
        );
    }
}

pub(super) fn draw_particles(state: &PlanetState, metrics: HudMetrics) {
    for particle in state.particles.particles() {
        let screen_x = metrics.grid_offset_x()
            + particle.position.x * metrics.tile_size
            + metrics.tile_size * 0.5;
        let screen_y = metrics.grid_offset_y()
            + particle.position.y * metrics.tile_size
            + metrics.tile_size * 0.5;
        let color = multiply_alpha(particle.color, particle.life_fraction());
        draw_circle(screen_x, screen_y, particle.size, color);
    }
}
