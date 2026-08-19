//! Records: every achievement the swarm can be told it has done, earned or not.
//!
//! The set has always been data and the simulation has always measured it, but
//! the only way to see any of it was a count in the operations panel and a
//! toast that faded. This is the list: what has been earned, what has not, and
//! how close the ones that are counted actually are.

use crate::state::{AchievementRecord, DirectiveRecord};
use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::notifications::{LoggedNotification, NotificationType};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, ScrollArea};

const HEADER_HEIGHT: f32 = 72.0;
const CARD_H: f32 = 76.0;
const CARD_GAP: f32 = 10.0;
/// Never fewer than this many across, and never so many that a card is a strip.
const MIN_COLUMNS: usize = 1;
const CARD_MIN_W: f32 = 260.0;
const LOG_ROW_H: f32 = 18.0;
/// The log gets whatever the grid leaves, down to this much.
const LOG_MIN_H: f32 = 120.0;

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
        (screen_h - area_y - LOG_MIN_H - 24.0).max(CARD_H),
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
        if y + CARD_H < area_y || y > grid_view.bottom() {
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
    let log_y = area_y + grid_h + 28.0;
    let log_h = screen_h - log_y - 36.0;
    if log_h >= LOG_MIN_H {
        draw_log(log, scroll, Rect::new(area_x, log_y, area_w, log_h));
    }

    draw_ui_text(
        "Drag the log to scroll | Tap Back to return",
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

/// Everything this world has been told, newest first.
///
/// A toast that faded while the player was reading the research tree used to be
/// the only copy of a message; this is where the messages go instead.
fn draw_log(log: &[LoggedNotification], scroll: &mut ScrollArea, area: Rect) {
    draw_panel(area.x, area.y, area.w, area.h);
    draw_ui_text("Log", area.x + 12.0, area.y + 24.0, 14.0, Colors::PRIMARY);
    let count = format!("{} entries", log.len());
    let count_w = measure_ui_text(&count, None, 10, 1.0).width;
    draw_ui_text(
        &count,
        area.x + area.w - count_w - 12.0,
        area.y + 24.0,
        10.0,
        Colors::TEXT_DIM,
    );

    let view = Rect::new(area.x + 12.0, area.y + 34.0, area.w - 24.0, area.h - 44.0);
    if log.is_empty() {
        draw_ui_text(
            "Nothing has happened worth reporting.",
            view.x,
            view.y + 18.0,
            10.0,
            Colors::TEXT_DIM,
        );
        return;
    }

    let content_h = log.len() as f32 * LOG_ROW_H;
    scroll.update(view, content_h);
    let offset = scroll.offset();

    for (index, entry) in log.iter().rev().enumerate() {
        let y = view.y + 14.0 + index as f32 * LOG_ROW_H - offset;
        if y < view.y || y > view.y + view.h {
            continue;
        }
        draw_ui_text(
            &entry.message,
            view.x,
            y,
            11.0,
            log_color(entry.notification_type),
        );
    }

    scroll.draw_scrollbar(view, content_h);
}

/// The log says the same thing the toast said, in the same colour it said it.
fn log_color(kind: NotificationType) -> Color {
    match kind {
        NotificationType::Success => Colors::SUCCESS,
        NotificationType::Warning => Colors::WARNING,
        NotificationType::Danger => Colors::ERROR,
        NotificationType::Info => Colors::TEXT,
    }
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
