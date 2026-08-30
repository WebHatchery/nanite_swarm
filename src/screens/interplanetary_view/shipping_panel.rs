//! The solar map's shipping controls, and the pods crossing between orbits.

use crate::engine::ResourceType;
use crate::state::{cargo_name, ExportOrder, Shipment};
use crate::ui::{draw_button_sized, draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, is_fully_visible, truncate_text_to_width, ScrollArea};

use super::status::{driver_operation_status, landing_pad_operation_status};
use super::{InterplanetaryAction, MapView};

/// How many in-flight pods the panel lists before it starts counting them.
const LISTED_SHIPMENTS: usize = 4;
const REMOTE_ROW_HEIGHT: f32 = 40.0;

/// The standing order for the world under the swarm, and the controls that
/// change it.
///
/// Only the current world: a driver is set up where it was built. What a world
/// keeps doing after the swarm leaves is the whole point, so the order stays
/// behind with it.
pub(super) fn draw(
    view: &MapView,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    scroll: &mut ScrollArea,
) -> InterplanetaryAction {
    draw_panel(x, y, w, h);
    draw_ui_text("Shipping", x + 12.0, y + 26.0, 16.0, Colors::PRIMARY);

    let home = view.data.planet(view.editing_world);
    let edited_order = view.orders.get(view.editing_world).copied().flatten();
    if !view.mass_driver_researched && edited_order.is_none() {
        draw_ui_text(
            "Requires: Mass Driver",
            x + 12.0,
            y + 50.0,
            12.0,
            Colors::ERROR,
        );
        return InterplanetaryAction::None;
    }

    let remote_worlds: Vec<usize> = view
        .orders
        .iter()
        .enumerate()
        .filter_map(|(world, order)| {
            (world != view.editing_world && order.is_some()).then_some(world)
        })
        .collect();
    let bar_y = 340.0f32.max(318.0 + remote_worlds.len() as f32 * REMOTE_ROW_HEIGHT + 8.0);
    let listed_flights = view.shipments.len().min(LISTED_SHIPMENTS);
    let extra_flight_line = if view.shipments.len() > LISTED_SHIPMENTS {
        16.0
    } else {
        0.0
    };
    let content_height =
        (bar_y + 46.0 + 20.0 + listed_flights as f32 * 16.0 + extra_flight_line + 12.0).max(420.0);
    let content_view = Rect::new(x + 8.0, y + 40.0, (w - 16.0).max(0.0), (h - 48.0).max(0.0));
    scroll.update(content_view, content_height);
    let offset_y = y - scroll.offset();
    let interactive = !scroll.absorbs_press();

    let pod_fraction = view
        .pod_fractions
        .get(view.editing_world)
        .copied()
        .unwrap_or(0.0);
    let drivers_online = view
        .drivers_online
        .get(view.editing_world)
        .copied()
        .unwrap_or(0);
    let drivers_built = view
        .drivers_built
        .get(view.editing_world)
        .copied()
        .unwrap_or(0);
    let status = driver_operation_status(drivers_built, drivers_online, home.name.as_str());
    let status_color = if drivers_online > 0 {
        Colors::SUCCESS
    } else if drivers_built > 0 {
        Colors::WARNING
    } else {
        Colors::TEXT_DIM
    };
    let rate = view
        .export_rates
        .get(view.editing_world)
        .copied()
        .unwrap_or(0.0);
    let status_line = if edited_order.is_some() {
        format!("{status}  |  {:.0}/min", rate)
    } else {
        format!("{status}  |  no route")
    };
    let status_line = truncate_text_to_width(&status_line, (w - 24.0).max(40.0), 11.0);
    draw_scrolled_text(
        &status_line,
        x + 12.0,
        offset_y + 48.0,
        11.0,
        status_color,
        content_view,
    );

    let button_w = w - 24.0;
    let cargo_label = match edited_order {
        Some(order) => format!("Cargo: {}", cargo_name(order.cargo)),
        None => "Cargo: --".to_string(),
    };
    let target_label = match edited_order {
        Some(order) => format!("Target: {}", view.data.planet(order.target).name),
        None => "Target: hold".to_string(),
    };

    let mut action = InterplanetaryAction::None;
    if draw_scrolled_button(
        Rect::new(x + 12.0, offset_y + 60.0, button_w, 28.0),
        &cargo_label,
        content_view,
        interactive,
    ) {
        action = if view.editing_world == view.current_planet {
            InterplanetaryAction::CycleExportCargo
        } else {
            InterplanetaryAction::CycleRemoteExportCargo
        };
    }
    if draw_scrolled_button(
        Rect::new(x + 12.0, offset_y + 94.0, button_w, 28.0),
        &target_label,
        content_view,
        interactive,
    ) {
        action = if view.editing_world == view.current_planet {
            InterplanetaryAction::CycleExportTarget
        } else {
            InterplanetaryAction::CycleRemoteExportTarget
        };
    }
    if draw_scrolled_button(
        Rect::new(x + 12.0, offset_y + 128.0, button_w, 28.0),
        &format!(
            "Landing pad: {}",
            edited_order
                .and_then(|order| order.target_pad)
                .map(|_| "specific")
                .unwrap_or("any")
        ),
        content_view,
        interactive,
    ) {
        action = InterplanetaryAction::CycleExportPad;
    }

    if draw_scrolled_button(
        Rect::new(x + 12.0, offset_y + 162.0, button_w, 28.0),
        &format!(
            "Cooldown: {:.0}s",
            edited_order
                .map(|order| order.schedule_seconds)
                .unwrap_or(0.0)
        ),
        content_view,
        interactive,
    ) {
        action = if view.editing_world == view.current_planet {
            InterplanetaryAction::CycleExportSchedule
        } else {
            InterplanetaryAction::CycleRemoteExportSchedule
        };
    }
    if draw_scrolled_button(
        Rect::new(x + 12.0, offset_y + 196.0, button_w, 28.0),
        &format!(
            "Priority: {}   {}",
            edited_order.map(|order| order.priority + 1).unwrap_or(1),
            if edited_order.is_some_and(|order| order.surplus_only) {
                "SURPLUS"
            } else {
                "ALL STOCK"
            }
        ),
        content_view,
        interactive,
    ) {
        action = if view.editing_world == view.current_planet {
            InterplanetaryAction::CycleExportPriority
        } else {
            InterplanetaryAction::CycleRemoteExportPriority
        };
    }
    if draw_scrolled_button(
        Rect::new(x + 12.0, offset_y + 230.0, button_w, 28.0),
        &format!(
            "Shipping reserve: {}",
            if edited_order.is_some_and(|order| order.surplus_only) {
                format!(
                    "{:.0}",
                    edited_order
                        .map(|order| order.reserve_source)
                        .unwrap_or(0.0)
                )
            } else {
                "off".to_string()
            }
        ),
        content_view,
        interactive,
    ) {
        action = if view.editing_world == view.current_planet {
            InterplanetaryAction::CycleExportReserve
        } else {
            InterplanetaryAction::CycleRemoteExportReserve
        };
    }

    // A route to a world with nothing to catch a pod is not an error, but the
    // pods will circle it until one gets built, so say so where it is set.
    if let Some(order) = edited_order {
        let pads_online = view.pads.get(order.target).copied().unwrap_or(0);
        let pads_built = view.pads_built.get(order.target).copied().unwrap_or(0);
        if pads_online == 0 {
            draw_scrolled_text(
                &landing_pad_operation_status(
                    pads_built,
                    pads_online,
                    view.data.planet(order.target).name.as_str(),
                ),
                x + 12.0,
                offset_y + 264.0,
                10.0,
                if pads_built > 0 {
                    Colors::WARNING
                } else {
                    Colors::ERROR
                },
                content_view,
            );
        } else if view.pending_pods.get(order.target).copied().unwrap_or(0)
            >= view.pod_caps.get(order.target).copied().unwrap_or(1)
        {
            draw_scrolled_text(
                "Destination queue full - pods hold in orbit",
                x + 12.0,
                offset_y + 264.0,
                10.0,
                Colors::WARNING,
                content_view,
            );
        }
        if view.overflow_pods.get(order.target).copied().unwrap_or(0) > 0 {
            draw_scrolled_text(
                "OVERFLOW: landing capacity is exhausted",
                x + 12.0,
                offset_y + 276.0,
                10.0,
                Colors::ERROR,
                content_view,
            );
        }
    }
    if !remote_worlds.is_empty() {
        draw_scrolled_text(
            "Remote standing orders",
            x + 12.0,
            offset_y + 300.0,
            10.0,
            Colors::PRIMARY,
            content_view,
        );
        let mut row_y = 318.0;
        for world in remote_worlds {
            let Some(order) = view.orders.get(world).copied().flatten() else {
                continue;
            };
            if draw_scrolled_button(
                Rect::new(x + 12.0, offset_y + row_y - 14.0, button_w, 24.0),
                &truncate_text_to_width(
                    &format!(
                        "{} -> {}",
                        view.data.planet(world).name,
                        view.data.planet(order.target).name
                    ),
                    (button_w - 24.0).max(40.0),
                    10.0,
                ),
                content_view,
                interactive,
            ) {
                action = InterplanetaryAction::SelectOrderWorld(world);
            }
            let rate = view.export_rates.get(world).copied().unwrap_or(0.0);
            let status = route_status_label(
                view.drivers_built.get(world).copied().unwrap_or(0),
                view.drivers_online.get(world).copied().unwrap_or(0),
                view.pads_built.get(order.target).copied().unwrap_or(0),
                view.pads.get(order.target).copied().unwrap_or(0),
                view.pending_pods.get(order.target).copied().unwrap_or(0),
                view.pod_caps.get(order.target).copied().unwrap_or(1),
                view.overflow_pods.get(order.target).copied().unwrap_or(0),
            );
            draw_scrolled_text(
                &format!(
                    "{} {:.0}/min | {status}",
                    cargo_name(order.cargo),
                    rate.max(0.0)
                ),
                x + 20.0,
                offset_y + row_y + 22.0,
                9.0,
                route_status_color(status),
                content_view,
            );
            row_y += REMOTE_ROW_HEIGHT;
        }
    }
    // The pod being loaded, so a route that is set but starved reads as
    // starved rather than as broken.
    let bar_rect = Rect::new(x + 12.0, offset_y + bar_y, button_w, 8.0);
    if is_fully_visible(bar_rect, content_view) {
        draw_rectangle(
            bar_rect.x,
            bar_rect.y,
            bar_rect.w,
            bar_rect.h,
            Colors::SURFACE_DARK,
        );
        draw_rectangle(
            bar_rect.x,
            bar_rect.y,
            button_w * pod_fraction.clamp(0.0, 1.0),
            bar_rect.h,
            Colors::ACCENT,
        );
    }
    let pod_text = if edited_order.is_none() {
        "No route set".to_string()
    } else if pod_fraction <= 0.0 {
        "Pod empty - route cargo to the driver".to_string()
    } else {
        format!("Pod {:.0}% loaded", pod_fraction * 100.0)
    };
    draw_scrolled_text(
        &pod_text,
        x + 12.0,
        offset_y + bar_y + 24.0,
        10.0,
        Colors::TEXT_DIM,
        content_view,
    );

    draw_in_flight_list(
        view.data,
        view.shipments,
        x,
        offset_y + bar_y + 46.0,
        w,
        content_view,
    );
    scroll.draw_scrollbar(content_view, content_height);

    action
}

