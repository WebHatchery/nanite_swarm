//! Small physical hoppers and output stacks that make processor state visible on-map.

use crate::data::{RecipeDef, UiTheme};
use crate::engine::{BuildingType, GridPos, ResourceType};
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, resource_color, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use super::metrics::{grid_to_screen, HudMetrics};

#[derive(Debug, Clone, PartialEq)]
struct ProcessorVisual {
    pos: GridPos,
    inputs: Vec<(ResourceType, f32)>,
    output: (ResourceType, f32),
    active: bool,
    blocked: bool,
    priority: bool,
    overclocked: bool,
    standby: bool,
}

pub(super) fn draw_processor_buffers(
    state: &PlanetState,
    metrics: HudMetrics,
    theme: &UiTheme,
    time: f32,
) {
    if metrics.tile_size < 24.0 {
        return;
    }
    for visual in processor_visuals(state) {
        draw_processor(&visual, metrics, theme, time);
    }
}

fn draw_processor(visual: &ProcessorVisual, metrics: HudMetrics, theme: &UiTheme, time: f32) {
    let (x, y) = grid_to_screen(visual.pos, metrics);
    let size = metrics.tile_size;
    let center = vec2(x + size * 0.5, y + size * 0.53);
    let tank_w = (size * 0.105).clamp(3.0, 5.0);
    let tank_h = size * 0.34;
    let tank_y = y + size - tank_h - 3.0;

    for (index, (resource, fill)) in visual.inputs.iter().enumerate() {
        let tank_x = x + 3.0 + index as f32 * (tank_w + 2.0);
        let color = resource_color(theme, *resource);
        draw_rectangle(
            tank_x,
            tank_y,
            tank_w,
            tank_h,
            with_alpha(Colors::BACKGROUND, 0.82),
        );
        draw_rectangle(
            tank_x + 1.0,
            tank_y + tank_h * (1.0 - fill),
            (tank_w - 2.0).max(1.0),
            tank_h * fill,
            with_alpha(color, 0.88),
        );
        draw_rectangle_lines(tank_x, tank_y, tank_w, tank_h, 1.0, with_alpha(color, 0.7));

        if visual.active && *fill > 0.001 {
            let start = vec2(tank_x + tank_w, tank_y + tank_h * 0.45);
            draw_line(
                start.x,
                start.y,
                center.x,
                center.y,
                1.0,
                with_alpha(color, 0.5),
            );
            let travel = (time * 1.8 + index as f32 * 0.43).fract();
            let packet = start.lerp(center, travel);
            draw_circle(packet.x, packet.y, 1.4, color);
        }
    }

    let output_color = resource_color(theme, visual.output.0);
    let pad_x = x + size - 10.0;
    let pad_y = y + size - 12.0;
    draw_rectangle(
        pad_x - 2.0,
        pad_y - 2.0,
        10.0,
        12.0,
        with_alpha(Colors::BACKGROUND, 0.72),
    );
    let crates = if visual.output.1 <= 0.001 {
        0
    } else {
        (visual.output.1 * 3.0).ceil().clamp(1.0, 3.0) as usize
    };
    for stack in 0..crates {
        let crate_y = pad_y + 6.0 - stack as f32 * 3.5;
        draw_rectangle(pad_x, crate_y, 6.0, 3.0, with_alpha(output_color, 0.9));
        draw_rectangle_lines(pad_x, crate_y, 6.0, 3.0, 0.7, Colors::BACKGROUND);
    }
    draw_rectangle_lines(
        pad_x - 2.0,
        pad_y - 2.0,
        10.0,
        12.0,
        1.0,
        with_alpha(output_color, 0.7),
    );

    if visual.blocked {
        let pulse = 0.62 + (time * 4.0).sin().abs() * 0.32;
        let blocked = color_from_rgba(&theme.colors.error);
        for shutter in 0..2 {
            let shutter_y = pad_y + shutter as f32 * 4.0;
            draw_line(
                pad_x - 2.0,
                shutter_y,
                pad_x + 8.0,
                shutter_y,
                1.2,
                with_alpha(blocked, pulse),
            );
        }
    }

    if visual.priority {
        let beacon = color_from_rgba(&theme.colors.primary);
        let beacon_y = y + 4.0 + (time * 3.0).sin() * 0.8;
        draw_triangle(
            vec2(center.x, beacon_y - 3.0),
            vec2(center.x - 3.0, beacon_y),
            vec2(center.x, beacon_y + 3.0),
            with_alpha(beacon, 0.9),
        );
        draw_triangle(
            vec2(center.x, beacon_y - 3.0),
            vec2(center.x + 3.0, beacon_y),
            vec2(center.x, beacon_y + 3.0),
            with_alpha(beacon, 0.9),
        );
    }

    if visual.standby {
        let standby = color_from_rgba(&theme.colors.text_dim);
        draw_rectangle(
            center.x - 5.0,
            center.y - 5.0,
            10.0,
            10.0,
            with_alpha(Colors::BACKGROUND, 0.78),
        );
        draw_line(
            center.x - 2.0,
            center.y - 3.0,
            center.x - 2.0,
            center.y + 3.0,
            1.5,
            standby,
        );
        draw_line(
            center.x + 2.0,
            center.y - 3.0,
            center.x + 2.0,
            center.y + 3.0,
            1.5,
            standby,
        );
    }

    if visual.active {
        let speed = if visual.overclocked { 3.8 } else { 2.4 };
        let phase = (time * speed + visual.pos.x as f32 * 0.17).fract();
        let packet = center.lerp(vec2(pad_x - 1.0, pad_y + 3.0), phase);
        draw_circle(packet.x, packet.y, 1.3, with_alpha(output_color, 0.85));
        if visual.overclocked {
            let second = center.lerp(vec2(pad_x - 1.0, pad_y + 3.0), (phase + 0.5).fract());
            draw_circle(second.x, second.y, 1.1, with_alpha(output_color, 0.66));
        }
    } else if visual.inputs.iter().any(|(_, fill)| *fill <= 0.001) {
        draw_circle_lines(
            center.x,
            center.y,
            size * 0.2,
            1.0,
            color_from_rgba(&theme.colors.warning),
        );
    }
}

