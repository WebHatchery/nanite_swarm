//! A single compact Seed Ship stage card.

use super::super::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

pub(super) fn draw(
    state: &PlanetState,
    index: usize,
    x: f32,
    y: f32,
    width: f32,
    pulse: f32,
    stage: &crate::data::SeedShipStageDef,
) {
    let ship = &state.seed_ship;
    let built = index < ship.stage_index();
    let active = index == ship.stage_index();
    let border = if built {
        Colors::SUCCESS
    } else if active {
        Color::new(0.0, 0.7 + pulse * 0.15, 0.9, 1.0)
    } else {
        Colors::SECONDARY
    };
    draw_panel(x, y, width, super::MOBILE_STAGE_HEIGHT);
    draw_rectangle_lines(x, y, width, super::MOBILE_STAGE_HEIGHT, 2.0, border);

    let text_w = (width - 28.0).max(40.0);
    draw_ui_text(
        &truncate_text_to_width(&format!("{}. {}", index + 1, stage.name), text_w, 13.0),
        x + 14.0,
        y + 20.0,
        13.0,
        if built || active {
            Colors::TEXT
        } else {
            Colors::TEXT_DIM
        },
    );
    draw_ui_text(
        &truncate_text_to_width(&stage.description, text_w, 10.0),
        x + 14.0,
        y + 39.0,
        10.0,
        Colors::TEXT_DIM,
    );
    if !stage.boon.is_empty() {
        draw_ui_text(
            &truncate_text_to_width(&stage.boon, text_w, 10.0),
            x + 14.0,
            y + 55.0,
            10.0,
            if built {
                Colors::SUCCESS
            } else {
                Colors::PRIMARY_SOFT
            },
        );
    }

    let waiting_on = if active {
        state.seed_ship_blocked_by()
    } else {
        None
    };
    let status = if built {
        "BUILT".to_string()
    } else if let Some(tech) = waiting_on {
        format!("NEEDS RESEARCH: {}", tech)
    } else if active {
        cost_summary(state, stage)
    } else if let Some(id) = stage.requires.as_deref() {
        let name = state
            .data()
            .research
            .nodes
            .iter()
            .find(|node| node.id == id)
            .map(|node| node.name.as_str())
            .unwrap_or(id);
        format!("SEALED - REQUIRES {}", name)
    } else {
        "SEALED".to_string()
    };
    draw_ui_text(
        &truncate_text_to_width(&status, text_w, 10.0),
        x + 14.0,
        y + 73.0,
        10.0,
        if built {
            Colors::SUCCESS
        } else if waiting_on.is_some() {
            Colors::ERROR
        } else if active {
            Colors::ACCENT
        } else {
            Colors::SECONDARY
        },
    );
}
