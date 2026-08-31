//! Recipe buffers and processor controls shown by the structure inspector.

use crate::data::UiTheme;
use crate::engine::{BuildingType, GridPos, ResourceType};
use crate::state::PlanetState;
use crate::ui::{draw_hud_button, draw_resource_icon, resource_color};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RecipeFlowData {
    pub(super) inputs: Vec<(ResourceType, f32)>,
    pub(super) output: (ResourceType, f32),
    pub(super) output_capacity: f32,
    pub(super) input_priority: bool,
    pub(super) standby: bool,
}

pub(super) fn recipe_flow_data(
    state: &PlanetState,
    pos: GridPos,
    building_type: BuildingType,
) -> Option<RecipeFlowData> {
    let recipe = &state.data().building(building_type.id()).recipe;
    if recipe.is_empty() {
        return None;
    }

    let hoppers = state.input_hoppers.get(&(pos.x, pos.y));
    let inputs = recipe
        .carried_ids()
        .into_iter()
        .filter_map(|id| {
            let resource = ResourceType::from_id(id)?;
            let amount = hoppers
                .and_then(|values| values.get(&resource))
                .copied()
                .unwrap_or(0.0);
            Some((resource, amount))
        })
        .collect::<Vec<_>>();
    let output = ResourceType::ALL.into_iter().find(|resource| {
        recipe
            .outputs
            .get(resource.id())
            .is_some_and(|rate| *rate > 0.0)
    })?;
    let waiting = state
        .output_buffers
        .get(&(pos.x, pos.y))
        .copied()
        .unwrap_or(0.0);
    Some(RecipeFlowData {
        inputs,
        output: (output, waiting),
        output_capacity: state.processor_pad_capacity(),
        input_priority: state
            .grid
            .get(pos)
            .and_then(|tile| tile.building.as_ref())
            .is_some_and(|building| building.input_priority),
        standby: state
            .grid
            .get(pos)
            .and_then(|tile| tile.building.as_ref())
            .is_some_and(|building| building.standby),
    })
}

pub(super) fn recipe_status(flow: &RecipeFlowData, powered: bool) -> String {
    if flow.standby {
        return "Standby - buffers held".to_string();
    }
    if !powered {
        return "No power".to_string();
    }
    if flow.output.1 >= flow.output_capacity - 0.001 {
        return "Output pad full".to_string();
    }
    let missing: Vec<&str> = flow
        .inputs
        .iter()
        .filter(|(_, amount)| *amount <= 0.001)
        .map(|(resource, _)| flow_resource_name(*resource))
        .collect();
    if !missing.is_empty() {
        return format!("Needs {}", missing.join(" + "));
    }
    if flow.output.1 >= 1.0 {
        "Output waiting".to_string()
    } else {
        "Running".to_string()
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_recipe_flow_row(
    flow: &RecipeFlowData,
    theme: &UiTheme,
    x: f32,
    y: f32,
    width: f32,
    label_color: Color,
    warning: Color,
) -> (bool, bool) {
    let priority_toggled = draw_hud_button(
        theme,
        Rect::new(x, y - 15.0, 70.0, 20.0),
        if flow.input_priority {
            "PRIORITY"
        } else {
            "STANDARD"
        },
    );
    let standby_toggled = draw_hud_button(
        theme,
        Rect::new(x + 74.0, y - 15.0, 54.0, 20.0),
        if flow.standby { "RESUME" } else { "STANDBY" },
    );
    let token_w = 46.0;
    let mut cursor = (x + width * 0.49).max(x + 132.0);
    for (index, (resource, amount)) in flow.inputs.iter().enumerate() {
        if index > 0 {
            draw_ui_text("+", cursor - 7.0, y, 9.0, label_color);
        }
        draw_resource_icon(
            *resource,
            Rect::new(cursor, y - 9.0, 10.0, 10.0),
            resource_color(theme, *resource),
        );
        draw_ui_text(
            &format!(
                "{} {}",
                flow_resource_abbrev(*resource),
                compact_amount(*amount)
            ),
            cursor + 13.0,
            y,
            9.0,
            if *amount <= 0.001 {
                warning
            } else {
                resource_color(theme, *resource)
            },
        );
        cursor += token_w;
    }
    draw_ui_text(">", cursor - 7.0, y, 9.0, label_color);
    let (resource, amount) = flow.output;
    draw_resource_icon(
        resource,
        Rect::new(cursor + 2.0, y - 9.0, 10.0, 10.0),
        resource_color(theme, resource),
    );
    draw_ui_text(
        &format!(
            "{} {}",
            flow_resource_abbrev(resource),
            compact_amount(amount)
        ),
        cursor + 15.0,
        y,
        9.0,
        resource_color(theme, resource),
    );
    (priority_toggled, standby_toggled)
}

fn compact_amount(amount: f32) -> String {
    if amount < 10.0 && amount.fract().abs() > 0.05 {
        format!("{amount:.1}")
    } else {
        format!("{amount:.0}")
    }
}

fn flow_resource_abbrev(resource: ResourceType) -> &'static str {
    match resource {
        ResourceType::Minerals => "O",
        ResourceType::Energy => "P",
        ResourceType::Data => "D",
        ResourceType::Biomass => "B",
        ResourceType::Alloy => "A",
        ResourceType::Components => "C",
    }
}

fn flow_resource_name(resource: ResourceType) -> &'static str {
    match resource {
        ResourceType::Minerals => "Ore",
        ResourceType::Components => "Parts",
        _ => crate::state::cargo_name(resource),
    }
}
