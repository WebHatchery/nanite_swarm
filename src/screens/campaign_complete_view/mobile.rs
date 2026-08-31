//! Compact completion report for portrait screens.

use super::report::{
    compact_consumption_line, compact_core_stage_line, completion_world_rows, consumption_entries,
    format_consumption,
};
use super::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width};

const PANEL_X: f32 = 12.0;
const PANEL_Y: f32 = 86.0;
const PANEL_H: f32 = 520.0;

pub(super) fn draw(campaign: &Campaign, pulse: f32) -> CampaignCompleteAction {
    let screen_w = screen_width();
    let screen_h = screen_height();
    let (panel_y, panel_h) = mobile_panel_geometry(screen_h);
    let panel_w = (screen_w - PANEL_X * 2.0).max(120.0);
    draw_panel(PANEL_X, panel_y, panel_w, panel_h);
    draw_rectangle_lines(
        PANEL_X,
        panel_y,
        panel_w,
        panel_h,
        2.0,
        Color::new(0.0, 0.7 + pulse * 0.2, 0.9, 1.0),
    );

    draw_ui_text(
        "SYSTEM CONSUMED",
        PANEL_X + 16.0,
        panel_y + 38.0,
        21.0,
        Colors::PRIMARY,
    );
    let compact = panel_h < 500.0;
    let ultra_compact = panel_h < 390.0;
    let description = if ultra_compact {
        [
            "Every world in reach has been processed.",
            "The ship has nowhere to be sent.",
        ]
        .as_slice()
    } else {
        [
            "Every world in reach has been processed.",
            "The ship on the pad has nowhere to be sent.",
            "The swarm notes this without concern.",
        ]
        .as_slice()
    };
    for (index, line) in description.iter().enumerate() {
        draw_ui_text(
            &truncate_text_to_width(line, panel_w - 32.0, 11.0),
            PANEL_X + 16.0,
            panel_y
                + if ultra_compact { 62.0 } else { 70.0 }
                + index as f32 * if ultra_compact { 16.0 } else { 18.0 },
            11.0,
            if index == 2 {
                Colors::TEXT_DIM
            } else {
                Colors::TEXT
            },
        );
    }

    let hours = (campaign.total_time_played() / 3600.0).floor() as i64;
    let minutes = ((campaign.total_time_played() % 3600.0) / 60.0).floor() as i64;
    let rows = [
        ("Worlds consumed", crate::state::PLANET_COUNT.to_string()),
        ("Seed Ships launched", campaign.total_launches().to_string()),
        (
            "Structures standing",
            campaign.total_structures().to_string(),
        ),
        ("Time elapsed", format!("{}h {}m", hours, minutes)),
    ];
    for (index, (label, value)) in rows.iter().enumerate() {
        let y = panel_y
            + if ultra_compact { 94.0 } else { 156.0 }
            + index as f32 * if ultra_compact { 15.0 } else { 25.0 };
        draw_ui_text(label, PANEL_X + 16.0, y, 11.0, Colors::TEXT_DIM);
        let value_w = measure_ui_text(value, None, 11, 1.0).width;
        draw_ui_text(
            value,
            PANEL_X + panel_w - value_w - 16.0,
            y,
            11.0,
            Colors::SUCCESS,
        );
    }

    let world_rows = completion_world_rows(campaign);
    let button_y = panel_y + panel_h - 58.0;
    let world_header_y = if ultra_compact {
        // Keep the report balanced when a slightly taller short viewport has
        // spare space, while still following the buttons on genuinely tiny
        // screens.
        let world_header_y = (button_y - 28.0).min(panel_y + 222.0);
        draw_ui_text(
            "LIFETIME CONSUMPTION",
            PANEL_X + 16.0,
            world_header_y - 32.0,
            9.0,
            Colors::TEXT_DIM,
        );
        draw_ui_text(
            &truncate_text_to_width(&compact_consumption_line(campaign), panel_w - 32.0, 7.0),
            PANEL_X + 16.0,
            world_header_y - 15.0,
            7.0,
            Colors::SUCCESS,
        );
        world_header_y
    } else if compact {
        draw_ui_text(
            "LIFETIME CONSUMPTION",
            PANEL_X + 16.0,
            panel_y + 258.0,
            10.0,
            Colors::TEXT_DIM,
        );
        draw_ui_text(
            &truncate_text_to_width(&compact_consumption_line(campaign), panel_w - 32.0, 8.0),
            PANEL_X + 16.0,
            panel_y + 276.0,
            8.0,
            Colors::SUCCESS,
        );
        panel_y + 302.0
    } else {
        draw_ui_text(
            "LIFETIME CONSUMPTION",
            PANEL_X + 16.0,
            panel_y + 260.0,
            10.0,
            Colors::TEXT_DIM,
        );
        let entries = consumption_entries(campaign);
        for column in 0..2 {
            for row in 0..3 {
                let (label, amount) = entries[column * 3 + row];
                let y = panel_y + 280.0 + row as f32 * 16.0;
                let x = if column == 0 {
                    PANEL_X + 16.0
                } else {
                    PANEL_X + panel_w * 0.52
                };
                let value = format_consumption(amount);
                let value_width = measure_ui_text(&value, None, 9, 1.0).width;
                let value_x = if column == 0 {
                    PANEL_X + panel_w * 0.47 - value_width
                } else {
                    PANEL_X + panel_w - 16.0 - value_width
                };
                draw_ui_text(label, x, y, 9.0, Colors::TEXT);
                draw_ui_text(&value, value_x, y, 9.0, Colors::SUCCESS);
            }
        }
        panel_y + 342.0
    };
    draw_ui_text(
        "WORLD CORE LEDGER",
        PANEL_X + 16.0,
        world_header_y,
        10.0,
        Colors::TEXT_DIM,
    );
    draw_ui_text(
        &truncate_text_to_width(&compact_core_stage_line(&world_rows), panel_w - 32.0, 9.0),
        PANEL_X + 16.0,
        world_header_y + 20.0,
        9.0,
        Colors::PRIMARY,
    );

    let button_w = ((panel_w - 44.0) * 0.5).max(50.0);
    let keep_label = if panel_w < 300.0 {
        "KEEP GOING"
    } else {
        "KEEP OPTIMISING"
    };
    let mut action = CampaignCompleteAction::None;
    if draw_button_sized(
        PANEL_X + 16.0,
        button_y,
        button_w,
        Dimensions::BUTTON_HEIGHT,
        keep_label,
    ) {
        action = CampaignCompleteAction::KeepGoing;
    }
    if draw_button_sized(
        PANEL_X + panel_w - button_w - 16.0,
        button_y,
        button_w,
        Dimensions::BUTTON_HEIGHT,
        "MAIN MENU",
    ) {
        action = CampaignCompleteAction::Close;
    }
    action
}

fn mobile_panel_geometry(screen_h: f32) -> (f32, f32) {
    let panel_y = if screen_h < 560.0 { 36.0 } else { PANEL_Y };
    let panel_h = if screen_h < 560.0 {
        (screen_h - panel_y - 16.0).max(0.0)
    } else {
        PANEL_H
    };
    (panel_y, panel_h)
}

#[cfg(test)]
mod tests;
