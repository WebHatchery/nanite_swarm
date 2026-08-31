use crate::data::GameData;
use crate::engine::ResourceType;
use crate::state::PlanetState;
use macroquad::prelude::Color;
use macroquad_toolkit::ui::truncate_text_to_width;

pub(super) fn mobile_status_text(
    save_failed: bool,
    power_collapse: bool,
    hibernating: bool,
    arriving: bool,
    offline_work: &str,
    hazard: &str,
    world_name: &str,
) -> (String, bool) {
    if save_failed {
        ("SAVE FAILED".to_string(), false)
    } else if power_collapse {
        ("POWER COLLAPSE".to_string(), false)
    } else if hibernating {
        ("HIBERNATION".to_string(), false)
    } else if arriving {
        (format!("ARRIVED {}", world_name.to_uppercase()), true)
    } else if !offline_work.is_empty() {
        (offline_work.to_string(), true)
    } else {
        (hazard.to_string(), false)
    }
}

pub(super) fn mobile_research_status(
    current_research: Option<&str>,
    progress: f32,
    data: &GameData,
) -> Option<String> {
    let id = current_research?;
    let node = data.research.nodes.iter().find(|node| node.id == id)?;
    Some(format!(
        "R {:.0}/{:.0} DATA",
        progress.clamp(0.0, node.data_cost),
        node.data_cost
    ))
}

pub(super) fn mobile_arrival_guidance_label(guidance: &str) -> &'static str {
    if guidance.contains("tap RESEARCH") {
        "TAP RESEARCH"
    } else if guidance.contains("tap BUILD") {
        "TAP BUILD"
    } else {
        "FOLLOW ARRIVAL GUIDE"
    }
}

pub(super) fn compact_resource_amount(amount: f32) -> String {
    let amount = amount.max(0.0);
    if amount >= 1_000.0 {
        format!("{:.0}k", amount / 1_000.0)
    } else {
        format!("{amount:.0}")
    }
}

pub(super) fn resource_rate_label(
    state: &PlanetState,
    resource: ResourceType,
    rate: &str,
) -> String {
    if state.resource_is_full(resource) && rate.starts_with('+') {
        "FULL".to_string()
    } else {
        rate.to_string()
    }
}

pub(super) fn resource_card_color(
    state: &PlanetState,
    resource: ResourceType,
    normal: Color,
    warning: Color,
) -> Color {
    if state.resource_is_full(resource) {
        warning
    } else {
        normal
    }
}

pub(super) fn compact_resource_readout(state: &PlanetState) -> String {
    let compact = |resource: ResourceType| {
        let value = state.resources.get(resource);
        let capacity = state.resource_capacity(resource);
        format!(
            "{}/{}",
            compact_resource_amount(value),
            compact_resource_amount(capacity)
        )
    };
    format!(
        "E{} M{} D{} A{} C{} B{}",
        compact(ResourceType::Energy),
        compact(ResourceType::Minerals),
        compact(ResourceType::Data),
        compact(ResourceType::Alloy),
        compact(ResourceType::Components),
        compact(ResourceType::Biomass)
    )
}

pub(super) fn compact_resource_readout_for_screen(
    state: &PlanetState,
    screen_w: f32,
) -> (String, f32) {
    let font_size = if screen_w < 320.0 { 7.0 } else { 7.5 };
    let available_width = (screen_w - 166.0).max(40.0);
    (
        truncate_text_to_width(&compact_resource_readout(state), available_width, font_size),
        font_size,
    )
}

pub(super) fn compact_hidden_resource_readout(state: &PlanetState) -> String {
    let compact = |resource: ResourceType| {
        format!(
            "{}/{}",
            compact_resource_amount(state.resources.get(resource)),
            compact_resource_amount(state.resource_capacity(resource))
        )
    };
    format!(
        "A{} B{}",
        compact(ResourceType::Alloy),
        compact(ResourceType::Biomass)
    )
}
