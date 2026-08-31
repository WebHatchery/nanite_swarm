use crate::engine::BuildingType;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

pub(super) fn draw_building_motion(
    building_type: BuildingType,
    center_x: f32,
    center_y: f32,
    tile_size: f32,
    world_time: f64,
    overclocked: bool,
) {
    let motion = if macroquad_toolkit::settings::reduced_motion_enabled() {
        0.0
    } else {
        world_time as f32 * if overclocked { 1.65 } else { 1.0 }
    };
    if overclocked {
        let pulse = (motion * 2.4).sin() * 0.5 + 0.5;
        draw_circle_lines(
            center_x,
            center_y,
            tile_size * (0.36 + pulse * 0.08),
            1.2,
            with_alpha(Color::new(0.78, 0.55, 0.95, 1.0), 0.45 + pulse * 0.4),
        );
        for spark in 0..3 {
            let angle = motion * 3.0 + spark as f32 * std::f32::consts::TAU / 3.0;
            draw_circle(
                center_x + angle.cos() * tile_size * 0.34,
                center_y + angle.sin() * tile_size * 0.34,
                1.2,
                Colors::WARNING,
            );
        }
    }
    match building_type {
        BuildingType::Drill => {
            let angle = motion * 4.0;
            let tip = vec2(
                center_x + angle.cos() * tile_size * 0.28,
                center_y + angle.sin() * tile_size * 0.28,
            );
            draw_line(center_x, center_y, tip.x, tip.y, 1.5, Colors::TEXT);
        }
        BuildingType::ServerBank => {
            let blink = if (motion * 2.0).sin() > 0.0 {
                1.0
            } else {
                0.35
            };
            draw_circle(
                center_x - tile_size * 0.18,
                center_y,
                2.0,
                with_alpha(Colors::SUCCESS, blink),
            );
            draw_circle(
                center_x + tile_size * 0.18,
                center_y,
                2.0,
                with_alpha(Colors::PRIMARY, 1.0 - blink * 0.5),
            );
        }
        BuildingType::WindTurbine => {
            for spoke in 0..3 {
                let angle = motion * 3.0 + spoke as f32 * std::f32::consts::TAU / 3.0;
                draw_line(
                    center_x,
                    center_y,
                    center_x + angle.cos() * tile_size * 0.32,
                    center_y + angle.sin() * tile_size * 0.32,
                    1.0,
                    Colors::PRIMARY,
                );
            }
        }
        BuildingType::Assembler => {
            let phase = (motion * 3.0).sin() * 0.5 + 0.5;
            let reach = tile_size * (0.08 + phase * 0.11);
            let outer = tile_size * 0.28;
            draw_line(
                center_x - outer,
                center_y - tile_size * 0.14,
                center_x - reach,
                center_y,
                1.4,
                Colors::PRIMARY,
            );
            draw_line(
                center_x + outer,
                center_y - tile_size * 0.14,
                center_x + reach,
                center_y,
                1.4,
                Colors::PRIMARY,
            );
            draw_circle(
                center_x,
                center_y,
                1.5 + phase,
                with_alpha(Colors::WARNING, 0.35 + phase * 0.55),
            );
        }
        BuildingType::Smelter => {
            let phase = (motion * 2.5).sin() * 0.5 + 0.5;
            draw_line(
                center_x - tile_size * 0.2,
                center_y + tile_size * 0.14,
                center_x + tile_size * 0.2,
                center_y + tile_size * 0.14,
                1.2 + phase,
                with_alpha(Colors::ACCENT, 0.45 + phase * 0.45),
            );
            draw_circle(
                center_x,
                center_y - tile_size * 0.08,
                1.2 + phase * 1.4,
                with_alpha(Colors::WARNING, 0.4 + phase * 0.5),
            );
        }
        _ => {}
    }
}
