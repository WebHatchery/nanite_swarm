//! Visual and hit-test helpers for the Solar System's freight lanes.
//!
//! The shipping panel owns standing-order controls and their status copy;
//! this module owns the map-facing representation of a route, a pod, and the
//! forgiving pointer test that selects a lane.

use crate::engine::ResourceType;
use crate::state::{cargo_name, ExportOrder, Shipment};
use crate::ui::Colors;
use macroquad::prelude::*;

/// The complete route on a pod label. Once more than one world is exporting,
/// a destination-only line is not enough to tell which standing order is
/// currently flying.
pub(crate) fn shipment_route_label(data: &crate::data::GameData, shipment: &Shipment) -> String {
    format!(
        "{:.0} {} {} -> {}",
        shipment.amount,
        cargo_name(shipment.cargo),
        data.planet(shipment.from).name,
        data.planet(shipment.to).name
    )
}

/// Every pod that is up, drawn where it actually is between the two worlds.
///
/// The map claimed to be a view of a running system while the only thing
/// moving on it was the planets; a pod crossing it is the system running.
pub(crate) fn draw_pods(shipments: &[Shipment], positions: &[(f32, f32)]) {
    for shipment in shipments {
        let (Some(from), Some(to)) = (
            positions.get(shipment.from).copied(),
            positions.get(shipment.to).copied(),
        ) else {
            continue;
        };
        let progress = shipment.progress();
        let px = from.0 + (to.0 - from.0) * progress;
        let py = from.1 + (to.1 - from.1) * progress;
        draw_line(
            from.0,
            from.1,
            to.0,
            to.1,
            1.0,
            Color::new(1.0, 0.42, 0.21, 0.18),
        );
        draw_circle(px, py, 3.0, Colors::ACCENT);
        draw_circle(px, py, 1.5, Colors::TEXT);
    }
}

/// Resolve a standing order into map coordinates without trusting a stale
/// save to point outside the currently authored system.
pub(crate) fn route_segment(
    source: usize,
    target: usize,
    positions: &[(f32, f32)],
) -> Option<((f32, f32), (f32, f32))> {
    if source == target {
        return None;
    }
    Some((
        positions.get(source).copied()?,
        positions.get(target).copied()?,
    ))
}

/// Draw the routes that exist even when every pod is on the ground. The
/// translucent lane is cargo-coloured and the arrowhead points at the target
/// world, so a settled network remains legible between shipments.
pub(crate) fn draw_routes(orders: &[Option<ExportOrder>], positions: &[(f32, f32)]) {
    for (source, order) in orders
        .iter()
        .enumerate()
        .filter_map(|(source, order)| order.as_ref().map(|order| (source, *order)))
    {
        let Some((from, to)) = route_segment(source, order.target, positions) else {
            continue;
        };
        let delta = vec2(to.0 - from.0, to.1 - from.1);
        let length = delta.length();
        if length < 1.0 {
            continue;
        }
        let direction = delta / length;
        let perpendicular = vec2(-direction.y, direction.x);
        let color = route_color(order.cargo);
        draw_line(from.0, from.1, to.0, to.1, 1.5, color);

        let tip = vec2(to.0, to.1) - direction * 10.0;
        let wing_start = tip - direction * 7.0;
        draw_line(
            tip.x,
            tip.y,
            (wing_start + perpendicular * 4.0).x,
            (wing_start + perpendicular * 4.0).y,
            1.5,
            color,
        );
        draw_line(
            tip.x,
            tip.y,
            (wing_start - perpendicular * 4.0).x,
            (wing_start - perpendicular * 4.0).y,
            1.5,
            color,
        );
    }
}

/// Find the closest standing lane under a pointer. The map gives the line a
/// forgiving hit radius because a one-pixel route is a signal to read, not a
/// touch target to thread with surgical accuracy.
pub(crate) fn route_hit_test(
    point: (f32, f32),
    orders: &[Option<ExportOrder>],
    positions: &[(f32, f32)],
    tolerance: f32,
) -> Option<usize> {
    let point = vec2(point.0, point.1);
    let tolerance = tolerance.max(0.0);
    let mut closest: Option<(usize, f32)> = None;
    for (source, order) in orders
        .iter()
        .enumerate()
        .filter_map(|(source, order)| order.as_ref().map(|order| (source, *order)))
    {
        let Some((from, to)) = route_segment(source, order.target, positions) else {
            continue;
        };
        let distance = point_to_segment_distance(point, vec2(from.0, from.1), vec2(to.0, to.1));
        if distance <= tolerance && closest.is_none_or(|(_, best)| distance < best) {
            closest = Some((source, distance));
        }
    }
    closest.map(|(source, _)| source)
}

fn point_to_segment_distance(point: Vec2, from: Vec2, to: Vec2) -> f32 {
    let segment = to - from;
    let length_squared = segment.length_squared();
    if length_squared <= f32::EPSILON {
        return point.distance(from);
    }
    let progress = ((point - from).dot(segment) / length_squared).clamp(0.0, 1.0);
    point.distance(from + segment * progress)
}

fn route_color(cargo: ResourceType) -> Color {
    match cargo {
        ResourceType::Minerals => Color::new(1.0, 0.42, 0.21, 0.36),
        ResourceType::Alloy => Color::new(1.0, 0.6, 0.0, 0.42),
        ResourceType::Components => Color::new(0.0, 0.851, 1.0, 0.42),
        _ => Color::new(0.67, 0.67, 0.67, 0.3),
    }
}