/// The most useful reason a remote route is not moving, in the order a player
/// can fix it. Source infrastructure wins over a destination warning because
/// a route cannot launch its first pod until its own driver exists and runs.
pub(super) fn route_status_label(
    drivers_built: usize,
    drivers_online: usize,
    pads_built: usize,
    pads_online: usize,
    pending_pods: usize,
    pod_cap: usize,
    overflow_pods: usize,
) -> &'static str {
    if drivers_built == 0 {
        "NO DRIVER"
    } else if drivers_online == 0 {
        "DRIVER OFFLINE"
    } else if pads_built == 0 {
        "NO PAD"
    } else if pads_online == 0 {
        "PAD OFFLINE"
    } else if overflow_pods > 0 {
        "OVERFLOW"
    } else if pending_pods >= pod_cap.max(1) {
        "HOLDING"
    } else {
        "NOMINAL"
    }
}

fn route_status_color(status: &str) -> Color {
    match status {
        "NOMINAL" => Colors::SUCCESS,
        "DRIVER OFFLINE" | "PAD OFFLINE" | "HOLDING" => Colors::WARNING,
        _ => Colors::ERROR,
    }
}

fn draw_scrolled_button(rect: Rect, text: &str, content_view: Rect, interactive: bool) -> bool {
    if !is_fully_visible(rect, content_view) {
        return false;
    }
    draw_button_sized(rect.x, rect.y, rect.w, rect.h, text) && interactive
}

