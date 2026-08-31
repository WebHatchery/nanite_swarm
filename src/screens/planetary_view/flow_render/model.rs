//! Live flow-model calculations and overlay drawing helpers.

use super::*;

mod drawing;

pub(super) fn draw_link(link: &FlowLink, metrics: HudMetrics, theme: &UiTheme, time: f32) {
    drawing::draw_link(link, metrics, theme, time);
}

pub(super) fn draw_node(node: &FlowNode, starved: bool, metrics: HudMetrics, theme: &UiTheme) {
    drawing::draw_node(node, starved, metrics, theme);
}

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
