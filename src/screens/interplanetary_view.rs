//! Solar system overview screen

mod core_marker;
mod mobile;
mod planet_panel;
mod shipping_panel;
mod status;
mod world_dossier;

use crate::state::{ExportOrder, InterplanetaryMobilePanel, Shipment, WorldTelemetry};
use crate::ui::{color_from_rgba, draw_button_sized, draw_panel, Colors, Dimensions};
use core_marker::draw_core_stage_marker;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, Pointer, ScrollArea};
use status::{mass_driver_status_label, seed_ship_status_label};

/// Actions from the interplanetary view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterplanetaryAction {
    None,
    Close,
    CancelLaunch,
    SelectPlanet(usize),
    RequestLaunch(usize),
    LaunchSeedShip(usize),
    CycleExportCargo,
    CycleExportTarget,
    CycleExportPad,
    CycleExportSchedule,
    CycleExportPriority,
    CycleExportReserve,
    SelectOrderWorld(usize),
    CycleRemoteExportCargo,
    CycleRemoteExportTarget,
    CycleRemoteExportSchedule,
    CycleRemoteExportPriority,
    CycleRemoteExportReserve,
    ToggleMobileWorlds,
    ToggleMobileShipping,
}

const PLANET_INFO_WIDTH: f32 = 300.0;
const PLANET_INFO_MARGIN: f32 = 16.0;
const SOLAR_MAP_GAP: f32 = 12.0;
const MIN_SOLAR_MAP_SCALE: f32 = 0.22;
const COMPACT_LAYOUT_WIDTH: f32 = 960.0;
const COMPACT_LAYOUT_HEIGHT: f32 = 560.0;

fn compact_interplanetary_layout(screen_w: f32, screen_h: f32) -> bool {
    screen_w < COMPACT_LAYOUT_WIDTH || screen_h < COMPACT_LAYOUT_HEIGHT
}

/// Keep the orbit diagram inside the space left between the touch panels.
/// Wide screens retain the authored orbit scale; narrow windows scale the
/// rings and move their center so the list remains a fallback for every tap.
fn solar_map_layout(
    screen_w: f32,
    screen_h: f32,
    header_height: f32,
    list_x: f32,
    list_w: f32,
    max_orbit: f32,
) -> (f32, f32, f32) {
    let info_x = screen_w - PLANET_INFO_WIDTH - PLANET_INFO_MARGIN;
    let map_left = list_x + list_w + SOLAR_MAP_GAP;
    let map_right = info_x - SOLAR_MAP_GAP;
    let map_top = header_height + SOLAR_MAP_GAP;
    let map_bottom = (screen_h - 30.0).max(map_top + 1.0);
    let horizontal_radius = ((map_right - map_left) * 0.5 - 16.0).max(20.0);
    let vertical_radius = ((map_bottom - map_top) * 0.5 - 12.0).max(20.0);
    let radius = horizontal_radius.min(vertical_radius);
    let orbit_scale = (radius / max_orbit.max(1.0)).clamp(0.05, 1.0);
    (
        (map_left + map_right) * 0.5,
        (map_top + map_bottom) * 0.5,
        orbit_scale,
    )
}

fn initial_dossier_focus(
    launch_confirmation_target: Option<usize>,
    editing_world: usize,
    current_planet: usize,
    planet_count: usize,
) -> Option<usize> {
    launch_confirmation_target
        .filter(|index| *index < planet_count)
        .or_else(|| (editing_world < planet_count).then_some(editing_world))
        .or_else(|| (current_planet < planet_count).then_some(current_planet))
}

