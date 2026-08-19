//! Drone and particle rendering

use crate::data::UiTheme;
use crate::engine::{DroneState, GridPos, ResourceType};
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_panel, draw_resource_icon, resource_color, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::colors::{multiply_alpha, with_alpha};
use macroquad_toolkit::ui::draw_ui_text;

use super::metrics::{grid_to_screen, HudMetrics};

pub(super) fn draw_drones(state: &PlanetState, metrics: HudMetrics, theme: &UiTheme, time: f32) {
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
            let tail_color = if drone.carrying > 0.0 {
                resource_color(theme, drone.resource_type)
            } else {
                drone_color
            };
            let tail_len = 10.0;
            for segment in 0..3 {
                let segment_ratio = segment as f32 / 3.0;
                let tail_x = drone_x - norm_x * tail_len * segment_ratio;
                let tail_y = drone_y - norm_y * tail_len * segment_ratio;
                let alpha = 0.4 * (1.0 - segment_ratio);
                let faded_tail = with_alpha(tail_color, alpha);
                draw_circle(tail_x, tail_y, 3.0 - segment as f32 * 0.6, faded_tail);
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
            let cargo_color = resource_color(theme, drone.resource_type);
            let packet_size = (metrics.tile_size * 0.34).clamp(8.0, 12.0);
            draw_circle(
                cargo_x,
                cargo_y,
                packet_size * 0.58,
                with_alpha(Colors::BACKGROUND, 0.92),
            );
            draw_resource_icon(
                drone.resource_type,
                Rect::new(
                    cargo_x - packet_size * 0.5,
                    cargo_y - packet_size * 0.5,
                    packet_size,
                    packet_size,
                ),
                cargo_color,
            );
        }

        if state.power_collapse_shutdown > 0.0 {
            // Power collapse: drones sag/fall
            let length = state.power_collapse_length.max(0.001);
            let fall = (1.0 - (state.power_collapse_shutdown / length)).clamp(0.0, 1.0);
            draw_circle(drone_x, drone_y + fall * 6.0, 2.0, Colors::ERROR);
        }
    }

    draw_freight_legend(state, metrics, theme);

    // A queue is drawn at the first reserved tile rather than hidden in a
    // counter: numbered chevrons tell the player which drone will get the
    // junction next.
    for (key, queue) in &state.drone_queues {
        let pos = GridPos::new(key.0, key.1);
        let (x, y) = grid_to_screen(pos, metrics);
        for (order, id) in queue.iter().enumerate().take(6) {
            draw_circle(
                x + metrics.tile_size * 0.18 + order as f32 * 7.0,
                y + metrics.tile_size * 0.18,
                4.0,
                with_alpha(Colors::ACCENT, 0.75),
            );
            macroquad_toolkit::ui::draw_ui_text(
                &format!("{}", order + 1),
                x + metrics.tile_size * 0.18 - 2.0 + order as f32 * 7.0,
                y + metrics.tile_size * 0.18 + 3.0,
                7.0,
                Colors::BACKGROUND,
            );
            let _ = id;
        }
    }
}

/// A small, dynamic key anchors the shape language while freight is moving.
/// It only lists materials that are actually on the network right now.
fn draw_freight_legend(state: &PlanetState, metrics: HudMetrics, theme: &UiTheme) {
    let resources: Vec<ResourceType> = ResourceType::ALL
        .into_iter()
        .filter(|resource| {
            state
                .drones
                .drones()
                .iter()
                .any(|drone| drone.carrying > 0.0 && drone.resource_type == *resource)
        })
        .collect();
    if resources.is_empty() {
        return;
    }

    let item_w = 64.0;
    let rect = Rect::new(
        metrics.base_offset_x() + 10.0,
        metrics.base_offset_y() + 10.0,
        66.0 + item_w * resources.len() as f32,
        28.0,
    );
    draw_hud_panel(theme, rect, None);
    draw_ui_text(
        "FREIGHT",
        rect.x + 8.0,
        rect.y + 18.0,
        9.0,
        color_from_rgba(&theme.colors.text_dim),
    );
    for (index, resource) in resources.into_iter().enumerate() {
        let x = rect.x + 68.0 + index as f32 * item_w;
        draw_resource_icon(
            resource,
            Rect::new(x, rect.y + 7.0, 14.0, 14.0),
            resource_color(theme, resource),
        );
        draw_ui_text(
            freight_label(resource),
            x + 18.0,
            rect.y + 18.0,
            9.0,
            color_from_rgba(&theme.colors.text),
        );
    }
}

fn freight_label(resource: ResourceType) -> &'static str {
    match resource {
        ResourceType::Minerals => "ORE",
        ResourceType::Energy => "POWER",
        ResourceType::Data => "DATA",
        ResourceType::Biomass => "BIO",
        ResourceType::Alloy => "ALLOY",
        ResourceType::Components => "PARTS",
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

#[cfg(test)]
mod tests;
