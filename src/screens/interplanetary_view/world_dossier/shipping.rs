use crate::data::GameData;
use crate::state::{cargo_name, ExportOrder};

/// The standing freight order for a world, short enough to share its dossier
/// with the live telemetry and construction rules.
pub(crate) fn shipping_summary(
    data: &GameData,
    order: Option<ExportOrder>,
    rate_per_minute: f32,
) -> Option<String> {
    let order = order?;
    Some(format!(
        "Shipping: {} -> {} @ {:.0}/min",
        cargo_name(order.cargo),
        data.planet(order.target).name,
        rate_per_minute.max(0.0)
    ))
}

pub(crate) fn shipping_status_line(status: &str) -> String {
    format!("Route status: {status}")
}

pub(crate) fn compact_shipping_status(target: &str, status: &str) -> String {
    format!("Route: {target} {status}")
}
