//! Optional factory-flow overlay: live recipe nodes and intended supply runs.

use crate::data::{RecipeDef, UiTheme};
use crate::engine::{route_over_network, BuildingType, GridPos, ResourceType};
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_resource_icon, resource_color, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::draw_ui_text;

use super::metrics::{grid_to_screen, HudMetrics};

#[derive(Debug, Clone)]
struct FlowLink {
    resource: ResourceType,
    path: Vec<GridPos>,
}

#[derive(Debug, Clone)]
struct FlowNode {
    pos: GridPos,
    inputs: Vec<ResourceType>,
    outputs: Vec<ResourceType>,
    readiness: f32,
    powered: bool,
}

pub(super) fn draw(state: &PlanetState, metrics: HudMetrics, theme: &UiTheme, time: f32) {
    if !state.flow_overlay {
        return;
    }

    for link in factory_flow_links(state) {
        draw_link(&link, metrics, theme, time);
    }

    let starved: std::collections::HashSet<GridPos> =
        state.starved_factories().into_iter().collect();
    for node in factory_flow_nodes(state) {
        draw_node(&node, starved.contains(&node.pos), metrics, theme);
    }

    draw_ui_text(
        "FACTORY FLOW // LIVE ROUTES",
        metrics.base_offset_x() + 12.0,
        metrics.base_offset_y() + 53.0,
        10.0,
        color_from_rgba(&theme.colors.primary_soft),
    );
}

fn draw_link(link: &FlowLink, metrics: HudMetrics, theme: &UiTheme, time: f32) {
    if link.path.len() < 2 {
        return;
    }
    let color = resource_color(theme, link.resource);
    for pair in link.path.windows(2) {
        let from = tile_center(pair[0], metrics);
        let to = tile_center(pair[1], metrics);
        draw_line(
            from.x,
            from.y,
            to.x,
            to.y,
            4.0,
            with_alpha(Colors::BACKGROUND, 0.8),
        );
        draw_line(from.x, from.y, to.x, to.y, 1.4, with_alpha(color, 0.62));
    }

    let travel = (time * 1.8 + link.path[0].x as f32 * 0.17 + link.path[0].y as f32 * 0.11)
        .rem_euclid((link.path.len() - 1) as f32);
    let index = travel.floor() as usize;
    let progress = travel.fract();
    let from = tile_center(link.path[index], metrics);
    let to = tile_center(link.path[index + 1], metrics);
    let packet = from.lerp(to, progress);
    draw_circle(packet.x, packet.y, 5.0, with_alpha(Colors::BACKGROUND, 0.9));
    draw_resource_icon(
        link.resource,
        Rect::new(packet.x - 4.0, packet.y - 4.0, 8.0, 8.0),
        color,
    );
}

fn draw_node(node: &FlowNode, starved: bool, metrics: HudMetrics, theme: &UiTheme) {
    let center = tile_center(node.pos, metrics);
    let icon = (metrics.tile_size * 0.29).clamp(8.0, 11.0);
    let item_count = node.inputs.len() + node.outputs.len();
    let width = (20.0 + item_count as f32 * (icon + 3.0)).max(40.0);
    let height = icon + 9.0;
    let x = center.x - width * 0.5;
    let y = center.y - metrics.tile_size * 0.72 - height;
    let border = if !node.powered {
        color_from_rgba(&theme.colors.text_dim)
    } else if starved {
        color_from_rgba(&theme.colors.warning)
    } else {
        color_from_rgba(&theme.colors.border_bright)
    };
    draw_rectangle(x, y, width, height, Color::new(0.006, 0.025, 0.035, 0.94));
    draw_rectangle_lines(x, y, width, height, 1.0, with_alpha(border, 0.86));

    let mut cursor = x + 5.0;
    for resource in &node.inputs {
        draw_resource_icon(
            *resource,
            Rect::new(cursor, y + 3.0, icon, icon),
            resource_color(theme, *resource),
        );
        cursor += icon + 3.0;
    }
    draw_ui_text(
        ">",
        cursor,
        y + icon + 2.0,
        8.0,
        color_from_rgba(&theme.colors.text_dim),
    );
    cursor += 8.0;
    for resource in &node.outputs {
        draw_resource_icon(
            *resource,
            Rect::new(cursor, y + 3.0, icon, icon),
            resource_color(theme, *resource),
        );
        cursor += icon + 3.0;
    }

    let bar_y = y + height - 3.0;
    draw_rectangle(
        x + 2.0,
        bar_y,
        width - 4.0,
        1.5,
        with_alpha(Colors::TEXT_DIM, 0.2),
    );
    draw_rectangle(
        x + 2.0,
        bar_y,
        (width - 4.0) * node.readiness,
        1.5,
        if starved {
            color_from_rgba(&theme.colors.warning)
        } else {
            color_from_rgba(&theme.colors.success)
        },
    );
}

