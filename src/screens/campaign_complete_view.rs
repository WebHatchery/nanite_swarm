//! The end of the campaign: the system is spent and the ship has nowhere left
//! to go.

use crate::state::Campaign;
use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::math::pulse01;
use macroquad_toolkit::ui::draw_ui_text;

/// Actions from the ending screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CampaignCompleteAction {
    None,
    /// Stay in the finished campaign and keep building.
    KeepGoing,
    /// Back to the main menu.
    Close,
}

/// Render the ending. A pure view: it reads the campaign and returns a choice.
pub fn render_campaign_complete_view(campaign: &Campaign) -> CampaignCompleteAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let pulse = pulse01(3.0);

    draw_drifting_field(screen_w, screen_h, pulse);

    let panel_w = (screen_w * 0.55).clamp(460.0, 760.0);
    let panel_h = 330.0;
    let panel_x = (screen_w - panel_w) * 0.5;
    let panel_y = (screen_h - panel_h) * 0.42;
    draw_panel(panel_x, panel_y, panel_w, panel_h);
    draw_rectangle_lines(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        2.0,
        Color::new(0.0, 0.7 + pulse * 0.2, 0.9, 1.0),
    );

    draw_ui_text(
        "SYSTEM CONSUMED",
        panel_x + 28.0,
        panel_y + 48.0,
        24.0,
        Colors::PRIMARY,
    );
    draw_ui_text(
        "Every world in reach has been processed. The ship on the pad has",
        panel_x + 28.0,
        panel_y + 82.0,
        13.0,
        Colors::TEXT,
    );
    draw_ui_text(
        "nowhere to be sent. The swarm notes this without concern.",
        panel_x + 28.0,
        panel_y + 102.0,
        13.0,
        Colors::TEXT,
    );

    let hours = (campaign.total_time_played() / 3600.0).floor() as i64;
    let minutes = ((campaign.total_time_played() % 3600.0) / 60.0).floor() as i64;
    let rows = [
        ("Worlds consumed", format!("{}", crate::state::PLANET_COUNT)),
        (
            "Seed Ships launched",
            format!("{}", campaign.total_launches()),
        ),
        (
            "Structures standing",
            format!("{}", campaign.total_structures()),
        ),
        ("Time elapsed", format!("{}h {}m", hours, minutes)),
    ];
    for (index, (label, value)) in rows.iter().enumerate() {
        let y = panel_y + 146.0 + index as f32 * 24.0;
        draw_ui_text(label, panel_x + 28.0, y, 13.0, Colors::TEXT_DIM);
        draw_ui_text(value, panel_x + panel_w - 120.0, y, 13.0, Colors::SUCCESS);
    }

    let button_w = (panel_w - 76.0) * 0.5;
    let button_y = panel_y + panel_h - 62.0;
    let mut action = CampaignCompleteAction::None;
    if draw_button_sized(
        panel_x + 28.0,
        button_y,
        button_w,
        Dimensions::BUTTON_HEIGHT,
        "KEEP OPTIMISING",
    ) {
        action = CampaignCompleteAction::KeepGoing;
    }
    if draw_button_sized(
        panel_x + panel_w - 28.0 - button_w,
        button_y,
        button_w,
        Dimensions::BUTTON_HEIGHT,
        "MAIN MENU",
    ) {
        action = CampaignCompleteAction::Close;
    }

    if is_key_pressed(KeyCode::Escape) {
        action = CampaignCompleteAction::KeepGoing;
    }

    action
}

/// A slow drift of leftover matter, because the system is not there any more.
fn draw_drifting_field(screen_w: f32, screen_h: f32, pulse: f32) {
    for i in 0..140u32 {
        let x = (i as f32 * 53.1).sin().abs() * screen_w;
        let y = ((i as f32 * 29.7).cos().abs() * screen_h + pulse * 6.0) % screen_h;
        draw_circle(
            x,
            y,
            0.7 + (i % 4) as f32 * 0.35,
            Color::new(0.5, 0.75, 0.95, 0.08),
        );
    }
}
