//! Desktop research-node inspection panel.

use crate::data::GameData;
use crate::engine::{ResearchState, ResearchTree};
use crate::ui::Colors;
use macroquad_toolkit::ui::draw_ui_text;

use super::compact::{active_research_status, parked_research_status};
use super::effects::node_effects;

/// Layout and state needed for the desktop node details column.
pub(super) struct DesktopNodeInfoInput<'a> {
    pub(super) data: &'a GameData,
    pub(super) research_state: &'a ResearchState,
    pub(super) research_tree: &'a ResearchTree,
    pub(super) node_id: &'a str,
    pub(super) heading: &'a str,
    pub(super) panel_x: f32,
    pub(super) y: f32,
    pub(super) data_available: f32,
    pub(super) active_condition: Option<&'a str>,
}

/// Draw the selected or hovered node in the roomy research sidebar.
pub(super) fn draw(input: DesktopNodeInfoInput<'_>) {
    let Some(node) = input.research_tree.get_node(input.node_id) else {
        return;
    };
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
    draw_ui_text(
        &node.description,
        input.panel_x + 12.0,
        input.y + 36.0,
        12.0,
        Colors::TEXT_DIM,
    );

    // What it actually does, under what it says it does.
    let mut y = input.y + 58.0;
    for (text, color) in node_effects(input.data, input.node_id) {
        draw_ui_text(&text, input.panel_x + 12.0, y, 12.0, color);
        y += 16.0;
    }
    y += 4.0;

    if !input.research_state.is_unlocked(input.node_id) {
        if let Some(status) = active_research_status(input.research_state, input.node_id, node) {
            draw_ui_text(&status, input.panel_x + 12.0, y, 12.0, Colors::WARNING);
        } else if let Some(status) =
            parked_research_status(input.research_state, input.node_id, node)
        {
            draw_ui_text(&status, input.panel_x + 12.0, y, 12.0, Colors::WARNING);
        } else {
            let cost_text = format!("Cost {:.0} Data", node.data_cost);
            draw_ui_text(&cost_text, input.panel_x + 12.0, y, 12.0, Colors::ACCENT);
            y += 18.0;

            if input.research_tree.can_select_on(
                input.node_id,
                &input.research_state.unlocked,
                input.active_condition,
            ) {
                if input.research_tree.can_research_on(
                    input.node_id,
                    &input.research_state.unlocked,
                    input.data_available,
                    input.active_condition,
                ) {
                    draw_ui_text(
                        "Tap node to research",
                        input.panel_x + 12.0,
                        y,
                        12.0,
                        Colors::SUCCESS,
                    );
                } else {
                    draw_ui_text(
                        "Tap node to select (insufficient Data)",
                        input.panel_x + 12.0,
                        y,
                        11.0,
                        Colors::WARNING,
                    );
                }
            } else if !node
                .prerequisites
                .iter()
                .all(|p| input.research_state.is_unlocked(p))
            {
                draw_ui_text(
                    "Prerequisites not met",
                    input.panel_x + 12.0,
                    y,
                    12.0,
                    Colors::ERROR,
                );
            } else {
                draw_ui_text(
                    "Not enough Data",
                    input.panel_x + 12.0,
                    y,
                    12.0,
                    Colors::WARNING,
                );
            }
        }
    } else {
        draw_ui_text("UNLOCKED", input.panel_x + 12.0, y, 12.0, Colors::SUCCESS);
    }
}
