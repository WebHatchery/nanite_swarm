use super::{CARD_H, FOOTER_RESERVE, GRID_TO_LOG_GAP, LOG_MIN_H};
use crate::state::{AchievementRecord, ConsumptionTotals};
use crate::ui::{draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub(super) fn draw_consumption_summary(consumption: ConsumptionTotals, screen_w: f32) {
    if screen_w < 600.0 {
        draw_ui_text(
            &format!(
                "SYSTEM SINKS  M {} | P {} | D {}",
                format_consumption(consumption.minerals),
                format_consumption(consumption.energy),
                format_consumption(consumption.data),
            ),
            18.0,
            70.0,
            8.0,
            Colors::SUCCESS,
        );
        draw_ui_text(
            &format!(
                "B {} | A {} | C {}",
                format_consumption(consumption.biomass),
                format_consumption(consumption.alloy),
                format_consumption(consumption.components),
            ),
            18.0,
            84.0,
            8.0,
            Colors::SUCCESS,
        );
    } else {
        draw_ui_text(
            &format!(
                "SYSTEM SINKS  MIN {} | PWR {} | DATA {} | BIO {} | ALLOY {} | PARTS {}",
                format_consumption(consumption.minerals),
                format_consumption(consumption.energy),
                format_consumption(consumption.data),
                format_consumption(consumption.biomass),
                format_consumption(consumption.alloy),
                format_consumption(consumption.components),
            ),
            18.0,
            74.0,
            9.0,
            Colors::SUCCESS,
        );
    }
}

fn format_consumption(amount: f32) -> String {
    if amount >= 1_000.0 {
        format!("{:.1}k", amount / 1_000.0)
    } else if amount >= 100.0 {
        format!("{amount:.0}")
    } else if amount >= 10.0 {
        format!("{amount:.1}")
    } else {
        format!("{amount:.2}")
    }
}

pub(super) fn records_grid_height(screen_h: f32, area_y: f32) -> f32 {
    (screen_h - area_y - LOG_MIN_H - GRID_TO_LOG_GAP - FOOTER_RESERVE).max(CARD_H)
}

pub(super) fn record_card_fits_view(y: f32, height: f32, view: Rect) -> bool {
    y >= view.y && y + height <= view.bottom()
}

pub(super) fn draw_record(record: &AchievementRecord, x: f32, y: f32, w: f32) {
    draw_panel(x, y, w, CARD_H);

    let (name_color, body_color) = if record.unlocked {
        (Colors::SUCCESS, Colors::TEXT)
    } else {
        (Colors::TEXT, Colors::TEXT_DIM)
    };
    draw_ui_text(&record.name, x + 12.0, y + 24.0, 14.0, name_color);

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

    draw_ui_text(&record.description, x + 12.0, y + 43.0, 10.0, body_color);

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
