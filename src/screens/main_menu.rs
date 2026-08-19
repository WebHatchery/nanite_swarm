//! Main menu screen

mod hero_art;

use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::math::bob;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

/// Actions that can be taken from the main menu
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuAction {
    None,
    NewGame,
    Continue,
    Load,
    Save,
    CycleSlot,
    Delete,
    Settings,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
}

/// Render the main menu and return any action taken
pub fn render_main_menu(has_save: bool, notice: Option<&str>, slot: &str) -> MenuAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let float_y = bob(1.2, 6.0);
    hero_art::draw(screen_w, screen_h, get_time() as f32);

    // Title
    let strings = crate::data::player_strings();
    let _title_size = measure_ui_text(&strings.title, None, 48, 1.0);
    draw_ui_text(&strings.title, 40.0, 80.0 + float_y, 48.0, Colors::PRIMARY);
    draw_ui_text(
        &format!("v{}", crate::release::BUILD_VERSION),
        42.0,
        132.0 + float_y * 0.4,
        10.0,
        Colors::TEXT_DIM,
    );

    // Subtitle
    let _sub_size = measure_ui_text(
        &strings.subtitle,
        None,
        Dimensions::FONT_SIZE_NORMAL as u16,
        1.0,
    );
    draw_ui_text(
        &strings.subtitle,
        40.0,
        110.0 + float_y * 0.5,
        Dimensions::FONT_SIZE_NORMAL,
        Colors::TEXT_DIM,
    );

    // Briefing panel
    let briefing_w = (screen_w * 0.3).clamp(300.0, 360.0);
    let briefing_h = 220.0;
    let briefing_x = 40.0;
    let briefing_y = 160.0;
    draw_panel(briefing_x, briefing_y, briefing_w, briefing_h);
    draw_ui_text(
        &strings.briefing_title,
        briefing_x + 16.0,
        briefing_y + 28.0,
        18.0,
        Colors::PRIMARY,
    );
    draw_ui_text(
        strings
            .briefing_lines
            .first()
            .map(String::as_str)
            .unwrap_or("Build a self-sustaining nanite colony."),
        briefing_x + 16.0,
        briefing_y + 58.0,
        13.0,
        Colors::TEXT,
    );
    draw_ui_text(
        strings
            .briefing_lines
            .get(1)
            .map(String::as_str)
            .unwrap_or("Expand power, automate drills, and research."),
        briefing_x + 16.0,
        briefing_y + 78.0,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_ui_text(
        strings
            .briefing_lines
            .get(2)
            .map(String::as_str)
            .unwrap_or("Short sprints. Clear milestones."),
        briefing_x + 16.0,
        briefing_y + 98.0,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_ui_text(
        &strings.briefing_tip,
        briefing_x + 16.0,
        briefing_y + 130.0,
        11.0,
        Colors::PRIMARY_SOFT,
    );

    // Menu panel
    let mut buttons = vec![
        (
            format!("Slot: {}  (tap to switch)", slot),
            MenuAction::CycleSlot,
        ),
        (strings.new_game.clone(), MenuAction::NewGame),
    ];
    if has_save {
        buttons.push((strings.r#continue.clone(), MenuAction::Continue));
    }
    buttons.push((strings.load.clone(), MenuAction::Load));
    if has_save {
        buttons.push((strings.save.clone(), MenuAction::Save));
    }
    buttons.push((strings.settings.clone(), MenuAction::Settings));
    if has_save {
        buttons.push((strings.delete_slot.clone(), MenuAction::Delete));
    }
    #[cfg(not(target_arch = "wasm32"))]
    buttons.push((strings.quit.clone(), MenuAction::Quit));

    let panel_w = 320.0;
    let panel_h = 78.0 + buttons.len() as f32 * 38.0;
    let panel_x = screen_w - panel_w - 40.0;
    let panel_y = ((screen_h - panel_h) * 0.5 + float_y * 0.2).max(88.0);
    draw_panel(panel_x, panel_y, panel_w, panel_h);
    draw_ui_text(
        &strings.command_menu,
        panel_x + 18.0,
        panel_y + 30.0,
        18.0,
        Colors::PRIMARY,
    );
    if let Some(notice) = notice {
        draw_ui_text(
            notice,
            panel_x + 20.0,
            panel_y + panel_h + 20.0,
            11.0,
            Colors::WARNING,
        );
    }

    // Buttons form one dense touch stack. Hidden save actions no longer leave
    // dead gaps that make the command panel look unfinished.
    let btn_w = panel_w - 40.0;
    let btn_x = panel_x + 20.0;
    let btn_spacing = 38.0;
    for (index, (label, action)) in buttons.iter().enumerate() {
        let y = panel_y + 60.0 + index as f32 * btn_spacing;
        if draw_button_sized(btn_x, y, btn_w, 30.0, label) {
            return *action;
        }
    }

    MenuAction::None
}
