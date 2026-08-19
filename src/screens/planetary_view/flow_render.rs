//! Optional factory-flow overlay: live recipe nodes and intended supply runs.

use crate::data::{RecipeDef, UiTheme};
use crate::engine::{BuildingType, GridPos, ResourceType};
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_panel, draw_resource_icon, resource_color, Colors};
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
    blocked: bool,
    output_pressure: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct FactoryLedger {
    processors: usize,
    active: usize,
    starved: usize,
    blocked: usize,
    boosted: usize,
    bottleneck: Option<ResourceType>,
    ore_rate: f32,
    alloy_rate: f32,
    components_rate: f32,
    auto_clocking: bool,
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

    draw_factory_ledger(&factory_ledger(state), metrics, theme);

    draw_ui_text(
        "FACTORY FLOW // LIVE ROUTES",
        metrics.base_offset_x() + 12.0,
        metrics.base_offset_y() + 53.0,
        10.0,
        color_from_rgba(&theme.colors.primary_soft),
    );
    draw_node_key(metrics, theme);
}

fn draw_node_key(metrics: HudMetrics, theme: &UiTheme) {
    let x = metrics.base_offset_x() + 12.0;
    let y = metrics.base_offset_y() + 69.0;
    draw_ui_text(
        "NODE BARS",
        x,
        y,
        8.0,
        color_from_rgba(&theme.colors.text_dim),
    );
    draw_line(
        x + 58.0,
        y - 3.0,
        x + 72.0,
        y - 3.0,
        2.0,
        color_from_rgba(&theme.colors.success),
    );
    draw_ui_text(
        "INPUT",
        x + 77.0,
        y,
        8.0,
        color_from_rgba(&theme.colors.text_dim),
    );
    draw_line(
        x + 116.0,
        y - 10.0,
        x + 116.0,
        y,
        2.0,
        color_from_rgba(&theme.colors.components),
    );
    draw_ui_text(
        "OUTPUT",
        x + 122.0,
        y,
        8.0,
        color_from_rgba(&theme.colors.text_dim),
    );
}

fn draw_factory_ledger(ledger: &FactoryLedger, metrics: HudMetrics, theme: &UiTheme) {
    let grid_room = screen_width() - metrics.base_offset_x() - metrics.right_panel_width - 14.0;
    let x = metrics.base_offset_x() + if grid_room >= 500.0 { 170.0 } else { 8.0 };
    let width = (screen_width() - x - metrics.right_panel_width - 14.0)
        .max(250.0)
        .min(520.0);
    let compact = width < 400.0;
    let area = Rect::new(
        x,
        metrics.base_offset_y() + 8.0,
        width,
        if compact { 108.0 } else { 88.0 },
    );
    draw_hud_panel(theme, area, Some("FACTORY LEDGER"));

    let text = color_from_rgba(&theme.colors.text);
    let dim = color_from_rgba(&theme.colors.text_dim);
    let warning = color_from_rgba(&theme.colors.warning);
    let success = color_from_rgba(&theme.colors.success);
    let summary = format!(
        "PROC {}   ACTIVE {}   STARVED {}   BLOCKED {}   BOOST {}",
        ledger.processors, ledger.active, ledger.starved, ledger.blocked, ledger.boosted
    );
    draw_ui_text(&summary, area.x + 12.0, area.y + 50.0, 10.0, text);

    let clock_mode = if ledger.auto_clocking {
        "AUTO CLOCK"
    } else {
        "MANUAL CLOCK"
    };
    let bottleneck = ledger
        .bottleneck
        .map(|resource| {
            let label = if resource == ResourceType::Minerals {
                "ORE".to_string()
            } else {
                resource.id().to_uppercase()
            };
            format!("{} // BOTTLENECK {}", clock_mode, label)
        })
        .unwrap_or_else(|| {
            if ledger.blocked > 0 {
                format!("{} // DISPATCH PAD FULL", clock_mode)
            } else {
                format!("{} // FLOW STABLE", clock_mode)
            }
        });
    draw_ui_text(
        &bottleneck,
        area.x + 12.0,
        area.y + 71.0,
        9.0,
        if ledger.bottleneck.is_some() || ledger.blocked > 0 {
            warning
        } else {
            success
        },
    );

    let rates = [
        (ResourceType::Minerals, "IN", ledger.ore_rate),
        (ResourceType::Alloy, "A", ledger.alloy_rate),
        (ResourceType::Components, "C", ledger.components_rate),
    ];
    let start_x = if compact {
        area.x + 12.0
    } else {
        area.x + area.w - 224.0
    };
    let rate_y = if compact {
        area.y + 82.0
    } else {
        area.y + 59.0
    };
    for (index, (resource, label, rate)) in rates.into_iter().enumerate() {
        let chip_x = start_x + index as f32 * 74.0;
        draw_resource_icon(
            resource,
            Rect::new(chip_x, rate_y, 10.0, 10.0),
            resource_color(theme, resource),
        );
        draw_ui_text(
            &format!("{} {:.1}/s", label, rate),
            chip_x + 14.0,
            rate_y + 10.0,
            8.0,
            dim,
        );
    }
}

