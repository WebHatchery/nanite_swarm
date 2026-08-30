//! Compact Seed Ship yard for portrait screens.

use super::*;
use macroquad_toolkit::ui::{draw_ui_text, is_fully_visible, truncate_text_to_width, ScrollArea};

const MOBILE_STAGE_HEIGHT: f32 = 88.0;
const MOBILE_STAGE_GAP: f32 = 8.0;

pub(super) fn draw(
    state: &PlanetState,
    scroll: &mut ScrollArea,
    pulse: f32,
    screen_h: f32,
    mut action: SeedShipAction,
) -> SeedShipAction {
    let screen_w = screen_width();
    let panel_x = 12.0;
    let panel_w = (screen_w - 24.0).max(80.0);
    let view = Rect::new(
        panel_x,
        HEADER_HEIGHT + 12.0,
        panel_w,
        (screen_h - HEADER_HEIGHT - 70.0).max(1.0),
    );
    let stage_count = state.data().seed_ship.stages.len();
    let stages_h = stage_count.saturating_sub(1) as f32 * (MOBILE_STAGE_HEIGHT + MOBILE_STAGE_GAP)
        + MOBILE_STAGE_HEIGHT;
    let content_h = stages_h + 92.0;
    scroll.update(view, content_h);
    let offset_y = view.y - scroll.offset();
    let interactive = !scroll.absorbs_press();

    for (index, stage) in state.data().seed_ship.stages.iter().enumerate() {
        let y = offset_y + index as f32 * (MOBILE_STAGE_HEIGHT + MOBILE_STAGE_GAP);
        if is_fully_visible(Rect::new(panel_x, y, panel_w, MOBILE_STAGE_HEIGHT), view) {
            draw_stage(state, index, panel_x, y, panel_w, pulse, stage);
        }
    }

    let footer_y = offset_y + stages_h + 12.0;
    if state.seed_ship.is_complete() {
        if footer_y + 44.0 >= view.y && footer_y <= view.bottom() {
            draw_ui_text(
                &truncate_text_to_width("The ship is whole. This world is spent.", panel_w, 12.0),
                panel_x,
                footer_y + 18.0,
                12.0,
                Colors::SUCCESS,
            );
        }
    } else {
        let label = if state.seed_ship.committed {
            "HALT DIVERSION"
        } else {
            "DIVERT PRODUCTION TO THE YARD"
        };
        let button = Rect::new(panel_x, footer_y, panel_w, Dimensions::BUTTON_HEIGHT);
        if is_fully_visible(button, view)
            && draw_button_sized(button.x, button.y, button.w, button.h, label)
            && interactive
        {
            action = SeedShipAction::ToggleCommitment;
        }
        let note = if state.seed_ship_blocked_by().is_some() {
            "Waiting on research before the yard can go on."
        } else if state.seed_ship.committed {
            "Harvest is being poured into the hull."
        } else {
            "The yard is idle; harvested resources stay in the pool."
        };
        if footer_y + 64.0 >= view.y && footer_y + 44.0 <= view.bottom() {
            draw_ui_text(
                &truncate_text_to_width(note, panel_w, 11.0),
                panel_x,
                footer_y + Dimensions::BUTTON_HEIGHT + 18.0,
                11.0,
                Colors::TEXT_DIM,
            );
        }
    }
    scroll.draw_scrollbar(view, content_h);

    if is_key_pressed(KeyCode::Escape) {
        action = SeedShipAction::Close;
    }
    if content_h > view.h {
        draw_ui_text(
            "Drag the yard to scroll",
            18.0,
            screen_h - 34.0,
            10.0,
            Colors::TEXT_DIM,
        );
    }
    draw_ui_text(
        "Tap Back to return",
        18.0,
        screen_h - 18.0,
        12.0,
        Colors::TEXT_DIM,
    );
    action
}

fn draw_stage(
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
    draw_panel(x, y, width, MOBILE_STAGE_HEIGHT);
    draw_rectangle_lines(x, y, width, MOBILE_STAGE_HEIGHT, 2.0, border);

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
