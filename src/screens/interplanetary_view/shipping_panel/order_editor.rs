//! Draw the standing order controls for the selected world.

use crate::state::{cargo_name, ExportOrder};
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::ui::truncate_text_to_width;

use super::route_status::{
    remote_section_shift, route_note_count, route_status_color, route_status_for,
    source_reserve_text, target_pad_available,
};
use super::{draw_scrolled_button, draw_scrolled_text, InterplanetaryAction, MapView};
use crate::screens::interplanetary_view::status::driver_operation_status;
use crate::screens::interplanetary_view::status::landing_pad_operation_status;

/// Vertical space reserved for each remote standing order row.
pub(super) const REMOTE_ROW_HEIGHT: f32 = 40.0;

/// All state and layout inputs needed to draw the order editor.
pub(super) struct OrderEditorInput<'view, 'data> {
    pub(super) view: &'view MapView<'data>,
    pub(super) x: f32,
    pub(super) offset_y: f32,
    pub(super) w: f32,
    pub(super) content_view: Rect,
    pub(super) interactive: bool,
    pub(super) edited_order: Option<ExportOrder>,
    pub(super) remote_worlds: &'view [usize],
}

/// Draw the selected world's order controls and the list of remote orders.
pub(super) fn draw(input: OrderEditorInput<'_, '_>) -> InterplanetaryAction {
    let OrderEditorInput {
        view,
        x,
        offset_y,
        w,
        content_view,
        interactive,
        edited_order,
        remote_worlds,
    } = input;
    let home = view.data.planet(view.editing_world);
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
        let route_note_lines = edited_order
            .map(|order| route_note_count(view, view.editing_world, order))
            .unwrap_or(0);
        let section_shift = remote_section_shift(route_note_lines);
        draw_scrolled_text(
            "Remote standing orders",
            x + 12.0,
            offset_y + 300.0 + section_shift,
            10.0,
            Colors::PRIMARY,
            content_view,
        );
        let mut row_y = 318.0 + section_shift;
        for &world in remote_worlds {
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

    action
}
