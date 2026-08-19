use crate::state::{StatReading, StatUnit};
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

/// The resolved stat sheet, including the names of its contributing research
/// nodes where the panel has room to show provenance.
pub(super) fn draw_swarm_sheet(sheet: &[StatReading], panel_x: f32, panel_y: f32, panel_w: f32) {
    draw_ui_text("Swarm", panel_x + 12.0, panel_y, 14.0, Colors::PRIMARY);
    let mut y = panel_y + 24.0;
    for reading in sheet {
        let unit = StatUnit::of(reading.stat);
        let color = if !reading.is_changed() {
            Colors::TEXT_DIM
        } else if reading.is_gain() {
            Colors::SUCCESS
        } else {
            Colors::WARNING
        };
        draw_ui_text(
            reading.stat.label(),
            panel_x + 12.0,
            y,
            11.0,
            Colors::TEXT_DIM,
        );
        let value = unit.format(reading.value);
        let width = measure_ui_text(&value, None, 11, 1.0).width;
        draw_ui_text(&value, panel_x + panel_w - 12.0 - width, y, 11.0, color);
        y += 17.0;
        if !reading.sources.is_empty() && y < panel_y + panel_w {
            draw_ui_text(
                &format!("from {}", reading.sources.join(", ")),
                panel_x + 22.0,
                y,
                8.0,
                Colors::TEXT_DIM,
            );
            y += 11.0;
        }
    }
}
