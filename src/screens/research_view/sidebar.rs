//! Research progress and desktop sidebar renderers.

use crate::engine::{ResearchState, ResearchTree};
use crate::state::StatReading;
use crate::ui::{draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::sheet::draw_swarm_sheet;

pub(super) fn draw_active_research(
    research_state: &ResearchState,
    research_tree: &ResearchTree,
    panel_x: f32,
    panel_w: f32,
    text_y: f32,
) -> f32 {
    if let Some(current) = &research_state.current_research {
        if let Some(node) = research_tree.get_node(current) {
            let progress = research_state.research_progress.min(node.data_cost);
            let pct = if node.data_cost > 0.0 {
                progress / node.data_cost
            } else {
                1.0
            };
            draw_ui_text(
                "Active Research",
                panel_x + 12.0,
                text_y,
                12.0,
                Colors::TEXT_DIM,
            );
            draw_ui_text(
                &node.name,
                panel_x + 12.0,
                text_y + 18.0,
                14.0,
                Colors::TEXT,
            );
            draw_rectangle(
                panel_x + 12.0,
                text_y + 32.0,
                panel_w - 24.0,
                10.0,
                Colors::SURFACE_DARK,
            );
            draw_rectangle(
                panel_x + 12.0,
                text_y + 32.0,
                (panel_w - 24.0) * pct,
                10.0,
                Colors::PRIMARY,
            );
            draw_rectangle_lines(
                panel_x + 12.0,
                text_y + 32.0,
                panel_w - 24.0,
                10.0,
                1.0,
                Colors::PANEL_BORDER,
            );
            return text_y + 60.0;
        }
    }

    draw_ui_text(
        "No research selected.",
        panel_x + 12.0,
        text_y,
        12.0,
        Colors::TEXT_DIM,
    );
    text_y + 24.0
}

pub(super) fn draw_desktop_legend(
    data: &crate::data::GameData,
    sheet: &[StatReading],
    panel_x: f32,
    panel_y: f32,
    panel_w: f32,
    panel_h: f32,
) {
    draw_panel(panel_x, panel_y, panel_w, panel_h);
    draw_ui_text(
        "Legend",
        panel_x + 12.0,
        panel_y + 28.0,
        16.0,
        Colors::PRIMARY,
    );
    for (label, y, color) in [
        ("Unlocked", 74.0, Colors::PRIMARY),
        ("In Progress", 116.0, Colors::WARNING),
        ("Available", 158.0, Colors::SUCCESS),
        ("Locked", 200.0, Colors::SECONDARY),
    ] {
        draw_ui_text(
            label,
            panel_x + 12.0,
            panel_y + y - 18.0,
            12.0,
            Colors::TEXT_DIM,
        );
        draw_circle(panel_x + 18.0, panel_y + y, 6.0, color);
    }
    draw_swarm_sheet(data, sheet, panel_x, panel_y + 232.0, panel_w);
}
