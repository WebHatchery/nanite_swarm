use super::LOG_ROW_H;
use crate::ui::{draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::notifications::{LoggedNotification, NotificationType};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width, ScrollArea};

/// Everything this world has been told, newest first.
///
/// A toast that faded while the player was reading the research tree used to be
/// the only copy of a message; this is where the messages go instead.
pub(super) fn draw_log(log: &[LoggedNotification], scroll: &mut ScrollArea, area: Rect) {
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
        let message = truncate_text_to_width(&entry.message, view.w.max(40.0), 11.0);
        draw_ui_text(
            &message,
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
