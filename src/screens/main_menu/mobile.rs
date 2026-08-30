//! Single-column command menu for portrait screens.

use super::*;
use macroquad_toolkit::ui::truncate_text_to_width;

pub(super) fn draw(
    strings: &crate::data::PlayerStrings,
    has_save: bool,
    notice: Option<&str>,
    slot: &str,
    screen_w: f32,
    screen_h: f32,
) -> MenuAction {
    let edge = 12.0;
    let panel_w = (screen_w - edge * 2.0).max(120.0);
    let briefing_y = 150.0;
    let briefing_h = 58.0;
    draw_panel(edge, briefing_y, panel_w, briefing_h);
    draw_ui_text(
        &truncate_text_to_width(&strings.briefing_title, panel_w - 32.0, 14.0),
        edge + 16.0,
        briefing_y + 22.0,
        14.0,
        Colors::PRIMARY,
    );
    draw_ui_text(
        &truncate_text_to_width(
            strings
                .briefing_lines
                .first()
                .map(String::as_str)
                .unwrap_or("Build a self-sustaining nanite colony."),
            panel_w - 32.0,
            10.0,
        ),
        edge + 16.0,
        briefing_y + 43.0,
        10.0,
        Colors::TEXT_DIM,
    );

    let buttons = buttons(strings, has_save, slot);
    let panel_y = briefing_y + briefing_h + 10.0;
    let panel_h = 52.0 + buttons.len() as f32 * 34.0;
    draw_panel(edge, panel_y, panel_w, panel_h);
    draw_ui_text(
        &truncate_text_to_width(&strings.command_menu, panel_w - 32.0, 16.0),
        edge + 16.0,
        panel_y + 26.0,
        16.0,
        Colors::PRIMARY,
    );

    let button_x = edge + 12.0;
    let button_w = panel_w - 24.0;
    for (index, (label, action)) in buttons.iter().enumerate() {
        let y = panel_y + 36.0 + index as f32 * 34.0;
        if draw_button_sized(
            button_x,
            y,
            button_w,
            28.0,
            &truncate_text_to_width(label, button_w - 16.0, 11.0),
        ) {
            return *action;
        }
    }
    if let Some(notice) = notice {
        draw_ui_text(
            &truncate_text_to_width(notice, panel_w - 32.0, 10.0),
            edge + 16.0,
            (panel_y + panel_h + 18.0).min(screen_h - 34.0),
            10.0,
            Colors::WARNING,
        );
    }
    MenuAction::None
}

fn buttons(
    strings: &crate::data::PlayerStrings,
    has_save: bool,
    slot: &str,
) -> Vec<(String, MenuAction)> {
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
    buttons
}
