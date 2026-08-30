//! Solar system overview screen

mod mobile;
mod shipping_panel;
mod status;
mod world_dossier;

use crate::state::{ExportOrder, InterplanetaryMobilePanel, Shipment, WorldTelemetry};
use crate::ui::{color_from_rgba, draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, note_neighbour, touch_area, Pointer, ScrollArea};
use status::{launch_action, launch_prompt, mass_driver_status_label, seed_ship_status_label};
use world_dossier::{
    draw_world_telemetry, hazard_color, hazard_summary, world_alert_code, world_alert_color,
    world_rule_lines, world_telemetry_height,
};

/// Actions from the interplanetary view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterplanetaryAction {
    None,
    Close,
    SelectPlanet(usize),
    RequestLaunch(usize),
    LaunchSeedShip(usize),
    CycleExportCargo,
    CycleExportTarget,
    CycleExportPad,
    CycleExportSchedule,
    CycleExportPriority,
    ToggleExportSurplus,
    SelectOrderWorld(usize),
    CycleRemoteExportCargo,
    CycleRemoteExportTarget,
    CycleRemoteExportSchedule,
    CycleRemoteExportPriority,
    ToggleRemoteExportSurplus,
    ToggleMobileWorlds,
    ToggleMobileShipping,
}

const PLANET_INFO_WIDTH: f32 = 300.0;
const PLANET_INFO_MARGIN: f32 = 16.0;
const SOLAR_MAP_GAP: f32 = 12.0;
const MIN_SOLAR_MAP_SCALE: f32 = 0.22;

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
    if draw_button_sized(screen_w - 110.0, 18.0, 80.0, 34.0, "Back") {
        return InterplanetaryAction::Close;
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
    if screen_w < 600.0 {
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
    let mut hovered_planet = view.launch_confirmation_target.or(Some(current_planet));
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

    let (list_focus, list_action) =
        draw_planet_list(view, list_x, list_y, list_w, list_h, can_launch, pointer);
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
            let angle = (i as f32) * 1.2 + get_time() as f32 * 0.02 * (1.0 / (i as f32 + 1.0));
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
            if let Some(clicked) =
                draw_planet_info(view, index, can_launch, screen_w, header_height, activated)
            {
                if action == InterplanetaryAction::None {
                    action = clicked;
                }
            }
        } else if index < planets.len() {
            // On a compressed map the list still changes the dossier, but
            // only the list owns activation for that selected world.
            draw_planet_info(view, index, can_launch, screen_w, header_height, false);
        }
    }

    draw_legend(screen_w, screen_h);

    // Instructions
    draw_ui_text(
        "Tap a planet to travel or launch | Tap Back to return",
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

/// Every world and what is piling up on it. A world left behind keeps working,
/// so the map says so rather than making the player fly there to find out.
/// Rows are controls as well as labels: the list remains usable when an orbit
/// is crowded, off-screen, or too small to comfortably tap.
fn draw_planet_list(
    view: &MapView,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    can_launch: bool,
    pointer: Pointer,
) -> (Option<usize>, InterplanetaryAction) {
    draw_panel(x, y, w, h);
    draw_ui_text("Planets", x + 12.0, y + 28.0, 16.0, Colors::PRIMARY);
    draw_ui_text("alerts", x + w - 104.0, y + 28.0, 9.0, Colors::TEXT_DIM);
    draw_ui_text("MIN", x + w - 40.0, y + 28.0, 9.0, Colors::TEXT_DIM);

    let mut focused = None;
    let mut action = InterplanetaryAction::None;
    let row_height = 24.0;
    let mut row_y = y + 56.0;
    for (index, planet) in view.data.planets.iter().enumerate() {
        let is_current = index == view.current_planet;
        let is_colonized = view.colonized.get(index).copied().unwrap_or(false);
        let row = Rect::new(x + 8.0, row_y - 18.0, w - 16.0, row_height);
        note_neighbour(row);
        let hit_row = touch_area(row);
        let is_focused = pointer.hovering_over(hit_row)
            || pointer.pressing(hit_row)
            || pointer.released_on(hit_row);
        if is_focused {
            focused = Some(index);
            let fill = if pointer.pressing(hit_row) {
                Colors::PRIMARY_SOFT
            } else {
                Color::new(0.0, 0.7, 0.9, 0.12)
            };
            draw_rectangle(row.x, row.y, row.w, row.h, fill);
        }
        let label_color = if is_current {
            Colors::PRIMARY
        } else if is_colonized {
            Colors::SUCCESS
        } else {
            Colors::TEXT_DIM
        };
        draw_ui_text(&planet.name, x + 12.0, row_y, 13.0, label_color);
        let telemetry = view.telemetry.get(index).copied().flatten();
        let alert_code = world_alert_code(telemetry);
        draw_ui_text(
            &alert_code,
            x + w - 104.0,
            row_y,
            9.0,
            world_alert_color(telemetry),
        );
        if let Some(Some(minerals)) = view.stockpiles.get(index) {
            draw_ui_text(
                &format!("{:.0}", minerals),
                x + w - 50.0,
                row_y,
                12.0,
                Colors::ACCENT,
            );
        }
        if pointer.released_on(hit_row) {
            if is_colonized && !is_current {
                action = InterplanetaryAction::SelectPlanet(index);
            } else if !is_colonized && can_launch {
                action = launch_action(index, view.launch_confirmation_target == Some(index));
            }
        }
        row_y += row_height;
    }
    (focused, action)
}

/// What the hovered world is, and what clicking it would do.
fn draw_planet_info(
    view: &MapView,
    index: usize,
    can_launch: bool,
    screen_w: f32,
    header_height: f32,
    activated: bool,
) -> Option<InterplanetaryAction> {
    let planet = &view.data.planets[index];
    let is_colonized = view.colonized.get(index).copied().unwrap_or(false);
    let is_current = index == view.current_planet;

    let panel_w = PLANET_INFO_WIDTH;
    let panel_h = 300.0;
    let panel_x = screen_w - panel_w - PLANET_INFO_MARGIN;
    let panel_y = header_height + 12.0;
    draw_panel(panel_x, panel_y, panel_w, panel_h);

    draw_ui_text(
        &planet.name,
        panel_x + 15.0,
        panel_y + 30.0,
        20.0,
        color_from_rgba(&planet.color),
    );
    draw_ui_text(
        &planet.description,
        panel_x + 15.0,
        panel_y + 55.0,
        13.0,
        Colors::TEXT,
    );
    draw_ui_text(
        &planet.difficulty,
        panel_x + 15.0,
        panel_y + 75.0,
        12.0,
        Colors::TEXT_DIM,
    );

    let (status, status_color) = if is_colonized {
        ("Status: COLONIZED", Colors::SUCCESS)
    } else {
        ("Status: Unexplored", Colors::WARNING)
    };
    draw_ui_text(status, panel_x + 15.0, panel_y + 105.0, 13.0, status_color);

    draw_ui_text(
        &format!("Hazards: {}", hazard_summary(planet)),
        panel_x + 15.0,
        panel_y + 128.0,
        11.0,
        hazard_color(planet),
    );

    let telemetry = view.telemetry.get(index).copied().flatten();
    if let Some(telemetry) = telemetry {
        draw_world_telemetry(&telemetry, panel_x + 15.0, panel_y + 147.0);
    }

    let rules_y = panel_y
        + if telemetry.is_some() {
            147.0 + world_telemetry_height()
        } else {
            149.0
        };
    draw_ui_text(
        "World rules:",
        panel_x + 15.0,
        rules_y,
        11.0,
        Colors::TEXT_DIM,
    );
    let rules = world_rule_lines(view.data, index);
    let rules_color = if rules.len() == 1 && rules[0] == "No special build rules" {
        Colors::SUCCESS
    } else {
        Colors::WARNING
    };
    for (line_index, line) in rules.iter().enumerate() {
        draw_ui_text(
            line,
            panel_x + 15.0,
            rules_y + 16.0 + line_index as f32 * 16.0,
            10.0,
            rules_color,
        );
    }

    let action_y = panel_y + if telemetry.is_some() { 289.0 } else { 213.0 };
    if is_current {
        draw_ui_text(
            "(Current Location)",
            panel_x + 15.0,
            action_y,
            11.0,
            Colors::PRIMARY,
        );
    } else if is_colonized {
        draw_ui_text(
            "Tap to travel",
            panel_x + 15.0,
            action_y,
            12.0,
            Colors::PRIMARY,
        );
    } else if can_launch {
        draw_ui_text(
            launch_prompt(view.launch_confirmation_target == Some(index)),
            panel_x + 15.0,
            action_y,
            12.0,
            Colors::SUCCESS,
        );
    } else {
        let blocker = if !view.mass_driver_researched {
            "Requires: Mass Driver research"
        } else if view.current_drivers_online == 0 {
            "Requires: a powered Mass Driver"
        } else {
            "Requires: a finished Seed Ship"
        };
        draw_ui_text(blocker, panel_x + 15.0, action_y, 11.0, Colors::ERROR);
    }

    if !activated {
        return None;
    }
    if is_colonized && !is_current {
        Some(InterplanetaryAction::SelectPlanet(index))
    } else if can_launch && !is_colonized {
        Some(launch_action(
            index,
            view.launch_confirmation_target == Some(index),
        ))
    } else {
        None
    }
}

fn draw_legend(screen_w: f32, screen_h: f32) {
    draw_panel(screen_w - 190.0, screen_h - 110.0, 180.0, 100.0);
    draw_circle(screen_w - 175.0, screen_h - 85.0, 6.0, Colors::SUCCESS);
    draw_ui_text(
        "Colonized",
        screen_w - 160.0,
        screen_h - 80.0,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_circle(screen_w - 175.0, screen_h - 65.0, 6.0, Colors::WARNING);
    draw_ui_text(
        "Unexplored",
        screen_w - 160.0,
        screen_h - 60.0,
        12.0,
        Colors::TEXT_DIM,
    );
    draw_circle(screen_w - 175.0, screen_h - 45.0, 6.0, Colors::PRIMARY);
    draw_ui_text(
        "Current",
        screen_w - 160.0,
        screen_h - 40.0,
        12.0,
        Colors::TEXT_DIM,
    );
}

#[cfg(test)]
mod tests;
