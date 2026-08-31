//! Records: every achievement the swarm can be told it has done, earned or not.
//!
//! The set has always been data and the simulation has always measured it, but
//! the only way to see any of it was a count in the operations panel and a
//! toast that faded. This is the list: what has been earned, what has not, and
//! how close the ones that are counted actually are.

mod cards;
mod log;

use crate::state::{AchievementRecord, ConsumptionTotals, DirectiveRecord};
use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::notifications::LoggedNotification;
use macroquad_toolkit::ui::{draw_ui_text, ScrollArea};

use cards::{draw_consumption_summary, draw_record, record_card_fits_view, records_grid_height};
use log::draw_log;

const HEADER_HEIGHT: f32 = 96.0;
const CARD_H: f32 = 76.0;
const CARD_GAP: f32 = 10.0;
/// Never fewer than this many across, and never so many that a card is a strip.
const MIN_COLUMNS: usize = 1;
const CARD_MIN_W: f32 = 260.0;
const LOG_ROW_H: f32 = 18.0;
/// The log gets whatever the grid leaves, down to this much.
const LOG_MIN_H: f32 = 120.0;
const GRID_TO_LOG_GAP: f32 = 22.0;
const FOOTER_RESERVE: f32 = 36.0;

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
pub fn render_records_view(
    world: &str,
    records: &[AchievementRecord],
    log: &[LoggedNotification],
    directives: &[DirectiveRecord],
    consumption: ConsumptionTotals,
    scroll: &mut ScrollArea,
    records_scroll: &mut ScrollArea,
) -> RecordsAction {
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
    draw_consumption_summary(consumption, screen_w);

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

    let rows = records.len().div_ceil(columns);
    let grid_h = rows as f32 * (CARD_H + CARD_GAP);
    let grid_view = Rect::new(
        area_x,
        area_y,
        area_w,
        records_grid_height(screen_h, area_y),
    );
    records_scroll.update(grid_view, grid_h);
    let grid_offset = records_scroll.offset();
    for (index, record) in records.iter().enumerate() {
        let column = index % columns;
        let row = index / columns;
        let x = area_x + (card_w + CARD_GAP) * column as f32;
        let y = area_y + (CARD_H + CARD_GAP) * row as f32 - grid_offset;
        // A set that outgrows the screen is a scrolling problem, not a reason
        // to draw over the footer.
        if !record_card_fits_view(y, CARD_H, grid_view) {
            continue;
        }
        draw_record(record, x, y, card_w);
    }
    records_scroll.draw_scrollbar(grid_view, grid_h);

    let completed_directives = directives.iter().filter(|record| record.completed).count();
    draw_ui_text(
        &format!(
            "Directives: {} completed / {} expired",
            completed_directives,
            directives.len().saturating_sub(completed_directives)
        ),
        area_x,
        area_y + grid_view.h + 14.0,
        10.0,
        Colors::TEXT_DIM,
    );

    // The log takes the room the grid does not want. Two halves of the same
    // question: what the swarm has done, and what it has been told.
    let log_y = grid_view.bottom() + GRID_TO_LOG_GAP;
    let log_h = screen_h - log_y - 36.0;
    if log_h >= LOG_MIN_H {
        draw_log(log, scroll, Rect::new(area_x, log_y, area_w, log_h));
    }

    draw_records_footer(screen_w, screen_h);

    if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::A) {
        return RecordsAction::Close;
    }

    action
}

fn draw_records_footer(screen_w: f32, screen_h: f32) {
    if screen_w < 600.0 {
        draw_ui_text(
            "Drag RECORDS or LOG to scroll",
            20.0,
            screen_h - 28.0,
            Dimensions::FONT_SIZE_SMALL,
            Colors::TEXT_DIM,
        );
        draw_ui_text(
            "Tap BACK to return",
            20.0,
            screen_h - 12.0,
            Dimensions::FONT_SIZE_SMALL,
            Colors::TEXT_DIM,
        );
    } else {
        draw_ui_text(
            "Drag RECORDS or LOG to scroll | Tap BACK to return",
            20.0,
            screen_h - 20.0,
            Dimensions::FONT_SIZE_SMALL,
            Colors::TEXT_DIM,
        );
    }
}

#[cfg(test)]
mod tests;
