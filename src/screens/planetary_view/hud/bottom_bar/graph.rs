//! Historical throughput, power, and Data graph rendering for the command bar.

#![allow(dead_code)]

use crate::data::UiTheme;
use crate::state::{GraphMode, PlanetState};
use crate::ui::{color_from_rgba, draw_hud_button};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::draw_ui_text;

/// Draw the selected graph and return whether its mode switch was pressed.
///
/// Ore banked at the Core is kept as a measured history rather than a
/// decorative curve. The same viewport also exposes power and Data so the
/// command bar can answer the three resource-flow questions without opening a
/// second screen.
pub(super) fn draw_throughput(
    state: &PlanetState,
    area: Rect,
    theme: &UiTheme,
    line: Color,
    text: Color,
) -> bool {
    let graph_mode = state.graph_mode;
    let buckets = state.throughput.buckets();
    let has_samples = match graph_mode {
        GraphMode::Flow => !buckets.is_empty() || !state.graph_samples.is_empty(),
        GraphMode::Power | GraphMode::Data => !state.graph_samples.is_empty(),
    };
    let switch_w = (area.w - 8.0).clamp(0.0, 54.0);
    let switch_rect = Rect::new(area.right() - switch_w - 4.0, area.y + 1.0, switch_w, 16.0);
    if !has_samples {
        draw_ui_text(
            match graph_mode {
                GraphMode::Flow => "NO FLOW YET",
                GraphMode::Power => "NO POWER SAMPLES",
                GraphMode::Data => "NO DATA SAMPLES",
            },
            area.x + 6.0,
            area.y + area.h * 0.62,
            9.0,
            text,
        );
        return switch_w >= 40.0 && draw_hud_button(theme, switch_rect, graph_mode.label());
    }

    // Always measured against zero, so a flat line at the bottom reads as
    // "nothing is arriving" rather than being stretched to fill the box.
    let peak = graph_peak(state, graph_mode);
    let plot_h = area.h - 10.0;
    if graph_mode == GraphMode::Flow {
        let step = area.w / buckets.len().max(2) as f32;
        for (index, bucket) in buckets.iter().enumerate() {
            let x = area.x + index as f32 * step;
            // The bucket's whole range, so a merged spike is still a spike.
            let low = area.y + area.h - 2.0 - (bucket.min / peak).clamp(0.0, 1.0) * plot_h;
            let high = area.y + area.h - 2.0 - (bucket.max / peak).clamp(0.0, 1.0) * plot_h;
            draw_line(x, low, x, high, step.max(1.0), with_alpha(line, 0.35));
            if index > 0 {
                let previous = buckets[index - 1].last;
                let y0 = area.y + area.h - 2.0 - (previous / peak).clamp(0.0, 1.0) * plot_h;
                let y1 = area.y + area.h - 2.0 - (bucket.last / peak).clamp(0.0, 1.0) * plot_h;
                draw_line(x - step, y0, x, y1, 1.0, line);
            }
        }
    }

    draw_ui_text(
        &latest_graph_readout(state, graph_mode),
        area.x + 4.0,
        area.y + 10.0,
        8.0,
        text,
    );
    if !state.graph_samples.is_empty() {
        draw_graph_series(state, area, peak, theme, graph_mode, line);
    }
    match graph_mode {
        GraphMode::Flow => draw_graph_legend(
            area,
            &[
                ("IN", line),
                ("O>", color_from_rgba(&theme.colors.minerals)),
                ("A+", color_from_rgba(&theme.colors.alloy)),
                ("A>", color_from_rgba(&theme.colors.warning)),
                ("C+", color_from_rgba(&theme.colors.components)),
            ],
        ),
        GraphMode::Power => draw_graph_legend(
            area,
            &[
                ("P+", color_from_rgba(&theme.colors.success)),
                ("P-", color_from_rgba(&theme.colors.error)),
            ],
        ),
        GraphMode::Data => draw_graph_legend(
            area,
            &[
                ("D+", color_from_rgba(&theme.colors.data)),
                ("D-", color_from_rgba(&theme.colors.warning)),
            ],
        ),
    }

    switch_w >= 40.0 && draw_hud_button(theme, switch_rect, graph_mode.label())
}

