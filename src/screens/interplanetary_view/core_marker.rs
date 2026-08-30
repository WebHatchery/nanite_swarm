//! Core evolution markers drawn on colonized Solar System nodes.

use crate::state::WorldTelemetry;
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::math::pulse01_at;
use macroquad_toolkit::settings::reduced_motion_enabled;
use macroquad_toolkit::ui::draw_ui_text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CoreStageMarker {
    stage: usize,
    stage_count: usize,
}

fn normalized_stage(stage: usize, stage_count: usize) -> (usize, usize) {
    let stage_count = stage_count.max(1);
    (stage.clamp(1, stage_count), stage_count)
}

fn marker_state(telemetry: Option<WorldTelemetry>) -> Option<CoreStageMarker> {
    let telemetry = telemetry?;
    let (stage, stage_count) = normalized_stage(telemetry.core_stage, telemetry.core_stage_count);
    Some(CoreStageMarker { stage, stage_count })
}

/// Give a colonized node a compact progress silhouette without changing its
/// existing hit target. The one-based stage label is deliberately explicit:
/// the dossier can explain the requirement, while the map can answer which
/// world is furthest along before the player opens another panel.
pub(super) fn draw_core_stage_marker(
    telemetry: Option<WorldTelemetry>,
    center: Vec2,
    planet_size: f32,
    orbit_scale: f32,
    time: f64,
) {
    let Some(marker) = marker_state(telemetry) else {
        return;
    };
    let radius = planet_size + (7.0 * orbit_scale).max(3.0);
    let motion = if reduced_motion_enabled() { 0.0 } else { 1.0 };
    let pulse = pulse01_at(time, 2.4) * motion;
    let ring_color = with_alpha(Colors::PRIMARY_SOFT, 0.38 + pulse * 0.18);
    draw_circle_lines(center.x, center.y, radius, 1.0, ring_color);

    let progress = marker.stage as f32 / marker.stage_count as f32;
    draw_progress_arc(
        center,
        radius + 1.5,
        progress,
        with_alpha(Colors::PRIMARY, 0.9),
    );

    if marker.stage >= 4 {
        let tether_top = center.y - radius - (8.0 * orbit_scale).max(4.0);
        draw_line(
            center.x,
            center.y - radius,
            center.x,
            tether_top,
            1.0,
            with_alpha(Colors::PRIMARY, 0.7),
        );
        draw_circle(center.x, tether_top, 1.8, with_alpha(Colors::PRIMARY, 0.85));
    }
    if marker.stage >= 5 {
        draw_ellipse_lines(
            center.x,
            center.y,
            radius * 1.65,
            (radius * 0.52).max(5.0),
            -12.0,
            1.0,
            with_alpha(Colors::PRIMARY, 0.6),
        );
    }

    draw_ui_text(
        &format!("C{}", marker.stage),
        center.x - 7.0,
        center.y - radius - (12.0 * orbit_scale.max(0.75)),
        9.0,
        Colors::PRIMARY,
    );
}

fn draw_progress_arc(center: Vec2, radius: f32, progress: f32, color: Color) {
    let segments = 18;
    let count = ((segments as f32 * progress.clamp(0.0, 1.0)).ceil() as usize).max(1);
    let start = -std::f32::consts::FRAC_PI_2;
    let step = std::f32::consts::TAU / segments as f32;
    for segment in 0..count.min(segments as usize) {
        let from = start + segment as f32 * step;
        let to = from + step * 0.78;
        draw_line(
            center.x + from.cos() * radius,
            center.y + from.sin() * radius,
            center.x + to.cos() * radius,
            center.y + to.sin() * radius,
            2.0,
            color,
        );
    }
}

#[cfg(test)]
mod tests;
