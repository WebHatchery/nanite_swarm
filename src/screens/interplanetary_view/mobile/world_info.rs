//! Touch-sized dossier details for one world in the Solar System list.

use crate::data::PlanetDef;
use crate::state::{ExportOrder, WorldTelemetry};
use crate::ui::Colors;
use macroquad::prelude::*;

use super::super::shipping_panel::route_status_for;
use super::super::status::launch_prompt;
use super::super::world_dossier::{shipping_summary, world_alert_code};
use super::super::MapView;
use super::{mobile_status_text, SHORT_MOBILE_HEIGHT};

mod compact;
mod medium;
mod tall;

struct MobileWorldInfo<'view, 'data> {
    view: &'view MapView<'data>,
    index: usize,
    can_launch: bool,
    x: f32,
    y: f32,
    width: f32,
    planet: &'view PlanetDef,
    is_colonized: bool,
    is_current: bool,
    telemetry: Option<WorldTelemetry>,
    order: Option<ExportOrder>,
    route_status: Option<&'static str>,
    shipping: Option<String>,
    status_line: String,
    status_color: Color,
    panel_h: f32,
    text_w: f32,
}

impl<'view, 'data> MobileWorldInfo<'view, 'data> {
    fn new(
        view: &'view MapView<'data>,
        index: usize,
        can_launch: bool,
        x: f32,
        y: f32,
        width: f32,
        screen_h: f32,
    ) -> Self {
        let planet = &view.data.planets[index];
        let is_colonized = view.colonized.get(index).copied().unwrap_or(false);
        let is_current = index == view.current_planet;
        let telemetry = view.telemetry.get(index).copied().flatten();
        let alert_code = telemetry
            .filter(|telemetry| telemetry.alerts > 0)
            .map(|telemetry| world_alert_code(Some(telemetry)));
        let status_line = mobile_status_text(is_colonized, alert_code.as_deref());
        let status_color = if alert_code.is_some() {
            Colors::ERROR
        } else if is_colonized {
            Colors::SUCCESS
        } else {
            Colors::WARNING
        };
        let order = view.orders.get(index).copied().flatten();
        let shipping = shipping_summary(
            view.data,
            order,
            view.export_rates.get(index).copied().unwrap_or(0.0),
        );
        let route_status = order.map(|order| route_status_for(view, index, order));
        let panel_h = mobile_info_height(screen_h, telemetry.is_some(), shipping.is_some());
        Self {
            view,
            index,
            can_launch,
            x,
            y,
            width,
            planet,
            is_colonized,
            is_current,
            telemetry,
            order,
            route_status,
            shipping,
            status_line,
            status_color,
            panel_h,
            text_w: (width - 30.0).max(40.0),
        }
    }

    fn target_name(&self) -> Option<&str> {
        self.order
            .map(|order| self.view.data.planet(order.target).name.as_str())
    }

    fn target_is_current(&self) -> bool {
        self.order
            .is_some_and(|order| order.target == self.view.current_planet)
    }

    fn action(&self, detailed: bool) -> String {
        if let Some(action) = mobile_route_action(
            &self.planet.name,
            self.target_name(),
            self.route_status,
            self.is_current,
            self.target_is_current(),
            self.is_colonized,
        ) {
            action
        } else if self.can_launch {
            launch_prompt(self.view.launch_confirmation_target == Some(self.index)).to_string()
        } else if detailed && !self.view.mass_driver_researched {
            "Requires Mass Driver research".to_string()
        } else if detailed && self.view.current_drivers_online == 0 {
            "Requires a powered Mass Driver".to_string()
        } else if detailed {
            "Requires a finished Seed Ship".to_string()
        } else {
            "World not ready to launch".to_string()
        }
    }

    fn action_color(&self) -> Color {
        if self.is_current || self.is_colonized {
            Colors::PRIMARY
        } else if self.can_launch {
            Colors::SUCCESS
        } else {
            Colors::ERROR
        }
    }
}

pub(super) fn draw_mobile_planet_info(
    view: &MapView,
    index: usize,
    can_launch: bool,
    x: f32,
    y: f32,
    width: f32,
    screen_h: f32,
) {
    let info = MobileWorldInfo::new(view, index, can_launch, x, y, width, screen_h);
    crate::ui::draw_panel(info.x, info.y, info.width, info.panel_h);
    if screen_h < SHORT_MOBILE_HEIGHT {
        compact::draw(&info);
    } else if screen_h < 560.0 {
        medium::draw(&info);
    } else {
        tall::draw(&info);
    }
}

pub(crate) fn compact_stockpile_summary(telemetry: &WorldTelemetry) -> String {
    format!(
        "Stock M{:.0}/{:.0} D{:.0} B{:.0} A{:.0} C{:.0}",
        telemetry.minerals,
        telemetry.mineral_capacity,
        telemetry.data,
        telemetry.biomass,
        telemetry.alloy,
        telemetry.components
    )
}

pub(crate) fn mobile_route_action(
    source_name: &str,
    target_name: Option<&str>,
    status: Option<&str>,
    is_current: bool,
    target_is_current: bool,
    is_colonized: bool,
) -> Option<String> {
    if !is_colonized {
        return None;
    }
    let target_needs_attention = matches!(
        status,
        Some("NO PAD" | "PAD OFFLINE" | "PAD TARGET UNAVAILABLE" | "HOLDING" | "OVERFLOW")
    );
    if target_needs_attention {
        if target_is_current {
            return Some("Current location - tap Back, then BUILD".to_string());
        }
        return target_name.map(|target| format!("Tap {target} row to travel"));
    }
    if is_current {
        Some("Current location".to_string())
    } else {
        Some(format!("Tap {source_name} row to travel"))
    }
}

pub(crate) fn mobile_info_height(screen_h: f32, has_telemetry: bool, has_shipping: bool) -> f32 {
    if screen_h < SHORT_MOBILE_HEIGHT {
        if has_telemetry {
            80.0
        } else {
            68.0
        }
    } else if screen_h < 560.0 {
        if has_telemetry {
            124.0
        } else {
            96.0
        }
    } else {
        250.0 + if has_shipping { 32.0 } else { 0.0 }
    }
}
