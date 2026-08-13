//! Main menu screen

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
    Settings,
    Quit,
}

/// Render the main menu and return any action taken
pub fn render_main_menu(has_save: bool) -> MenuAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let float_y = bob(1.2, 6.0);

    // Ambient glow
    draw_circle(
        screen_w * 0.2,
        screen_h * 0.8,
        220.0,
        Color::new(0.0, 0.3, 0.4, 0.1),
    );
    draw_circle(
        screen_w * 0.75,
        screen_h * 0.25,
        160.0,
        Color::new(0.0, 0.2, 0.35, 0.12),
    );

    // Title
    let title = "NANITE SWARM";
    let _title_size = measure_ui_text(title, None, 48, 1.0);
    draw_ui_text(title, 40.0, 80.0 + float_y, 48.0, Colors::PRIMARY);

    // Subtitle
    let subtitle = "Consume. Evolve. Expand.";
    let _sub_size = measure_ui_text(subtitle, None, Dimensions::FONT_SIZE_NORMAL as u16, 1.0);
    draw_ui_text(
        subtitle,
        40.0,
        110.0 + float_y * 0.5,
        Dimensions::FONT_SIZE_NORMAL,
        Colors::TEXT_DIM,
    );

    // Briefing panel
    let briefing_w = 360.0;
    let briefing_h = 220.0;
    let briefing_x = 40.0;
    let briefing_y = 160.0;
    draw_panel(briefing_x, briefing_y, briefing_w, briefing_h);
    draw_ui_text(
        "Mission Briefing",
        briefing_x + 16.0,
        briefing_y + 28.0,
        18.0,
        Colors::PRIMARY,
    );
    draw_ui_text(
        "Build a self-sustaining nanite colony.",
        briefing_x + 16.0,
        briefing_y + 58.0,
        13.0,
        Colors::TEXT,
    );
    draw_ui_text(
        "Expand power, automate drills, and research.",
        briefing_x + 16.0,
        briefing_y + 78.0,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_ui_text(
        "Short sprints. Clear milestones.",
        briefing_x + 16.0,
        briefing_y + 98.0,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_ui_text(
        "Tip: Tap a building card, then tap the grid.",
        briefing_x + 16.0,
        briefing_y + 130.0,
        11.0,
        Colors::PRIMARY_SOFT,
    );

    // Menu panel
    let panel_w = 320.0;
    let panel_h = 340.0;
    let panel_x = screen_w - panel_w - 60.0;
    let panel_y = screen_h * 0.3 + float_y * 0.2;
    draw_panel(panel_x, panel_y, panel_w, panel_h);
    draw_ui_text(
        "Command Menu",
        panel_x + 18.0,
        panel_y + 30.0,
        18.0,
        Colors::PRIMARY,
    );

    // Buttons
    let btn_w = panel_w - 40.0;
    let btn_x = panel_x + 20.0;
    let mut btn_y = panel_y + 60.0;
    let btn_spacing = 46.0;

    if draw_button_sized(btn_x, btn_y, btn_w, 36.0, "New Game") {
        return MenuAction::NewGame;
    }
    btn_y += btn_spacing;

    if has_save && draw_button_sized(btn_x, btn_y, btn_w, 36.0, "Continue") {
        return MenuAction::Continue;
    }
    btn_y += btn_spacing;

    if draw_button_sized(btn_x, btn_y, btn_w, 36.0, "Load") {
        return MenuAction::Load;
    }
    btn_y += btn_spacing;

    if has_save && draw_button_sized(btn_x, btn_y, btn_w, 36.0, "Save") {
        return MenuAction::Save;
    }
    btn_y += btn_spacing;

    if draw_button_sized(btn_x, btn_y, btn_w, 36.0, "Settings") {
        return MenuAction::Settings;
    }
    btn_y += btn_spacing;

    if draw_button_sized(btn_x, btn_y, btn_w, 36.0, "Quit") {
        return MenuAction::Quit;
    }

    MenuAction::None
}
