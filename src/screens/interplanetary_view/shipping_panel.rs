//! The solar map's shipping controls, and the pods crossing between orbits.

use crate::state::{cargo_name, Shipment};
use crate::ui::{draw_button_sized, draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use super::{InterplanetaryAction, MapView};

/// How many in-flight pods the panel lists before it starts counting them.
const LISTED_SHIPMENTS: usize = 4;

/// The standing order for the world under the swarm, and the controls that
/// change it.
///
/// Only the current world: a driver is set up where it was built. What a world
/// keeps doing after the swarm leaves is the whole point, so the order stays
/// behind with it.
pub(super) fn draw(view: &MapView, x: f32, y: f32, w: f32, h: f32) -> InterplanetaryAction {
    draw_panel(x, y, w, h);
    draw_ui_text("Shipping", x + 12.0, y + 26.0, 16.0, Colors::PRIMARY);

    let home = crate::data::game_data().planet(view.editing_world);
    let edited_order = view.orders.get(view.editing_world).copied().flatten();
    if !view.has_mass_driver && edited_order.is_none() {
        draw_ui_text(
            "Requires: Mass Driver",
            x + 12.0,
            y + 50.0,
            12.0,
            Colors::ERROR,
        );
        return InterplanetaryAction::None;
    }

    let (status, status_color) = match view.drivers_online {
        0 => ("No driver on this world".to_string(), Colors::TEXT_DIM),
        1 => (format!("1 driver on {}", home.name), Colors::SUCCESS),
        count => (
            format!("{} drivers on {}", count, home.name),
            Colors::SUCCESS,
        ),
    };
    draw_ui_text(&status, x + 12.0, y + 48.0, 11.0, status_color);

    let button_w = w - 24.0;
    let cargo_label = match edited_order {
        Some(order) => format!("Cargo: {}", cargo_name(order.cargo)),
        None => "Cargo: --".to_string(),
    };
    let target_label = match edited_order {
        Some(order) => format!(
            "Target: {}",
            crate::data::game_data().planet(order.target).name
        ),
        None => "Target: hold".to_string(),
    };

    let mut action = InterplanetaryAction::None;
    if draw_button_sized(x + 12.0, y + 60.0, button_w, 28.0, &cargo_label) {
        action = if view.editing_world == view.current_planet {
            InterplanetaryAction::CycleExportCargo
        } else {
            InterplanetaryAction::CycleRemoteExportCargo
        };
    }
    if draw_button_sized(x + 12.0, y + 94.0, button_w, 28.0, &target_label) {
        action = if view.editing_world == view.current_planet {
            InterplanetaryAction::CycleExportTarget
        } else {
            InterplanetaryAction::CycleRemoteExportTarget
        };
    }
    if draw_button_sized(
        x + 12.0,
        y + 128.0,
        button_w,
        28.0,
        &format!(
            "Landing pad: {}",
            edited_order
                .and_then(|order| order.target_pad)
                .map(|_| "specific")
                .unwrap_or("any")
        ),
    ) {
        action = InterplanetaryAction::CycleExportPad;
    }

    if draw_button_sized(
        x + 12.0,
        y + 162.0,
        button_w,
        28.0,
        &format!(
            "Cooldown: {:.0}s",
            edited_order
                .map(|order| order.schedule_seconds)
                .unwrap_or(0.0)
        ),
    ) {
        action = if view.editing_world == view.current_planet {
            InterplanetaryAction::CycleExportSchedule
        } else {
            InterplanetaryAction::CycleRemoteExportSchedule
        };
    }
    if draw_button_sized(
        x + 12.0,
        y + 196.0,
        button_w,
        28.0,
        &format!(
            "Priority: {}   {}",
            edited_order.map(|order| order.priority + 1).unwrap_or(1),
            if edited_order.is_some_and(|order| order.surplus_only) {
                "SURPLUS"
            } else {
                "ALL STOCK"
            }
        ),
    ) {
        action = if view.editing_world == view.current_planet {
            InterplanetaryAction::CycleExportPriority
        } else {
            InterplanetaryAction::CycleRemoteExportPriority
        };
    }
    if draw_button_sized(
        x + 12.0,
        y + 230.0,
        button_w,
        28.0,
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
            draw_ui_text(
                &format!(
                    "{} has no Landing Pad",
                    crate::data::game_data().planet(order.target).name
                ),
                x + 12.0,
                y + 264.0,
                10.0,
                Colors::ERROR,
            );
        } else if view.pending_pods.get(order.target).copied().unwrap_or(0)
            >= view.pod_caps.get(order.target).copied().unwrap_or(1)
        {
            draw_ui_text(
                "Destination queue full - pods hold in orbit",
                x + 12.0,
                y + 264.0,
                10.0,
                Colors::WARNING,
            );
        }
        if view.overflow_pods.get(order.target).copied().unwrap_or(0) > 0 {
            draw_ui_text(
                "OVERFLOW: landing capacity is exhausted",
                x + 12.0,
                y + 276.0,
                10.0,
                Colors::ERROR,
            );
        }
    }
    if view.orders.iter().any(Option::is_some) {
        draw_ui_text(
            "Remote standing orders",
            x + 12.0,
            y + 300.0,
            10.0,
            Colors::PRIMARY,
        );
        let mut row_y = y + 318.0;
        for (world, order) in view.orders.iter().enumerate() {
            if world == view.editing_world || order.is_none() {
                continue;
            }
            if draw_button_sized(
                x + 12.0,
                row_y - 12.0,
                button_w,
                22.0,
                crate::data::game_data().planet(world).name.as_str(),
            ) {
                action = InterplanetaryAction::SelectOrderWorld(world);
            }
            row_y += 26.0;
        }
    }
    // The pod being loaded, so a route that is set but starved reads as
    // starved rather than as broken.
    let bar_y = y + 340.0;
    draw_rectangle(x + 12.0, bar_y, button_w, 8.0, Colors::SURFACE_DARK);
    draw_rectangle(
        x + 12.0,
        bar_y,
        button_w * view.pod_fraction.clamp(0.0, 1.0),
        8.0,
        Colors::ACCENT,
    );
    let pod_text = if view.editing_world != view.current_planet || view.export.is_none() {
        "No route set".to_string()
    } else if view.pod_fraction <= 0.0 {
        "Pod empty - route cargo to the driver".to_string()
    } else {
        format!("Pod {:.0}% loaded", view.pod_fraction * 100.0)
    };
    draw_ui_text(&pod_text, x + 12.0, bar_y + 24.0, 10.0, Colors::TEXT_DIM);

    draw_in_flight_list(view.shipments, x, bar_y + 46.0, w);

    action
}

fn draw_in_flight_list(shipments: &[Shipment], x: f32, y: f32, w: f32) {
    draw_ui_text("In flight", x + 12.0, y, 12.0, Colors::PRIMARY);
    if shipments.is_empty() {
        draw_ui_text("Nothing up", x + 12.0, y + 20.0, 10.0, Colors::TEXT_DIM);
        return;
    }

    let data = crate::data::game_data();
    let mut row_y = y + 20.0;
    for shipment in shipments.iter().take(LISTED_SHIPMENTS) {
        draw_ui_text(
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
        );
        // A pod that has arrived and found nowhere to land says so rather than
        // sitting on "0s" forever.
        let (eta, eta_color) = if shipment.is_holding() {
            ("HOLD".to_string(), Colors::WARNING)
        } else {
            (format!("{:.0}s", shipment.remaining), Colors::TEXT_DIM)
        };
        draw_ui_text(&eta, x + w - 46.0, row_y, 10.0, eta_color);
        row_y += 16.0;
    }
    if shipments.len() > LISTED_SHIPMENTS {
        draw_ui_text(
            &format!("+{} more", shipments.len() - LISTED_SHIPMENTS),
            x + 12.0,
            row_y,
            10.0,
            Colors::TEXT_DIM,
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
