//! The solar map's shipping controls, and the pods crossing between orbits.

use crate::state::{cargo_name, Shipment};
use crate::ui::{draw_button_sized, draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, is_fully_visible, truncate_text_to_width, ScrollArea};

use super::status::{driver_operation_status, landing_pad_operation_status};
use super::{InterplanetaryAction, MapView};

mod route_status;
mod routes;

pub(super) use route_status::{
    remote_section_shift, route_status_color, route_status_for, target_pad_available,
};
#[cfg(test)]
pub(super) use route_status::{reserve_status_text, route_status_label};
use route_status::{route_note_count, source_reserve_text};
pub(super) use routes::{draw_pods, draw_routes, route_hit_test, shipment_route_label};

#[cfg(test)]
pub(super) use routes::route_segment;

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
    let route_note_lines = edited_order
        .map(|order| route_note_count(view, view.editing_world, order))
        .unwrap_or(0);
    let remote_section_shift = remote_section_shift(route_note_lines);
    let bar_y = 340.0f32
        .max(318.0 + remote_section_shift + remote_worlds.len() as f32 * REMOTE_ROW_HEIGHT + 8.0);
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
                .map(|order| {
                    if order.target_pad.is_some() && !target_pad_available(view, order) {
                        "unavailable"
                    } else if order.target_pad.is_some() {
                        "specific"
                    } else {
                        "any"
                    }
                })
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
    let mut route_note_y = offset_y + 264.0;
    if let Some(order) = edited_order {
        if let Some((reserve_text, held)) = source_reserve_text(view, view.editing_world, order) {
            draw_scrolled_text(
                &reserve_text,
                x + 12.0,
                route_note_y,
                10.0,
                if held {
                    Colors::WARNING
                } else {
                    Colors::SUCCESS
                },
                content_view,
            );
            route_note_y += 16.0;
        }
        let pads_online = view.pads.get(order.target).copied().unwrap_or(0);
        let pads_built = view.pads_built.get(order.target).copied().unwrap_or(0);
        let target_pad_unavailable = !target_pad_available(view, order);
        if target_pad_unavailable {
            draw_scrolled_text(
                "Selected landing pad unavailable - cycle Landing pad",
                x + 12.0,
                route_note_y,
                10.0,
                Colors::WARNING,
                content_view,
            );
            route_note_y += 16.0;
        }
        if pads_online == 0 && !target_pad_unavailable {
            draw_scrolled_text(
                &landing_pad_operation_status(
                    pads_built,
                    pads_online,
                    view.data.planet(order.target).name.as_str(),
                ),
                x + 12.0,
                route_note_y,
                10.0,
                if pads_built > 0 {
                    Colors::WARNING
                } else {
                    Colors::ERROR
                },
                content_view,
            );
            route_note_y += 16.0;
        } else if view.pending_pods.get(order.target).copied().unwrap_or(0)
            >= view.pod_caps.get(order.target).copied().unwrap_or(1)
        {
            draw_scrolled_text(
                "Destination queue full - pods hold in orbit",
                x + 12.0,
                route_note_y,
                10.0,
                Colors::WARNING,
                content_view,
            );
            route_note_y += 16.0;
        }
        if view.overflow_pods.get(order.target).copied().unwrap_or(0) > 0 {
            draw_scrolled_text(
                "OVERFLOW: landing capacity is exhausted",
                x + 12.0,
                route_note_y,
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
            offset_y + 300.0 + remote_section_shift,
            10.0,
            Colors::PRIMARY,
            content_view,
        );
        let mut row_y = 318.0 + remote_section_shift;
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
            let status = route_status_for(view, world, order);
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
