//! Single-column command menu for portrait screens.

use super::*;
use macroquad_toolkit::ui::truncate_text_to_width;

pub(super) struct MobileMenuState<'a> {
    pub(super) has_save: bool,
    pub(super) has_active_campaign: bool,
    pub(super) notice: Option<&'a str>,
    pub(super) slot: &'a str,
    pub(super) delete_armed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MobileMenuLayout {
    briefing_y: f32,
    briefing_h: f32,
    panel_y: f32,
    panel_h: f32,
    button_h: f32,
    button_spacing: f32,
}

pub(super) fn draw(
    strings: &crate::data::PlayerStrings,
    state: MobileMenuState<'_>,
    screen_w: f32,
    screen_h: f32,
) -> MenuAction {
    let edge = 12.0;
    let panel_w = (screen_w - edge * 2.0).max(120.0);
    let compact = screen_h < 560.0;
    let buttons = buttons(
        strings,
        state.has_save,
        state.has_active_campaign,
        state.slot,
        state.delete_armed,
    );
    let layout = mobile_menu_layout(screen_w, screen_h, buttons.len());
    let briefing_y = layout.briefing_y;
    let briefing_h = layout.briefing_h;
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
        briefing_y + if compact { 34.0 } else { 43.0 },
        10.0,
        Colors::TEXT_DIM,
    );

    let panel_y = layout.panel_y;
    let button_h = layout.button_h;
    let button_spacing = layout.button_spacing;
    let panel_h = layout.panel_h;
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
        let y = panel_y + if compact { 34.0 } else { 36.0 } + index as f32 * button_spacing;
        if draw_button_sized(
            button_x,
            y,
            button_w,
            button_h,
            &truncate_text_to_width(label, button_w - 16.0, 11.0),
        ) {
            return *action;
        }
    }
    if let Some(notice) = state.notice {
        let notice_y = if compact {
            panel_y - 6.0
        } else {
            (panel_y + panel_h + 18.0).min(screen_h - 34.0)
        };
        draw_ui_text(
            &truncate_text_to_width(notice, panel_w - 32.0, 10.0),
            edge + 16.0,
            notice_y,
            10.0,
            Colors::WARNING,
        );
    }
    MenuAction::None
}

fn mobile_menu_layout(screen_w: f32, screen_h: f32, button_count: usize) -> MobileMenuLayout {
    let compact = screen_h < 560.0;
    let narrow = screen_w < 300.0;
    let extra_compact = narrow || screen_h < 460.0;
    let briefing_y = if narrow {
        140.0
    } else if compact {
        136.0
    } else {
        150.0
    };
    let briefing_h = if narrow {
        38.0
    } else if compact {
        42.0
    } else {
        58.0
    };
    let panel_y = briefing_y
        + briefing_h
        + if extra_compact {
            4.0
        } else if compact {
            12.0
        } else {
            10.0
        };
    let button_h = if extra_compact {
        20.0
    } else if compact {
        24.0
    } else {
        28.0
    };
    let button_spacing = if extra_compact {
        21.0
    } else if compact {
        27.0
    } else {
        34.0
    };
    let panel_header_h = if extra_compact {
        42.0
    } else if compact {
        48.0
    } else {
        52.0
    };
    let panel_h = panel_header_h + button_count as f32 * button_spacing;

    MobileMenuLayout {
        briefing_y,
        briefing_h,
        panel_y,
        panel_h,
        button_h,
        button_spacing,
    }
}

fn buttons(
    strings: &crate::data::PlayerStrings,
    has_save: bool,
    has_active_campaign: bool,
    slot: &str,
    delete_armed: bool,
) -> Vec<(String, MenuAction)> {
    let mut buttons = vec![
        (
            super::slot_button_label(slot, has_save),
            MenuAction::CycleSlot,
        ),
        (strings.new_game.clone(), MenuAction::NewGame),
    ];
    if has_save {
        buttons.push((strings.r#continue.clone(), MenuAction::Continue));
    }
    buttons.push((strings.load.clone(), MenuAction::Load));
    if super::show_save_button(has_active_campaign) {
        buttons.push((strings.save.clone(), MenuAction::Save));
    }
    buttons.push((strings.settings.clone(), MenuAction::Settings));
    if has_save {
        buttons.push((
            super::delete_slot_label(strings, delete_armed),
            MenuAction::Delete,
        ));
    }
    #[cfg(not(target_arch = "wasm32"))]
    buttons.push((strings.quit.clone(), MenuAction::Quit));
    buttons
}

#[cfg(test)]
mod tests;
