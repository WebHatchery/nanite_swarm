//! Records: every achievement the swarm can be told it has done, earned or not.
//!
//! The set has always been data and the simulation has always measured it, but
//! the only way to see any of it was a count in the operations panel and a
//! toast that faded. This is the list: what has been earned, what has not, and
//! how close the ones that are counted actually are.

use crate::state::AchievementRecord;
use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const HEADER_HEIGHT: f32 = 72.0;
const CARD_H: f32 = 76.0;
const CARD_GAP: f32 = 10.0;
/// Never fewer than this many across, and never so many that a card is a strip.
const MIN_COLUMNS: usize = 1;
const CARD_MIN_W: f32 = 260.0;

/// Actions from the records view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordsAction {
    None,
    Close,
}

/// Render the achievements screen.
///
/// A pure view: it reads the records it is handed and returns what the player
/// asked for.
pub fn render_records_view(world: &str, records: &[AchievementRecord]) -> RecordsAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();

    let earned = records.iter().filter(|record| record.unlocked).count();
    draw_panel(0.0, 0.0, screen_w, HEADER_HEIGHT);
    draw_ui_text("Records", 18.0, 30.0, 18.0, Colors::PRIMARY);
    draw_ui_text(
        &format!("{} - {} of {} earned", world, earned, records.len()),
        18.0,
        52.0,
        12.0,
        Colors::TEXT_DIM,
    );

    let mut action = RecordsAction::None;
    if draw_button_sized(screen_w - 110.0, 18.0, 80.0, 34.0, "Back") {
        action = RecordsAction::Close;
    }

    let area_x = 16.0;
    let area_w = screen_w - 32.0;
    let area_y = HEADER_HEIGHT + 16.0;

    let columns = ((area_w + CARD_GAP) / (CARD_MIN_W + CARD_GAP)).floor() as usize;
    let columns = columns.max(MIN_COLUMNS);
    let card_w = (area_w - CARD_GAP * (columns - 1) as f32) / columns as f32;

    for (index, record) in records.iter().enumerate() {
        let column = index % columns;
        let row = index / columns;
        let x = area_x + (card_w + CARD_GAP) * column as f32;
        let y = area_y + (CARD_H + CARD_GAP) * row as f32;
        // A set that outgrows the screen is a scrolling problem, not a reason
        // to draw over the footer.
        if y + CARD_H > screen_h - 40.0 {
            break;
        }
        draw_record(record, x, y, card_w);
    }

    draw_ui_text(
        "Press ESC or A to return",
        20.0,
        screen_h - 20.0,
        Dimensions::FONT_SIZE_SMALL,
        Colors::TEXT_DIM,
    );

    if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::A) {
        return RecordsAction::Close;
    }

    action
}

fn draw_record(record: &AchievementRecord, x: f32, y: f32, w: f32) {
    draw_panel(x, y, w, CARD_H);

    let (name_color, body_color) = if record.unlocked {
        (Colors::SUCCESS, Colors::TEXT)
    } else {
        (Colors::TEXT, Colors::TEXT_DIM)
    };
    draw_ui_text(record.name, x + 12.0, y + 24.0, 14.0, name_color);

    let marker = if record.unlocked { "EARNED" } else { "LOCKED" };
    let marker_w = measure_ui_text(marker, None, 10, 1.0).width;
    draw_ui_text(
        marker,
        x + w - marker_w - 12.0,
        y + 24.0,
        10.0,
        if record.unlocked {
            Colors::SUCCESS
        } else {
            Colors::TEXT_DIM
        },
    );

    draw_ui_text(record.description, x + 12.0, y + 43.0, 10.0, body_color);

    let bar_y = y + CARD_H - 22.0;
    let bar_w = w - 24.0;
    draw_rectangle(x + 12.0, bar_y, bar_w, 6.0, Colors::SURFACE_DARK);
    let fill = record.fraction();
    if fill > 0.0 {
        draw_rectangle(
            x + 12.0,
            bar_y,
            bar_w * fill,
            6.0,
            if record.unlocked {
                Colors::SUCCESS
            } else {
                Colors::PRIMARY
            },
        );
    }

    // Only where a running total means something. "0 / 1" for a one-shot
    // condition tells the player nothing they cannot see from the bar.
    if !record.unlocked && record.countable {
        if let Some(progress) = record.progress {
            let text = format!("{:.0} / {:.0}", progress, record.target);
            let text_w = measure_ui_text(&text, None, 10, 1.0).width;
            draw_ui_text(
                &text,
                x + w - text_w - 12.0,
                bar_y - 4.0,
                10.0,
                Colors::TEXT_DIM,
            );
        }
    }
}