fn processor_visuals(state: &PlanetState) -> Vec<ProcessorVisual> {
    let mut visuals = Vec::new();
    for def in &crate::data::game_data().buildings {
        if def.recipe.is_empty() {
            continue;
        }
        let Some(kind) = BuildingType::from_id(&def.id) else {
            continue;
        };
        for pos in state.grid.find_buildings(kind) {
            if let Some(visual) = processor_visual(state, pos, &def.recipe) {
                visuals.push(visual);
            }
        }
    }
    visuals.sort_by_key(|visual| (visual.pos.y, visual.pos.x));
    visuals
}

fn processor_visual(
    state: &PlanetState,
    pos: GridPos,
    recipe: &RecipeDef,
) -> Option<ProcessorVisual> {
    let building = state.grid.get(pos)?.building.as_ref()?;
    let mut inputs = Vec::new();
    for resource in ResourceType::ALL {
        let rate = recipe.inputs.get(resource.id()).copied().unwrap_or(0.0);
        if rate <= 0.0 {
            continue;
        }
        let available = if recipe.carried_ids().contains(&resource.id()) {
            state
                .input_hoppers
                .get(&(pos.x, pos.y))
                .and_then(|hopper| hopper.get(&resource))
                .copied()
                .unwrap_or(0.0)
        } else {
            state.resources.get(resource)
        };
        inputs.push((resource, (available / (rate * 5.0)).clamp(0.0, 1.0)));
    }
    let output = ResourceType::ALL.into_iter().find_map(|resource| {
        let rate = recipe.outputs.get(resource.id()).copied().unwrap_or(0.0);
        (rate > 0.0).then_some((resource, rate))
    })?;
    let waiting = state
        .output_buffers
        .get(&(pos.x, pos.y))
        .copied()
        .unwrap_or(0.0);
    let output_fill = (waiting / state.processor_pad_capacity()).clamp(0.0, 1.0);
    let active = building.powered
        && !building.standby
        && !building.is_dust_stalled()
        && inputs.iter().all(|(_, fill)| *fill > 0.001)
        && output_fill < 0.999;
    Some(ProcessorVisual {
        pos,
        inputs,
        output: (output.0, output_fill),
        active,
        blocked: output_fill >= 0.999,
        priority: building.input_priority,
        overclocked: building.overclocked,
        standby: building.standby,
    })
}

#[cfg(test)]
mod tests;