/// Everything the map reads out of the campaign. One struct rather than a
/// growing row of positional booleans, which is how a caller ends up passing
/// "ready" where "colonized" was wanted.
pub struct MapView<'a> {
    pub data: &'a crate::data::GameData,
    pub current_planet: usize,
    /// Whether the swarm has researched Mass Drivers, for remote order setup.
    pub mass_driver_researched: bool,
    pub seed_ship_ready: bool,
    pub colonized: &'a [bool],
    pub stockpiles: &'a [Option<f32>],
    /// Live snapshots for the worlds the swarm has already reached.
    pub telemetry: &'a [Option<WorldTelemetry>],
    /// Powered Mass Drivers standing on the world underfoot.
    pub current_drivers_online: usize,
    /// Mass Drivers standing on the world underfoot, including offline ones.
    pub current_drivers_built: usize,
    /// Untouched world awaiting the second tap required to launch.
    pub launch_confirmation_target: Option<usize>,
    /// Powered Mass Drivers on every colonized world, for remote order edits.
    pub drivers_online: &'a [usize],
    /// Mass Drivers standing on every colonized world, including offline ones.
    pub drivers_built: &'a [usize],
    /// Powered Landing Pads on each world, so the panel can say when the
    /// target has nothing to catch a pod with.
    pub pads: &'a [usize],
    /// Landing Pads standing on each world, including offline pads.
    pub pads_built: &'a [usize],
    /// Powered Landing Pad positions, so a pinned route can detect a pad that
    /// was demolished or lost power even when another pad is still online.
    pub pad_positions: &'a [Vec<crate::engine::GridPos>],
    /// The fullest live outbound pod on each world, for remote order edits.
    pub pod_fractions: &'a [f32],
    /// Nominal throughput of each world's standing order, in units per minute.
    pub export_rates: &'a [f32],
    pub shipments: &'a [Shipment],
    pub orders: &'a [Option<ExportOrder>],
    pub pending_pods: &'a [usize],
    pub pod_caps: &'a [usize],
    pub overflow_pods: &'a [usize],
    pub editing_world: usize,
}

