//! The solar map's shipping controls, and the pods crossing between orbits.

use crate::state::Shipment;
use crate::ui::{draw_button_sized, draw_panel, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, is_fully_visible, truncate_text_to_width, ScrollArea};

use super::{InterplanetaryAction, MapView};

mod order_editor;
mod route_status;
mod routes;

use order_editor::REMOTE_ROW_HEIGHT;

use route_status::route_note_count;
pub(super) use route_status::{remote_section_shift, route_status_color, route_status_for};
#[cfg(test)]
pub(super) use route_status::{reserve_status_text, route_status_label, target_pad_available};
pub(super) use routes::{draw_pods, draw_routes, route_hit_test, shipment_route_label};

#[cfg(test)]
pub(super) use routes::route_segment;

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
    let action = order_editor::draw(order_editor::OrderEditorInput {
        view,
        x,
        offset_y,
        w,
        content_view,
        interactive,
        edited_order,
        remote_worlds: &remote_worlds,
    });

    // The pod being loaded, so a route that is set but starved reads as
    // starved rather than as broken.
    let bar_width = w - 24.0;
    let bar_rect = Rect::new(x + 12.0, offset_y + bar_y, bar_width, 8.0);
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
            bar_width * pod_fraction.clamp(0.0, 1.0),
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

pub(super) fn draw_scrolled_button(
    rect: Rect,
    text: &str,
    content_view: Rect,
    interactive: bool,
) -> bool {
    if !is_fully_visible(rect, content_view) {
        return false;
    }
    draw_button_sized(rect.x, rect.y, rect.w, rect.h, text) && interactive
}

pub(super) fn draw_scrolled_text(
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    color: Color,
    content_view: Rect,
) {
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