fn graph_peak(state: &PlanetState, mode: GraphMode) -> f32 {
    let peak = match mode {
        GraphMode::Flow => state.throughput.max().unwrap_or(0.0).max(
            state
                .graph_samples
                .iter()
                .flat_map(|sample| {
                    [
                        sample.minerals_consumed,
                        sample.alloy_produced,
                        sample.alloy_consumed,
                        sample.components_produced,
                        sample.components_consumed,
                    ]
                })
                .fold(0.0, f32::max),
        ),
        GraphMode::Power => state
            .graph_samples
            .iter()
            .flat_map(|sample| [sample.power_produced, sample.power_consumed])
            .fold(0.0, f32::max),
        GraphMode::Data => state
            .graph_samples
            .iter()
            .flat_map(|sample| [sample.data_produced, sample.data_consumed])
            .fold(0.0, f32::max),
    };
    peak.max(0.001)
}

fn latest_graph_readout(state: &PlanetState, mode: GraphMode) -> String {
    match mode {
        GraphMode::Flow => format!("IN {:.0}", state.throughput.last().unwrap_or(0.0)),
        GraphMode::Power => state
            .graph_samples
            .last()
            .map(|sample| {
                format!(
                    "+{:.0}/-{:.0}",
                    sample.power_produced, sample.power_consumed
                )
            })
            .unwrap_or_else(|| "NO PWR".to_string()),
        GraphMode::Data => state
            .graph_samples
            .last()
            .map(|sample| format!("+{:.0}/-{:.0}", sample.data_produced, sample.data_consumed))
            .unwrap_or_else(|| "NO DATA".to_string()),
    }
}

fn draw_graph_series(
    state: &PlanetState,
    area: Rect,
    peak: f32,
    theme: &UiTheme,
    mode: GraphMode,
    line: Color,
) {
    let count = state.graph_samples.len().max(2) as f32;
    let step = area.w / count;
    let series = match mode {
        GraphMode::Flow => vec![
            (0usize, color_from_rgba(&theme.colors.minerals)),
            (1, color_from_rgba(&theme.colors.alloy)),
            (2, color_from_rgba(&theme.colors.warning)),
            (3, color_from_rgba(&theme.colors.components)),
        ],
        GraphMode::Power => vec![
            (4usize, color_from_rgba(&theme.colors.success)),
            (5, color_from_rgba(&theme.colors.error)),
        ],
        GraphMode::Data => vec![
            (6usize, color_from_rgba(&theme.colors.data)),
            (7, color_from_rgba(&theme.colors.warning)),
        ],
    };
    for (kind, color) in series {
        for (index, pair) in state.graph_samples.windows(2).enumerate() {
            let x0 = area.x + index as f32 * step;
            let x1 = x0 + step;
            let read = |sample: &crate::state::GraphSample| match kind {
                0 => sample.minerals_consumed,
                1 => sample.alloy_produced,
                2 => sample.alloy_consumed,
                3 => sample.components_produced,
                4 => sample.power_produced,
                5 => sample.power_consumed,
                6 => sample.data_produced,
                _ => sample.data_consumed,
            };
            let y0 =
                area.y + area.h - 2.0 - (read(&pair[0]) / peak).clamp(0.0, 1.0) * (area.h - 10.0);
            let y1 =
                area.y + area.h - 2.0 - (read(&pair[1]) / peak).clamp(0.0, 1.0) * (area.h - 10.0);
            draw_line(x0, y0, x1, y1, 0.7, with_alpha(color, 0.55));
        }
    }
    draw_line(
        area.x + 4.0,
        area.bottom() - 5.0,
        area.x + 14.0,
        area.bottom() - 5.0,
        1.0,
        line,
    );
}

fn draw_graph_legend(area: Rect, legend: &[(&str, Color)]) {
    let legend_step = (area.w - 8.0) / legend.len().max(1) as f32;
    for (index, (label, color)) in legend.iter().enumerate() {
        draw_ui_text(
            label,
            area.x + 4.0 + index as f32 * legend_step,
            area.bottom() - 3.0,
            8.0,
            *color,
        );
    }
}

#[cfg(test)]
mod tests;
