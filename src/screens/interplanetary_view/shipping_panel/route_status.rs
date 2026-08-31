//! Shared readiness and reserve copy for freight routes.

use crate::engine::ResourceType;
use crate::state::ExportOrder;
use crate::ui::Colors;
use macroquad::prelude::Color;

use super::super::MapView;

/// The most useful reason a remote route is not moving, in the order a player
/// can fix it. Source infrastructure wins over a destination warning because
/// a route cannot launch its first pod until its own driver exists and runs.
#[allow(clippy::too_many_arguments)]
pub(in crate::screens::interplanetary_view) fn route_status_label(
    drivers_built: usize,
    drivers_online: usize,
    pads_built: usize,
    pads_online: usize,
    pending_pods: usize,
    pod_cap: usize,
    overflow_pods: usize,
    target_pad_available: bool,
) -> &'static str {
    if drivers_built == 0 {
        "NO DRIVER"
    } else if drivers_online == 0 {
        "DRIVER OFFLINE"
    } else if pads_built == 0 {
        "NO PAD"
    } else if !target_pad_available {
        "PAD TARGET UNAVAILABLE"
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

pub(in crate::screens::interplanetary_view) fn route_status_color(status: &str) -> Color {
    match status {
        "NOMINAL" => Colors::SUCCESS,
        "DRIVER OFFLINE"
        | "PAD OFFLINE"
        | "PAD TARGET UNAVAILABLE"
        | "HOLDING"
        | "RESERVE HELD" => Colors::WARNING,
        _ => Colors::ERROR,
    }
}

pub(in crate::screens::interplanetary_view) fn route_status_for(
    view: &MapView,
    world: usize,
    order: ExportOrder,
) -> &'static str {
    let status = route_status_label(
        view.drivers_built.get(world).copied().unwrap_or(0),
        view.drivers_online.get(world).copied().unwrap_or(0),
        view.pads_built.get(order.target).copied().unwrap_or(0),
        view.pads.get(order.target).copied().unwrap_or(0),
        view.pending_pods.get(order.target).copied().unwrap_or(0),
        view.pod_caps.get(order.target).copied().unwrap_or(1),
        view.overflow_pods.get(order.target).copied().unwrap_or(0),
        target_pad_available(view, order),
    );
    if status == "NOMINAL" && source_reserve_text(view, world, order).is_some_and(|(_, held)| held)
    {
        "RESERVE HELD"
    } else {
        status
    }
}

pub(super) fn route_note_count(view: &MapView, world: usize, order: ExportOrder) -> usize {
    let mut count = usize::from(source_reserve_text(view, world, order).is_some());
    let target_pad_unavailable = !target_pad_available(view, order);
    count += usize::from(target_pad_unavailable);
    let pads_online = view.pads.get(order.target).copied().unwrap_or(0);
    if (!target_pad_unavailable && pads_online == 0)
        || view.pending_pods.get(order.target).copied().unwrap_or(0)
            >= view.pod_caps.get(order.target).copied().unwrap_or(1)
    {
        count += 1;
    }
    if view.overflow_pods.get(order.target).copied().unwrap_or(0) > 0 {
        count += 1;
    }
    count
}

pub(in crate::screens::interplanetary_view) fn target_pad_available(
    view: &MapView,
    order: ExportOrder,
) -> bool {
    order.target_pad.is_none_or(|target| {
        view.pad_positions
            .get(order.target)
            .is_some_and(|pads| pads.contains(&target))
    })
}

pub(in crate::screens::interplanetary_view) fn remote_section_shift(
    route_note_lines: usize,
) -> f32 {
    route_note_lines.saturating_sub(2) as f32 * 16.0
}

/// Read the central stockpile for the cargo a standing order is protecting.
/// Refined goods live in remote telemetry; minerals have the older stockpile
/// slice for the map list, so the panel can explain every accepted cargo type.
fn source_stockpile(view: &MapView, world: usize, cargo: ResourceType) -> Option<f32> {
    match cargo {
        ResourceType::Minerals => view.stockpiles.get(world).copied().flatten(),
        ResourceType::Biomass | ResourceType::Alloy | ResourceType::Components => view
            .telemetry
            .get(world)
            .copied()
            .flatten()
            .map(|telemetry| match cargo {
                ResourceType::Biomass => telemetry.biomass,
                ResourceType::Alloy => telemetry.alloy,
                ResourceType::Components => telemetry.components,
                _ => 0.0,
            }),
        _ => None,
    }
}

pub(super) fn source_reserve_text(
    view: &MapView,
    world: usize,
    order: ExportOrder,
) -> Option<(String, bool)> {
    if !order.surplus_only {
        return None;
    }
    let stockpile = source_stockpile(view, world, order.cargo)?;
    Some(reserve_status_text(stockpile, order.reserve_source))
}

pub(in crate::screens::interplanetary_view) fn reserve_status_text(
    stockpile: f32,
    reserve_source: f32,
) -> (String, bool) {
    let reserve = reserve_source.max(0.0);
    if stockpile <= reserve + 0.001 {
        (
            format!("Source reserve held: {:.0}/{:.0}", stockpile, reserve),
            true,
        )
    } else {
        (
            format!("Source surplus available: +{:.0}", stockpile - reserve),
            false,
        )
    }
}