fn factory_ledger(state: &PlanetState) -> FactoryLedger {
    let nodes = factory_flow_nodes(state);
    let starved: std::collections::HashSet<GridPos> =
        state.starved_factories().into_iter().collect();
    let blocked: std::collections::HashSet<GridPos> =
        state.blocked_factories().into_iter().collect();
    let active = nodes
        .iter()
        .filter(|node| node.powered && node.readiness > 0.001 && !blocked.contains(&node.pos))
        .count();
    let boosted = nodes
        .iter()
        .filter(|node| {
            state
                .grid
                .get(node.pos)
                .and_then(|tile| tile.building.as_ref())
                .is_some_and(|building| building.overclocked)
        })
        .count();
    FactoryLedger {
        processors: nodes.len(),
        active,
        starved: starved.len(),
        blocked: blocked.len(),
        boosted,
        bottleneck: factory_bottleneck(state, &starved),
        ore_rate: state.throughput.last().unwrap_or(0.0),
        alloy_rate: state.observed_alloy_rate(),
        components_rate: state.observed_components_rate(),
        auto_clocking: state.auto_clocking,
    }
}

fn factory_bottleneck(
    state: &PlanetState,
    starved: &std::collections::HashSet<GridPos>,
) -> Option<ResourceType> {
    let mut missing = Vec::<(ResourceType, usize)>::new();
    for pos in starved {
        let Some(kind) = state
            .grid
            .get(*pos)
            .and_then(|tile| tile.building.as_ref())
            .map(|building| building.building_type)
        else {
            continue;
        };
        let recipe = &crate::data::game_data().building(kind.id()).recipe;
        for resource in ordered_resources(recipe.inputs.keys().map(String::as_str)) {
            if input_available(state, *pos, recipe, resource) <= 0.001 {
                if let Some((_, count)) = missing.iter_mut().find(|(item, _)| *item == resource) {
                    *count += 1;
                } else {
                    missing.push((resource, 1));
                }
            }
        }
    }
    missing
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(resource, _)| resource)
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
    } else if node.blocked {
        color_from_rgba(&theme.colors.error)
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
        if node.blocked {
            color_from_rgba(&theme.colors.error)
        } else if starved {
            color_from_rgba(&theme.colors.warning)
        } else {
            color_from_rgba(&theme.colors.success)
        },
    );
    let output_x = x + width - 3.0;
    let output_h = (height - 4.0) * node.output_pressure;
    draw_rectangle(
        output_x,
        y + 2.0,
        1.5,
        height - 4.0,
        with_alpha(Colors::TEXT_DIM, 0.2),
    );
    draw_rectangle(
        output_x,
        y + height - 2.0 - output_h,
        1.5,
        output_h,
        if node.blocked {
            color_from_rgba(&theme.colors.error)
        } else {
            node.outputs
                .first()
                .map(|resource| resource_color(theme, *resource))
                .unwrap_or(Colors::TEXT_DIM)
        },
    );
}

fn factory_flow_nodes(state: &PlanetState) -> Vec<FlowNode> {
    let mut nodes = Vec::new();
    let blocked: std::collections::HashSet<GridPos> =
        state.blocked_factories().into_iter().collect();
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
                blocked: blocked.contains(&pos),
                output_pressure: (state
                    .output_buffers
                    .get(&(pos.x, pos.y))
                    .copied()
                    .unwrap_or(0.0)
                    / state.processor_pad_capacity())
                .clamp(0.0, 1.0),
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
    }

    let mut links = Vec::new();
    for (source, resource) in producers {
        let target = state
            .delivery_for(source, core, resource)
            .map(|(_, path)| path);
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
    recipe
        .inputs
        .iter()
        .filter(|(_, rate)| **rate > 0.0)
        .filter_map(|(id, rate)| {
            let resource = ResourceType::from_id(id)?;
            let available = input_available(state, pos, recipe, resource);
            Some((available / (*rate * 5.0)).clamp(0.0, 1.0))
        })
        .fold(1.0, f32::min)
}

fn input_available(
    state: &PlanetState,
    pos: GridPos,
    recipe: &RecipeDef,
    resource: ResourceType,
) -> f32 {
    if recipe.carried_ids().contains(&resource.id()) {
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
    }
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