/// Render the interplanetary solar system view
pub fn render_interplanetary_view(
    view: &MapView,
    shipping_scroll: &mut ScrollArea,
    mobile_panel: InterplanetaryMobilePanel,
) -> InterplanetaryAction {
    // A new world costs a whole Seed Ship, thrown by a Mass Driver.
    let can_launch = view.current_drivers_online > 0 && view.seed_ship_ready;
    let current_planet = view.current_planet;
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let pointer = Pointer::read(|position| position);

    let header_height = 72.0;

    // Header
    draw_panel(0.0, 0.0, screen_w, header_height);
    draw_ui_text("Solar System", 18.0, 30.0, 18.0, Colors::PRIMARY);
    draw_ui_text("Planetary Map", 18.0, 52.0, 12.0, Colors::TEXT_DIM);
    let back_label = if view.launch_confirmation_target.is_some() {
        "Cancel"
    } else {
        "Back"
    };
    if draw_button_sized(screen_w - 110.0, 18.0, 80.0, 34.0, back_label) {
        return if view.launch_confirmation_target.is_some() {
            InterplanetaryAction::CancelLaunch
        } else {
            InterplanetaryAction::Close
        };
    }

    let driver_label = mass_driver_status_label(
        view.mass_driver_researched,
        view.current_drivers_built,
        view.current_drivers_online,
    );
    let driver_color = if view.current_drivers_online > 0 {
        Colors::SUCCESS
    } else if view.mass_driver_researched {
        Colors::WARNING
    } else {
        Colors::TEXT_DIM
    };
    let ship_label = seed_ship_status_label(
        view.seed_ship_ready,
        view.current_drivers_built,
        view.current_drivers_online,
    );
    let ship_color = if view.seed_ship_ready && view.current_drivers_online > 0 {
        Colors::SUCCESS
    } else if view.seed_ship_ready {
        Colors::WARNING
    } else {
        Colors::TEXT_DIM
    };
    if compact_interplanetary_layout(screen_w, screen_h) {
        draw_ui_text(driver_label, 18.0, header_height + 14.0, 10.0, driver_color);
        draw_ui_text(ship_label, 18.0, header_height + 30.0, 10.0, ship_color);
        return mobile::draw(view, shipping_scroll, mobile_panel, screen_h, can_launch);
    }

    draw_ui_text(driver_label, screen_w - 320.0, 34.0, 12.0, driver_color);

    draw_ui_text(ship_label, screen_w - 320.0, 54.0, 12.0, ship_color);

    let planets = &view.data.planets;
    // Keep a dossier visible when the map opens. This gives touch players a
    // useful first target and means the planet list can be used for travel
    // without making the orbit nodes the only way to learn about a world.
    let mut hovered_planet = initial_dossier_focus(
        view.launch_confirmation_target,
        view.editing_world,
        current_planet,
        planets.len(),
    );
    let mut action = InterplanetaryAction::None;

    let list_x = 16.0;
    let list_y = header_height + 12.0;
    let list_w = 220.0;
    let list_h = 44.0 + planets.len() as f32 * 24.0;
    let max_orbit = planets
        .iter()
        .map(|planet| planet.orbit_radius)
        .fold(1.0, f32::max);
    let (center_x, center_y, orbit_scale) =
        solar_map_layout(screen_w, screen_h, header_height, list_x, list_w, max_orbit);
    let map_compressed = orbit_scale < MIN_SOLAR_MAP_SCALE;
    let animation_time = get_time();

    if map_compressed {
        draw_ui_text(
            "MAP",
            center_x - 12.0,
            center_y + 4.0,
            9.0,
            Colors::TEXT_DIM,
        );
    } else {
        // Scale the sun with the rings too: a fixed 40px sun swallows the
        // inner orbits on a narrow window even after the outer rings fit.
        let sun_radius = 40.0 * orbit_scale;
        draw_circle(
            center_x,
            center_y,
            sun_radius * 1.25,
            Color::new(1.0, 0.8, 0.2, 0.2),
        );
        draw_circle(
            center_x,
            center_y,
            sun_radius * 1.125,
            Color::new(1.0, 0.7, 0.1, 0.3),
        );
        draw_circle(center_x, center_y, sun_radius, Colors::WARNING);
    }

    let (list_focus, list_action) = planet_panel::draw_planet_list(
        view, list_x, list_y, list_w, list_h, can_launch, pointer, false,
    );
    if list_focus.is_some() {
        hovered_planet = list_focus;
    }
    if list_action != InterplanetaryAction::None {
        action = list_action;
    }

    // Under the map's own furniture: the shipping panel takes what the planet
    // list does not need.
    let shipping_y = list_y + list_h + 12.0;
    let shipping_h = (screen_h - shipping_y - 80.0).max(120.0);
    let shipping_action = shipping_panel::draw(
        view,
        list_x,
        shipping_y,
        list_w,
        shipping_h,
        shipping_scroll,
    );
    if shipping_action != InterplanetaryAction::None {
        action = shipping_action;
    }

    // Where each world ended up this frame, so a pod can be drawn between two
    // of them rather than between two orbits.
    let mut positions = Vec::with_capacity(planets.len());
    // There is not enough horizontal room to make an orbit diagram legible in
    // compressed mode; the planet list and dossier remain fully functional.
    if !map_compressed {
        for (i, planet) in planets.iter().enumerate() {
            // Draw orbit
            let orbit_color = if i == current_planet {
                Color::new(0.0, 0.85, 1.0, 0.3)
            } else {
                Colors::SECONDARY
            };
            draw_circle_lines(
                center_x,
                center_y,
                planet.orbit_radius * orbit_scale,
                1.0,
                orbit_color,
            );

            // Calculate planet position
            let angle = (i as f32) * 1.2 + animation_time as f32 * 0.02 * (1.0 / (i as f32 + 1.0));
            let px = center_x + planet.orbit_radius * orbit_scale * angle.cos();
            let py = center_y + planet.orbit_radius * orbit_scale * angle.sin();
            positions.push((px, py));

            let rendered_size = (planet.size * orbit_scale).max(4.0);
            let target_radius = (rendered_size + 5.0).max(22.0);
            let target = Rect::new(
                px - target_radius,
                py - target_radius,
                target_radius * 2.0,
                target_radius * 2.0,
            );
            let is_hovered = pointer.hovering_over(target);
            let focused = is_hovered || pointer.pressing(target) || pointer.released_on(target);
            if focused {
                hovered_planet = Some(i);
            }

            let planet_size = if i == current_planet {
                rendered_size * 1.3
            } else {
                rendered_size
            };
            let is_colonized = view.colonized.get(i).copied().unwrap_or(false);

            // Draw colonization glow
            if is_colonized {
                draw_circle(
                    px,
                    py,
                    planet_size + (6.0 * orbit_scale).max(3.0),
                    Color::new(0.0, 1.0, 0.5, 0.2),
                );
            }

            // Draw planet
            draw_circle(px, py, planet_size, color_from_rgba(&planet.color));
            draw_core_stage_marker(
                view.telemetry.get(i).copied().flatten(),
                vec2(px, py),
                planet_size,
                orbit_scale,
                animation_time,
            );

            // Hover highlight
            if is_hovered {
                draw_circle_lines(
                    px,
                    py,
                    planet_size + (4.0 * orbit_scale).max(3.0),
                    2.0,
                    Colors::PRIMARY,
                );
            }

            // Current planet indicator
            if i == current_planet {
                draw_circle_lines(
                    px,
                    py,
                    planet_size + (8.0 * orbit_scale).max(4.0),
                    2.0,
                    Colors::SUCCESS,
                );
            }

            // Draw name
            let name_color = if is_colonized {
                Colors::SUCCESS
            } else {
                Colors::TEXT_DIM
            };
            draw_ui_text(
                &planet.name,
                px - 20.0,
                py + planet_size + (15.0 * orbit_scale.max(0.75)),
                12.0,
                name_color,
            );
        }
    }

    let route_focus = if map_compressed {
        None
    } else {
        shipping_panel::route_hit_test(
            (pointer.position.x, pointer.position.y),
            view.orders,
            &positions,
            12.0,
        )
    };
    let pointer_on_planet = positions.iter().enumerate().any(|(index, position)| {
        let radius = ((planets[index].size * orbit_scale).max(4.0) + 5.0).max(22.0);
        let dx = pointer.position.x - position.0;
        let dy = pointer.position.y - position.1;
        dx * dx + dy * dy <= radius * radius
    });
    if list_focus.is_none() && !pointer_on_planet {
        if let Some(source) = route_focus {
            hovered_planet = Some(source);
        }
    }
    if pointer.released && action == InterplanetaryAction::None && !pointer_on_planet {
        if let Some(source) = route_focus {
            action = InterplanetaryAction::SelectOrderWorld(source);
        }
    }

    shipping_panel::draw_routes(view.orders, &positions);
    shipping_panel::draw_pods(view.shipments, &positions);

    // Info panel for hovered planet
    if let Some(index) = hovered_planet {
        if let Some(target) = positions.get(index).copied() {
            let radius = ((planets[index].size * orbit_scale).max(4.0) + 5.0).max(22.0);
            let activated = pointer.released_on(Rect::new(
                target.0 - radius,
                target.1 - radius,
                radius * 2.0,
                radius * 2.0,
            ));
            if let Some(clicked) = planet_panel::draw_planet_info(
                view,
                index,
                can_launch,
                screen_w,
                header_height,
                activated,
            ) {
                if action == InterplanetaryAction::None {
                    action = clicked;
                }
            }
        } else if index < planets.len() {
            // On a compressed map the list still changes the dossier, but
            // only the list owns activation for that selected world.
            planet_panel::draw_planet_info(view, index, can_launch, screen_w, header_height, false);
        }
    }

    planet_panel::draw_legend(screen_w, screen_h);

    // Instructions
    let footer = if view.launch_confirmation_target.is_some() {
        "Tap CONFIRM row to launch | Tap Cancel to clear | Tap a freight lane to inspect"
    } else {
        "Tap a planet to travel or launch | Tap a freight lane to inspect | Tap Back"
    };
    draw_ui_text(
        footer,
        20.0,
        screen_h - 20.0,
        Dimensions::FONT_SIZE_SMALL,
        Colors::TEXT_DIM,
    );

    if is_key_pressed(KeyCode::Escape) {
        return InterplanetaryAction::Close;
    }

    action
}

#[cfg(test)]
mod tests;