fn factory_flow_nodes(state: &PlanetState) -> Vec<FlowNode> {
    let mut nodes = Vec::new();
    for def in &crate::data::game_data().buildings {
        if def.recipe.is_empty() {
            continue;
        }
        let Some(kind) = BuildingType::from_id(&def.id) else {
            continue;
        };
        let inputs = ordered_resources(def.recipe.inputs.keys().map(String::as_str));
        let outputs = ordered_resources(def.recipe.outputs.keys().map(String::as_str));
        for pos in state.grid.find_buildings(kind) {
            nodes.push(FlowNode {
                pos,
                inputs: inputs.clone(),
                outputs: outputs.clone(),
                readiness: recipe_readiness(state, pos, &def.recipe),
                powered: is_operational(state, pos),
            });
        }
    }
    nodes.sort_by_key(|node| (node.pos.y, node.pos.x));
    nodes
}

fn factory_flow_links(state: &PlanetState) -> Vec<FlowLink> {
    let Some(core) = state.grid.find_core() else {
        return Vec::new();
    };
    let mut producers: Vec<(GridPos, ResourceType)> = state
        .grid
        .find_buildings(BuildingType::Drill)
        .into_iter()
        .filter(|pos| is_operational(state, *pos))
        .map(|pos| (pos, ResourceType::Minerals))
        .collect();
    let mut consumers: Vec<(GridPos, ResourceType)> = Vec::new();

    for def in &crate::data::game_data().buildings {
        let Some(kind) = BuildingType::from_id(&def.id) else {
            continue;
        };
        let positions = state.grid.find_buildings(kind);
        let active: Vec<GridPos> = positions
            .iter()
            .copied()
            .filter(|pos| is_operational(state, *pos))
            .collect();
        for resource in ordered_resources(def.recipe.outputs.keys().map(String::as_str)) {
            if resource.is_physical() {
                producers.extend(active.iter().copied().map(|pos| (pos, resource)));
            }
        }
        for id in def.recipe.carried_ids() {
            let Some(resource) = ResourceType::from_id(id) else {
                continue;
            };
            consumers.extend(active.iter().copied().map(|pos| (pos, resource)));
        }
    }

    let mut links = Vec::new();
    for (source, resource) in producers {
        let target = consumers
            .iter()
            .filter(|(pos, wanted)| *wanted == resource && *pos != source)
            .filter_map(|(pos, _)| {
                route_over_network(&state.grid, source, *pos).map(|path| (*pos, path))
            })
            .min_by_key(|(_, path)| path.len())
            .map(|(_, path)| path)
            .or_else(|| route_over_network(&state.grid, source, core));
        if let Some(mut path) = target {
            // Engine routes are walking instructions and therefore exclude
            // the source. The overlay is a diagram, so close that visual gap.
            if path.first() != Some(&source) {
                path.insert(0, source);
            }
            links.push(FlowLink { resource, path });
        }
    }
    links
}

fn recipe_readiness(state: &PlanetState, pos: GridPos, recipe: &RecipeDef) -> f32 {
    let carried = recipe.carried_ids();
    recipe
        .inputs
        .iter()
        .filter(|(_, rate)| **rate > 0.0)
        .filter_map(|(id, rate)| {
            let resource = ResourceType::from_id(id)?;
            let available = if carried.contains(&resource.id()) {
                state
                    .input_hoppers
                    .get(&(pos.x, pos.y))
                    .and_then(|hopper| hopper.get(&resource))
                    .copied()
                    .unwrap_or_else(|| {
                        if recipe.carried.as_deref() == Some(resource.id()) {
                            state
                                .input_buffers
                                .get(&(pos.x, pos.y))
                                .copied()
                                .unwrap_or(0.0)
                        } else {
                            0.0
                        }
                    })
            } else {
                state.resources.get(resource)
            };
            Some((available / (*rate * 5.0)).clamp(0.0, 1.0))
        })
        .fold(1.0, f32::min)
}

fn ordered_resources<'a>(ids: impl Iterator<Item = &'a str>) -> Vec<ResourceType> {
    let ids: std::collections::HashSet<&str> = ids.collect();
    ResourceType::ALL
        .into_iter()
        .filter(|resource| ids.contains(resource.id()))
        .collect()
}

fn is_operational(state: &PlanetState, pos: GridPos) -> bool {
    state
        .grid
        .get(pos)
        .and_then(|tile| tile.building.as_ref())
        .is_some_and(|building| building.powered && !building.is_dust_stalled())
}

fn tile_center(pos: GridPos, metrics: HudMetrics) -> Vec2 {
    let (x, y) = grid_to_screen(pos, metrics);
    vec2(x + metrics.tile_size * 0.5, y + metrics.tile_size * 0.5)
}

#[cfg(test)]
mod tests;
