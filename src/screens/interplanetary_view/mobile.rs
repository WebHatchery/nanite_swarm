//! Touch-first solar-system sections for portrait screens.

use super::planet_panel::draw_planet_list;
use super::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width, Pointer};

mod world_info;

#[cfg(test)]
pub(super) use world_info::{compact_stockpile_summary, mobile_info_height, mobile_route_action};

const MOBILE_EDGE: f32 = 12.0;
const MOBILE_TAB_Y: f32 = 112.0;
const MOBILE_CONTENT_Y: f32 = 156.0;
const SHORT_MOBILE_HEIGHT: f32 = 440.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct MobileWorldsLayout {
    list_h: f32,
    row_height: f32,
    info_gap: f32,
    compact: bool,
}

fn mobile_status_text(is_colonized: bool, alert_code: Option<&str>) -> String {
    let status = if is_colonized {
        "Status: COLONIZED"
    } else {
        "Status: UNEXPLORED"
    };
    alert_code
        .map(|code| format!("{status} / {code}"))
        .unwrap_or_else(|| status.to_string())
}

pub(super) fn draw(
    view: &MapView,
    shipping_scroll: &mut ScrollArea,
    panel: InterplanetaryMobilePanel,
    screen_h: f32,
    can_launch: bool,
) -> InterplanetaryAction {
    let screen_w = screen_width();
    let pointer = Pointer::read(|position| position);
    let action = draw_tabs(panel, screen_w);
    if action != InterplanetaryAction::None {
        if action == InterplanetaryAction::ToggleMobileWorlds {
            shipping_scroll.set_offset(0.0);
        }
        return action;
    }

    let action = match panel {
        InterplanetaryMobilePanel::Worlds => {
            draw_worlds(view, can_launch, pointer, screen_w, screen_h)
        }
        InterplanetaryMobilePanel::Shipping => shipping_panel::draw(
            view,
            MOBILE_EDGE,
            MOBILE_CONTENT_Y,
            (screen_w - MOBILE_EDGE * 2.0).max(80.0),
            (screen_h - MOBILE_CONTENT_Y - 32.0).max(120.0),
            shipping_scroll,
        ),
    };

    let footer_hint = truncate_text_to_width(
        mobile_footer_hint_text(panel, view.launch_confirmation_target.is_some(), screen_w),
        (screen_w - MOBILE_EDGE * 2.0).max(40.0),
        10.0,
    );
    draw_ui_text(
        &footer_hint,
        MOBILE_EDGE,
        screen_h - 18.0,
        10.0,
        Colors::TEXT_DIM,
    );
    if is_key_pressed(KeyCode::Escape) {
        return InterplanetaryAction::Close;
    }
    action
}

fn mobile_footer_hint_text(
    panel: InterplanetaryMobilePanel,
    confirming: bool,
    screen_w: f32,
) -> &'static str {
    if screen_w < 360.0 {
        match (panel, confirming) {
            (InterplanetaryMobilePanel::Worlds, true) => "Tap CONFIRM | SHIPPING for routes",
            (InterplanetaryMobilePanel::Worlds, false) => "Tap world | SHIPPING for routes",
            (InterplanetaryMobilePanel::Shipping, _) => "Drag to scroll | WORLDS to return",
        }
    } else {
        match (panel, confirming) {
            (InterplanetaryMobilePanel::Worlds, true) => {
                "Tap CONFIRM again to launch | Tap Shipping for routes"
            }
            (InterplanetaryMobilePanel::Worlds, false) => {
                "Tap a world to travel | Tap Shipping for routes"
            }
            (InterplanetaryMobilePanel::Shipping, _) => {
                "Drag panel to scroll | Tap controls | Tap Worlds to return"
            }
        }
    }
}

fn draw_tabs(panel: InterplanetaryMobilePanel, screen_w: f32) -> InterplanetaryAction {
    let tab_w = ((screen_w - MOBILE_EDGE * 3.0) * 0.5).max(60.0);
    let shipping_x = MOBILE_EDGE * 2.0 + tab_w;
    let worlds = draw_button_sized(MOBILE_EDGE, MOBILE_TAB_Y, tab_w, 32.0, "");
    let shipping = draw_button_sized(shipping_x, MOBILE_TAB_Y, tab_w, 32.0, "");
    draw_tab_label(
        "WORLDS",
        MOBILE_EDGE,
        tab_w,
        panel == InterplanetaryMobilePanel::Worlds,
    );
    draw_tab_label(
        "SHIPPING",
        shipping_x,
        tab_w,
        panel == InterplanetaryMobilePanel::Shipping,
    );
    if worlds && panel != InterplanetaryMobilePanel::Worlds {
        return InterplanetaryAction::ToggleMobileWorlds;
    }
    if shipping && panel != InterplanetaryMobilePanel::Shipping {
        return InterplanetaryAction::ToggleMobileShipping;
    }
    InterplanetaryAction::None
}

fn draw_tab_label(label: &str, x: f32, width: f32, selected: bool) {
    let text_width = measure_ui_text(label, None, 11, 1.0).width;
    draw_ui_text(
        label,
        x + (width - text_width) * 0.5,
        MOBILE_TAB_Y + 21.0,
        11.0,
        if selected {
            Colors::PRIMARY
        } else {
            Colors::TEXT
        },
    );
}

fn draw_worlds(
    view: &MapView,
    can_launch: bool,
    pointer: Pointer,
    screen_w: f32,
    screen_h: f32,
) -> InterplanetaryAction {
    let panel_w = (screen_w - MOBILE_EDGE * 2.0).max(80.0);
    let layout = mobile_worlds_layout(screen_h, view.data.planets.len());
    let (focused, list_action) = draw_planet_list(
        view,
        MOBILE_EDGE,
        MOBILE_CONTENT_Y,
        panel_w,
        layout.list_h,
        can_launch,
        pointer,
        layout.compact,
    );
    let index = mobile_world_focus(
        focused,
        view.launch_confirmation_target,
        view.editing_world,
        view.current_planet,
        view.data.planets.len(),
    );
    let info_y = MOBILE_CONTENT_Y + layout.list_h + layout.info_gap;
    world_info::draw_mobile_planet_info(
        view,
        index,
        can_launch,
        MOBILE_EDGE,
        info_y,
        panel_w,
        screen_h,
    );
    list_action
}

fn mobile_world_focus(
    focused: Option<usize>,
    launch_confirmation_target: Option<usize>,
    editing_world: usize,
    current_planet: usize,
    planet_count: usize,
) -> usize {
    let fallback = if current_planet < planet_count {
        current_planet
    } else {
        0
    };
    focused
        .or(launch_confirmation_target)
        .or((editing_world < planet_count).then_some(editing_world))
        .unwrap_or(fallback)
}

fn mobile_worlds_layout(screen_h: f32, planet_count: usize) -> MobileWorldsLayout {
    let compact = screen_h < SHORT_MOBILE_HEIGHT;
    let row_height = if compact { 19.0 } else { 24.0 };
    let list_header_height = if compact { 34.0 } else { 44.0 };
    MobileWorldsLayout {
        list_h: list_header_height + planet_count as f32 * row_height,
        row_height,
        info_gap: if compact { 6.0 } else { 10.0 },
        compact,
    }
}

#[cfg(test)]
mod tests;
