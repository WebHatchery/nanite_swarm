//! The solar map's shipping controls, and the pods crossing between orbits.

use crate::state::{cargo_name, Shipment};
use crate::ui::{draw_button_sized, draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, is_fully_visible, ScrollArea};

use super::status::driver_operation_status;
use super::{InterplanetaryAction, MapView};

/// How many in-flight pods the panel lists before it starts counting them.
const LISTED_SHIPMENTS: usize = 4;

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
    let bar_y = 340.0f32.max(318.0 + remote_worlds.len() as f32 * 26.0 + 8.0);
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
    draw_scrolled_text(
        &status,
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
            InterplanetaryAction::ToggleExportSurplus
        } else {
            InterplanetaryAction::ToggleRemoteExportSurplus
        };
    }

    // A route to a world with nothing to catch a pod is not an error, but the
    // pods will circle it until one gets built, so say so where it is set.
    if let Some(order) = edited_order {
        if view.pads.get(order.target).copied().unwrap_or(0) == 0 {
            draw_scrolled_text(
                &format!("{} has no Landing Pad", view.data.planet(order.target).name),
                x + 12.0,
                offset_y + 264.0,
                10.0,
                Colors::ERROR,
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
            if draw_scrolled_button(
                Rect::new(x + 12.0, offset_y + row_y - 12.0, button_w, 22.0),
                view.data.planet(world).name.as_str(),
                content_view,
                interactive,
            ) {
                action = InterplanetaryAction::SelectOrderWorld(world);
            }
            row_y += 26.0;
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
        "In flight",
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
        draw_scrolled_text(
            &format!(
                "{:.0} {} -> {}",
                shipment.amount,
                cargo_name(shipment.cargo),
                data.planet(shipment.to).name
            ),
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
