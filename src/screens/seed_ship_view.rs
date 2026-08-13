//! Seed Ship yard: the megastructure this world is being converted into

use crate::state::PlanetState;
use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::math::pulse01;
use macroquad_toolkit::ui::draw_ui_text;

const HEADER_HEIGHT: f32 = 72.0;
const STAGE_HEIGHT: f32 = 102.0;
const STAGE_GAP: f32 = 12.0;

/// Actions from the Seed Ship view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeedShipAction {
    None,
    Close,
    ToggleCommitment,
}

/// Render the Seed Ship construction screen.
///
/// A pure view: it reads the planet and returns what the player asked for.
pub fn render_seed_ship_view(state: &PlanetState) -> SeedShipAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let ship = &state.seed_ship;
    let pulse = pulse01(2.0);

    draw_starfield(screen_w, screen_h);

    draw_panel(0.0, 0.0, screen_w, HEADER_HEIGHT);
    draw_ui_text("Seed Ship", 18.0, 30.0, 18.0, Colors::PRIMARY);
    draw_ui_text(
        &format!(
            "{} - stage {} of {}",
            state.name,
            (ship.stage_index() + 1).min(ship.stage_count().max(1)),
            ship.stage_count()
        ),
        18.0,
        52.0,
        12.0,
        Colors::TEXT_DIM,
    );

    let mut action = SeedShipAction::None;
    if draw_button_sized(screen_w - 110.0, 18.0, 80.0, 34.0, "Back") {
        action = SeedShipAction::Close;
    }

    let panel_w = (screen_w * 0.56).clamp(420.0, 720.0);
    let panel_x = (screen_w - panel_w) * 0.5;
    let mut y = HEADER_HEIGHT + 28.0;

    for (index, stage) in crate::data::game_data().seed_ship.stages.iter().enumerate() {
        let built = index < ship.stage_index();
        let active = index == ship.stage_index();
        draw_stage(
            state, index, panel_x, y, panel_w, built, active, pulse, stage,
        );
        y += STAGE_HEIGHT + STAGE_GAP;
    }

    let footer_y = (y + 10.0).min(screen_h - 96.0);
    if ship.is_complete() {
        draw_ui_text(
            "The ship is whole. This world is spent; nothing here is worth another cycle.",
            panel_x,
            footer_y + 22.0,
            14.0,
            Colors::SUCCESS,
        );
    } else {
        let label = if ship.committed {
            "HALT DIVERSION"
        } else {
            "DIVERT PRODUCTION TO THE YARD"
        };
        if draw_button_sized(panel_x, footer_y, panel_w, Dimensions::BUTTON_HEIGHT, label) {
            action = SeedShipAction::ToggleCommitment;
        }
        let note = if state.seed_ship_blocked_by().is_some() {
            "The yard is waiting on research before it can go on."
        } else if ship.committed {
            "Harvest is being poured into the hull as it arrives."
        } else {
            "The yard is idle. Everything harvested stays in the pool."
        };
        draw_ui_text(
            note,
            panel_x,
            footer_y + Dimensions::BUTTON_HEIGHT + 20.0,
            12.0,
            Colors::TEXT_DIM,
        );
    }

    if is_key_pressed(KeyCode::Escape) {
        action = SeedShipAction::Close;
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

#[allow(clippy::too_many_arguments)]
fn draw_stage(
    state: &PlanetState,
    index: usize,
    x: f32,
    y: f32,
    width: f32,
    built: bool,
    active: bool,
    pulse: f32,
    stage: &crate::data::SeedShipStageDef,
) {
    let border = if built {
        Colors::SUCCESS
    } else if active {
        Color::new(0.0, 0.7 + pulse * 0.15, 0.9, 1.0)
    } else {
        Colors::SECONDARY
    };

    draw_panel(x, y, width, STAGE_HEIGHT);
    draw_rectangle_lines(x, y, width, STAGE_HEIGHT, 2.0, border);

    let label_color = if built || active {
        Colors::TEXT
    } else {
        Colors::TEXT_DIM
    };
    draw_ui_text(
        &format!("{}. {}", index + 1, stage.name),
        x + 14.0,
        y + 24.0,
        14.0,
        label_color,
    );
    draw_ui_text(
        &stage.description,
        x + 14.0,
        y + 44.0,
        11.0,
        Colors::TEXT_DIM,
    );
    // What standing it up does for this world. A stage that only costs is a
    // stage with no reason to be reached before the next one.
    if !stage.boon.is_empty() {
        draw_ui_text(
            &stage.boon,
            x + 14.0,
            y + 62.0,
            11.0,
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
    let status = match (built, active, waiting_on) {
        (true, _, _) => "BUILT".to_string(),
        // A stage nobody knows how to build yet says so rather than showing a
        // cost the player cannot pay towards.
        (_, true, Some(tech)) => format!("NEEDS RESEARCH: {}", tech),
        (_, true, None) => cost_summary(state, stage),
        _ => "SEALED".to_string(),
    };
    let status_color = if built {
        Colors::SUCCESS
    } else if waiting_on.is_some() {
        Colors::ERROR
    } else if active {
        Colors::ACCENT
    } else {
        Colors::SECONDARY
    };
    draw_ui_text(&status, x + 14.0, y + 84.0, 12.0, status_color);

    if active && waiting_on.is_none() {
        let bar_w = width - 28.0;
        let fraction = state.seed_ship.stage_fraction();
        draw_rectangle(
            x + 14.0,
            y + STAGE_HEIGHT - 10.0,
            bar_w,
            4.0,
            Colors::SURFACE_DARK,
        );
        draw_rectangle(
            x + 14.0,
            y + STAGE_HEIGHT - 10.0,
            bar_w * fraction,
            4.0,
            Colors::PRIMARY,
        );
    }
}

/// "Minerals 120/250  Data 12/40" for the stage being built.
fn cost_summary(state: &PlanetState, stage: &crate::data::SeedShipStageDef) -> String {
    let paid = state.seed_ship.progress();
    let cost = stage.cost;
    [
        ("Minerals", paid.minerals, cost.minerals),
        ("Data", paid.data, cost.data),
        ("Biomass", paid.biomass, cost.biomass),
        ("Alloy", paid.alloy, cost.alloy),
    ]
    .iter()
    .filter(|(_, _, needed)| *needed > 0.0)
    .map(|(label, have, needed)| format!("{} {:.0}/{:.0}", label, have, needed))
    .collect::<Vec<_>>()
    .join("   ")
}

fn draw_starfield(screen_w: f32, screen_h: f32) {
    for i in 0..90u32 {
        let x = (i as f32 * 41.3).sin().abs() * screen_w;
        let y = (i as f32 * 23.7).cos().abs() * screen_h;
        draw_circle(
            x,
            y,
            0.8 + (i % 3) as f32 * 0.4,
            Color::new(0.6, 0.8, 1.0, 0.10),
        );
    }
}
