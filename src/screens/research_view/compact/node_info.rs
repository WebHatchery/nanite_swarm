//! Compact and mobile research-node detail rendering.

use crate::ui::Colors;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width, wrap_text};

use super::super::effects::node_effects;
use super::{active_research_status, parked_research_status, CompactNodeInfoInput};

const MOBILE_DESCRIPTION_LINE_HEIGHT: f32 = 13.0;
const MOBILE_DESCRIPTION_MAX_LINES: usize = 2;

pub(super) fn draw(input: CompactNodeInfoInput<'_>) {
    let Some(node) = input.research_tree.get_node(input.node_id) else {
        return;
    };
    if input.panel_w < 360.0 {
        draw_mobile(&input, node);
        return;
    }
    let split_x = input.panel_x + input.panel_w * 0.52;
    draw_ui_text(
        input.heading,
        input.panel_x + 12.0,
        input.y,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_ui_text(
        &node.name,
        input.panel_x + 12.0,
        input.y + 18.0,
        14.0,
        Colors::TEXT,
    );
    let description = truncate_text_to_width(&node.description, input.panel_w * 0.46, 11.0);
    draw_ui_text(
        &description,
        input.panel_x + 12.0,
        input.y + 38.0,
        11.0,
        Colors::TEXT_DIM,
    );
    let mut detail_y = input.y;
    for (text, color) in node_effects(input.data, input.node_id) {
        draw_ui_text(&text, split_x, detail_y, 11.0, color);
        detail_y += 16.0;
    }
    if input.research_state.is_unlocked(input.node_id) {
        draw_ui_text("UNLOCKED", split_x, detail_y + 2.0, 11.0, Colors::SUCCESS);
        return;
    }
    if let Some(status) = active_research_status(input.research_state, input.node_id, node) {
        draw_ui_text(&status, split_x, detail_y + 2.0, 11.0, Colors::WARNING);
        return;
    }
    if let Some(status) = parked_research_status(input.research_state, input.node_id, node) {
        draw_ui_text(
            &truncate_text_to_width(&status, input.panel_w * 0.46, 11.0),
            split_x,
            detail_y + 2.0,
            11.0,
            Colors::WARNING,
        );
        return;
    }
    draw_ui_text(
        &format!("Cost {:.0} Data", node.data_cost),
        split_x,
        detail_y + 2.0,
        11.0,
        Colors::ACCENT,
    );
    let status = node_status(&input, node);
    draw_ui_text(
        &truncate_text_to_width(&status.0, input.panel_w * 0.46, 11.0),
        split_x,
        detail_y + 18.0,
        11.0,
        status.1,
    );
}

fn node_status(
    input: &CompactNodeInfoInput<'_>,
    node: &crate::engine::ResearchNode,
) -> (String, macroquad::prelude::Color) {
    if node
        .planet_condition
        .as_deref()
        .is_some_and(|required| input.active_condition != Some(required))
    {
        (
            format!(
                "Requires {} world",
                node.planet_condition.as_deref().unwrap_or("matching")
            ),
            Colors::ERROR,
        )
    } else if !node
        .prerequisites
        .iter()
        .all(|id| input.research_state.is_unlocked(id))
    {
        ("Prerequisites not met".to_string(), Colors::ERROR)
    } else if input.research_tree.can_research_on(
        input.node_id,
        &input.research_state.unlocked,
        input.data_available,
        input.active_condition,
    ) {
        ("Tap node to research".to_string(), Colors::SUCCESS)
    } else {
        (
            "Tap node to select (insufficient Data)".to_string(),
            Colors::WARNING,
        )
    }
}

fn draw_mobile(input: &CompactNodeInfoInput<'_>, node: &crate::engine::ResearchNode) {
    let text_w = (input.panel_w - 24.0).max(40.0);
    draw_ui_text(
        input.heading,
        input.panel_x + 12.0,
        input.y,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_ui_text(
        &truncate_text_to_width(&node.name, text_w, 14.0),
        input.panel_x + 12.0,
        input.y + 18.0,
        14.0,
        Colors::TEXT,
    );
    let description_lines = mobile_description_lines(&node.description, text_w, 11.0);
    for (line_index, line) in description_lines.iter().enumerate() {
        draw_ui_text(
            line,
            input.panel_x + 12.0,
            input.y + 38.0 + line_index as f32 * MOBILE_DESCRIPTION_LINE_HEIGHT,
            11.0,
            Colors::TEXT_DIM,
        );
    }

    let mut detail_y =
        input.y + 38.0 + description_lines.len() as f32 * MOBILE_DESCRIPTION_LINE_HEIGHT + 7.0;
    for (text, color) in node_effects(input.data, input.node_id) {
        draw_ui_text(
            &truncate_text_to_width(&text, text_w, 11.0),
            input.panel_x + 12.0,
            detail_y,
            11.0,
            color,
        );
        detail_y += 15.0;
    }
    let (status, color) = if input.research_state.is_unlocked(input.node_id) {
        ("UNLOCKED".to_string(), Colors::SUCCESS)
    } else if let Some(status) = active_research_status(input.research_state, input.node_id, node) {
        (status, Colors::WARNING)
    } else if let Some(status) = parked_research_status(input.research_state, input.node_id, node) {
        (
            format!("Cost {:.0} Data - {}", node.data_cost, status),
            Colors::WARNING,
        )
    } else {
        let (status, color) = node_status(input, node);
        (format!("Cost {:.0} Data - {status}", node.data_cost), color)
    };
    draw_ui_text(
        &truncate_text_to_width(&status, text_w, 11.0),
        input.panel_x + 12.0,
        detail_y + 2.0,
        11.0,
        color,
    );
}

fn mobile_description_lines(text: &str, text_w: f32, font_size: f32) -> Vec<String> {
    let mut lines = wrap_text(text, text_w, font_size);
    let truncated = lines.len() > MOBILE_DESCRIPTION_MAX_LINES;
    lines.truncate(MOBILE_DESCRIPTION_MAX_LINES);
    if truncated {
        if let Some(last) = lines.last_mut() {
            *last = truncate_text_to_width(&format!("{last}..."), text_w, font_size);
        }
    }
    lines
}