fn draw_scrolled_text(text: &str, x: f32, y: f32, size: f32, color: Color, content_view: Rect) {
    if y >= content_view.y && y <= content_view.bottom() {
        draw_ui_text(text, x, y, size, color);
    }
}

fn draw_in_flight_list(
    data: &crate::data::GameData,
    shipments: &[Shipment],
    x: f32,
    y: f32,
    w: f32,
    content_view: Rect,
) {
    draw_scrolled_text(
        "In flight | full route",
        x + 12.0,
        y,
        12.0,
        Colors::PRIMARY,
        content_view,
    );
    if shipments.is_empty() {
        draw_scrolled_text(
            "Nothing up",
            x + 12.0,
            y + 20.0,
            10.0,
            Colors::TEXT_DIM,
            content_view,
        );
        return;
    }

    let mut row_y = y + 20.0;
    for shipment in shipments.iter().take(LISTED_SHIPMENTS) {
        let label = shipment_route_label(data, shipment);
        draw_scrolled_text(
            &truncate_text_to_width(&label, (w - 70.0).max(50.0), 10.0),
            x + 12.0,
            row_y,
            10.0,
            Colors::TEXT,
            content_view,
        );
        // A pod that has arrived and found nowhere to land says so rather than
        // sitting on "0s" forever.
        let (eta, eta_color) = if shipment.is_holding() {
            ("HOLD".to_string(), Colors::WARNING)
        } else {
            (format!("{:.0}s", shipment.remaining), Colors::TEXT_DIM)
        };
        draw_scrolled_text(&eta, x + w - 46.0, row_y, 10.0, eta_color, content_view);
        row_y += 16.0;
    }
    if shipments.len() > LISTED_SHIPMENTS {
        draw_scrolled_text(
            &format!("+{} more", shipments.len() - LISTED_SHIPMENTS),
            x + 12.0,
            row_y,
            10.0,
            Colors::TEXT_DIM,
            content_view,
        );
    }
}

/// The complete route on a pod label. Once more than one world is exporting,
/// a destination-only line is not enough to tell which standing order is
/// currently flying.
pub(super) fn shipment_route_label(data: &crate::data::GameData, shipment: &Shipment) -> String {
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
pub(super) fn draw_pods(shipments: &[Shipment], positions: &[(f32, f32)]) {
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
pub(super) fn route_segment(
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
pub(super) fn draw_routes(orders: &[Option<ExportOrder>], positions: &[(f32, f32)]) {
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
pub(super) fn route_hit_test(
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
