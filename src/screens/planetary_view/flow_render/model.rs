//! Live flow-model calculations and overlay drawing helpers.

use super::*;

pub(super) fn factory_ledger(state: &PlanetState) -> FactoryLedger {
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
    let priority = nodes.iter().filter(|node| node.priority).count();
    let standby = nodes.iter().filter(|node| node.standby).count();
    FactoryLedger {
        processors: nodes.len(),
        active,
        starved: starved.len(),
        blocked: blocked.len(),
        boosted,
        priority,
        standby,
        congested: state.congested_tiles(),
        peak_network_load: state.traffic.values().copied().max().unwrap_or(0),
        network_capacity: state.config.buildings.conduit_capacity.max(1.0),
        bottleneck: factory_bottleneck(state, &starved),
        ore_rate: state.throughput.last().unwrap_or(0.0),
        alloy_rate: state.observed_alloy_rate(),
        alloy_capacity: processor_output_capacity(state, ResourceType::Alloy),
        components_rate: state.observed_components_rate(),
        components_capacity: processor_output_capacity(state, ResourceType::Components),
        auto_clocking: state.auto_clocking,
    }
}

pub(super) fn processor_output_capacity(state: &PlanetState, resource: ResourceType) -> f32 {
    let dust_response = state.resolved_dust_response();
    state
        .grid
        .iter_tiles()
        .filter_map(|(_, tile)| tile.building.as_ref())
        .filter_map(|building| {
            let rate = state
                .data()
                .building(building.building_type.id())
                .recipe
                .outputs
                .get(resource.id())
                .copied()?;
            Some(rate * building.work_multiplier() * building.dust_efficiency_with(&dust_response))
        })
        .sum()
}

pub(super) fn factory_bottleneck(
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
        let recipe = &state.data().building(kind.id()).recipe;
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

pub(super) fn draw_link(link: &FlowLink, metrics: HudMetrics, theme: &UiTheme, time: f32) {
    if link.path.len() < 2 {
        return;
    }
    let resource = resource_color(theme, link.resource);
    let saturation = link.peak_load as f32 / link.capacity.max(1.0);
    let congested = saturation > 1.0;
    let color = if congested {
        color_from_rgba(&theme.colors.warning)
    } else {
        resource
    };
    let pressure = saturation.clamp(0.0, 1.0);
    let route_width = 1.2 + pressure * 1.8;
    let pulse = 0.55 + (time * 4.0).sin().abs() * 0.3;
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
        if congested {
            draw_line(
                from.x,
                from.y,
                to.x,
                to.y,
                5.0,
                with_alpha(color, 0.12 * pulse),
            );
        }
        draw_line(
            from.x,
            from.y,
            to.x,
            to.y,
            route_width,
            with_alpha(color, 0.44 + pressure * 0.38),
        );
    }

    let packet_speed = 1.8 / saturation.max(1.0);
    let travel =
        (time * packet_speed + link.path[0].x as f32 * 0.17 + link.path[0].y as f32 * 0.11)
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
        resource,
    );
}

pub(super) fn draw_node(node: &FlowNode, starved: bool, metrics: HudMetrics, theme: &UiTheme) {
    let center = tile_center(node.pos, metrics);
    let icon = (metrics.tile_size * 0.29).clamp(8.0, 11.0);
    let item_count = node.inputs.len() + node.outputs.len();
    let width = (20.0 + item_count as f32 * (icon + 3.0)).max(40.0);
    let height = icon + 9.0;
    let x = center.x - width * 0.5;
    let y = center.y - metrics.tile_size * 0.72 - height;
    let border = if node.standby || !node.powered {
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
    if node.priority {
        draw_circle(
            x + 3.0,
            y + 3.0,
            4.0,
            color_from_rgba(&theme.colors.primary),
        );
        draw_ui_text("P", x + 1.0, y + 6.0, 7.0, Colors::BACKGROUND);
    }
    if node.standby {
        let standby = color_from_rgba(&theme.colors.text_dim);
        draw_line(x + 5.0, y + 4.0, x + 5.0, y + 11.0, 1.5, standby);
        draw_line(x + 8.0, y + 4.0, x + 8.0, y + 11.0, 1.5, standby);
    }
}

pub(super) fn factory_flow_nodes(state: &PlanetState) -> Vec<FlowNode> {
    let mut nodes = Vec::new();
    let blocked: std::collections::HashSet<GridPos> =
        state.blocked_factories().into_iter().collect();
    for def in &state.data().buildings {
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
                priority: state
                    .grid
                    .get(pos)
                    .and_then(|tile| tile.building.as_ref())
                    .is_some_and(|building| building.input_priority),
                standby: state
                    .grid
                    .get(pos)
                    .and_then(|tile| tile.building.as_ref())
                    .is_some_and(|building| building.standby),
            });
        }
    }
    nodes.sort_by_key(|node| (node.pos.y, node.pos.x));
    nodes
}

pub(super) fn factory_flow_links(state: &PlanetState) -> Vec<FlowLink> {
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
    for def in &state.data().buildings {
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
            let peak_load = path
                .iter()
                .filter_map(|pos| state.traffic.get(&(pos.x, pos.y)))
                .copied()
                .max()
                .unwrap_or(0);
            links.push(FlowLink {
                resource,
                path,
                peak_load,
                capacity: state.config.buildings.conduit_capacity.max(1.0),
            });
        }
    }
    links
}

pub(super) fn recipe_readiness(state: &PlanetState, pos: GridPos, recipe: &RecipeDef) -> f32 {
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

pub(super) fn input_available(
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

pub(super) fn ordered_resources<'a>(ids: impl Iterator<Item = &'a str>) -> Vec<ResourceType> {
    let ids: std::collections::HashSet<&str> = ids.collect();
    ResourceType::ALL
        .into_iter()
        .filter(|resource| ids.contains(resource.id()))
        .collect()
}

pub(super) fn is_operational(state: &PlanetState, pos: GridPos) -> bool {
    let dust_response = state.resolved_dust_response();
    state
        .grid
        .get(pos)
        .and_then(|tile| tile.building.as_ref())
        .is_some_and(|building| {
            building.powered && !building.standby && !building.is_dust_stalled_with(&dust_response)
        })
}

pub(super) fn tile_center(pos: GridPos, metrics: HudMetrics) -> Vec2 {
    let (x, y) = grid_to_screen(pos, metrics);
    vec2(x + metrics.tile_size * 0.5, y + metrics.tile_size * 0.5)
}
