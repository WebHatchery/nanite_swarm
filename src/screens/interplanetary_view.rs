//! Solar system overview screen

use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

/// Planet data for the solar system
#[derive(Debug, Clone)]
pub struct PlanetInfo {
    pub name: &'static str,
    pub orbit_radius: f32,
    pub color: Color,
    pub size: f32,
    pub description: &'static str,
    pub difficulty: &'static str,
}

impl PlanetInfo {
    const fn new(
        name: &'static str,
        orbit_radius: f32,
        color: Color,
        size: f32,
        description: &'static str,
        difficulty: &'static str,
    ) -> Self {
        Self {
            name,
            orbit_radius,
            color,
            size,
            description,
            difficulty,
        }
    }
}

/// Get the default planet data
pub fn get_planets() -> [PlanetInfo; 5] {
    [
        PlanetInfo::new(
            "Mercury",
            80.0,
            Color::new(0.6, 0.6, 0.6, 1.0),
            8.0,
            "Scorched world rich in metals",
            "Hard - Extreme temperatures",
        ),
        PlanetInfo::new(
            "Venus",
            120.0,
            Color::new(0.9, 0.7, 0.3, 1.0),
            11.0,
            "Thick atmosphere, volcanic",
            "Hard - Corrosive atmosphere",
        ),
        PlanetInfo::new(
            "Mars",
            170.0,
            Color::new(0.8, 0.4, 0.3, 1.0),
            10.0,
            "Red planet with polar ice caps",
            "Normal - Starting world",
        ),
        PlanetInfo::new(
            "Jupiter",
            240.0,
            Color::new(0.8, 0.6, 0.4, 1.0),
            18.0,
            "Gas giant with many moons",
            "Medium - Moon colonization",
        ),
        PlanetInfo::new(
            "Saturn",
            310.0,
            Color::new(0.9, 0.8, 0.5, 1.0),
            16.0,
            "Ringed giant, resource-rich moons",
            "Medium - Distant orbit",
        ),
    ]
}

/// Actions from the interplanetary view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterplanetaryAction {
    None,
    Close,
    SelectPlanet(usize),
    LaunchSeedShip(usize),
}

/// Render the interplanetary solar system view
pub fn render_interplanetary_view(
    current_planet: usize,
    has_mass_driver: bool,
    seed_ship_ready: bool,
    colonized_planets: &[bool],
) -> InterplanetaryAction {
    // A new world costs a whole Seed Ship, thrown by a Mass Driver.
    let can_launch = has_mass_driver && seed_ship_ready;
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let (mouse_x, mouse_y) = mouse_position();

    let header_height = 72.0;

    // Header
    draw_panel(0.0, 0.0, screen_w, header_height);
    draw_ui_text("Solar System", 18.0, 30.0, 18.0, Colors::PRIMARY);
    draw_ui_text("Planetary Map", 18.0, 52.0, 12.0, Colors::TEXT_DIM);
    if draw_button_sized(screen_w - 110.0, 18.0, 80.0, 34.0, "Back") {
        return InterplanetaryAction::Close;
    }

    let (driver_label, driver_color) = if has_mass_driver {
        ("Mass Driver: ONLINE", Colors::SUCCESS)
    } else {
        ("Mass Driver: OFFLINE", Colors::TEXT_DIM)
    };
    draw_ui_text(driver_label, screen_w - 320.0, 34.0, 12.0, driver_color);

    let (ship_label, ship_color) = if seed_ship_ready {
        ("Seed Ship: READY", Colors::SUCCESS)
    } else {
        ("Seed Ship: UNDER CONSTRUCTION", Colors::TEXT_DIM)
    };
    draw_ui_text(ship_label, screen_w - 320.0, 54.0, 12.0, ship_color);

    // Draw sun at center
    let center_x = screen_w / 2.0;
    let center_y = screen_h / 2.0 + 20.0;

    // Sun glow
    draw_circle(center_x, center_y, 50.0, Color::new(1.0, 0.8, 0.2, 0.2));
    draw_circle(center_x, center_y, 45.0, Color::new(1.0, 0.7, 0.1, 0.3));
    draw_circle(center_x, center_y, 40.0, Colors::WARNING);

    let planets = get_planets();
    let mut hovered_planet: Option<usize> = None;
    let mut action = InterplanetaryAction::None;

    // Left planet list
    let list_x = 16.0;
    let list_y = header_height + 12.0;
    let list_w = 220.0;
    let list_h = screen_h - list_y - 80.0;
    draw_panel(list_x, list_y, list_w, list_h);
    draw_ui_text(
        "Planets",
        list_x + 12.0,
        list_y + 28.0,
        16.0,
        Colors::PRIMARY,
    );
    let mut list_row_y = list_y + 56.0;
    for (index, planet) in planets.iter().enumerate() {
        let is_current = index == current_planet;
        let is_colonized = colonized_planets.get(index).copied().unwrap_or(false);
        let label_color = if is_current {
            Colors::PRIMARY
        } else if is_colonized {
            Colors::SUCCESS
        } else {
            Colors::TEXT_DIM
        };
        draw_ui_text(planet.name, list_x + 12.0, list_row_y, 13.0, label_color);
        list_row_y += 20.0;
    }

    for (i, planet) in planets.iter().enumerate() {
        // Draw orbit
        let orbit_color = if i == current_planet {
            Color::new(0.0, 0.85, 1.0, 0.3)
        } else {
            Colors::SECONDARY
        };
        draw_circle_lines(center_x, center_y, planet.orbit_radius, 1.0, orbit_color);

        // Calculate planet position
        let angle = (i as f32) * 1.2 + get_time() as f32 * 0.02 * (1.0 / (i as f32 + 1.0));
        let px = center_x + planet.orbit_radius * angle.cos();
        let py = center_y + planet.orbit_radius * angle.sin();

        // Check hover
        let dist = ((mouse_x - px).powi(2) + (mouse_y - py).powi(2)).sqrt();
        let is_hovered = dist < planet.size + 5.0;
        if is_hovered {
            hovered_planet = Some(i);
        }

        let planet_size = if i == current_planet {
            planet.size * 1.3
        } else {
            planet.size
        };
        let is_colonized = colonized_planets.get(i).copied().unwrap_or(false);

        // Draw colonization glow
        if is_colonized {
            draw_circle(px, py, planet_size + 6.0, Color::new(0.0, 1.0, 0.5, 0.2));
        }

        // Draw planet
        draw_circle(px, py, planet_size, planet.color);

        // Hover highlight
        if is_hovered {
            draw_circle_lines(px, py, planet_size + 4.0, 2.0, Colors::PRIMARY);
        }

        // Current planet indicator
        if i == current_planet {
            draw_circle_lines(px, py, planet_size + 8.0, 2.0, Colors::SUCCESS);
        }

        // Draw name
        let name_color = if is_colonized {
            Colors::SUCCESS
        } else {
            Colors::TEXT_DIM
        };
        draw_ui_text(
            planet.name,
            px - 20.0,
            py + planet_size + 15.0,
            12.0,
            name_color,
        );
    }

    // Info panel for hovered planet
    if let Some(index) = hovered_planet {
        let planet = &planets[index];
        let is_colonized = colonized_planets.get(index).copied().unwrap_or(false);
        let is_current = index == current_planet;

        let panel_w = 300.0;
        let panel_h = 200.0;
        let panel_x = screen_w - panel_w - 16.0;
        let panel_y = header_height + 12.0;
        draw_panel(panel_x, panel_y, panel_w, panel_h);

        draw_ui_text(
            planet.name,
            panel_x + 15.0,
            panel_y + 30.0,
            20.0,
            planet.color,
        );
        draw_ui_text(
            planet.description,
            panel_x + 15.0,
            panel_y + 55.0,
            13.0,
            Colors::TEXT,
        );
        draw_ui_text(
            planet.difficulty,
            panel_x + 15.0,
            panel_y + 75.0,
            12.0,
            Colors::TEXT_DIM,
        );

        if is_colonized {
            draw_ui_text(
                "Status: COLONIZED",
                panel_x + 15.0,
                panel_y + 105.0,
                13.0,
                Colors::SUCCESS,
            );
        } else {
            draw_ui_text(
                "Status: Unexplored",
                panel_x + 15.0,
                panel_y + 105.0,
                13.0,
                Colors::WARNING,
            );
        }

        if is_current {
            draw_ui_text(
                "(Current Location)",
                panel_x + 15.0,
                panel_y + 125.0,
                11.0,
                Colors::PRIMARY,
            );
        } else if is_colonized {
            draw_ui_text(
                "Click to travel",
                panel_x + 15.0,
                panel_y + 150.0,
                12.0,
                Colors::PRIMARY,
            );
        } else if can_launch {
            draw_ui_text(
                "Click to launch the Seed Ship",
                panel_x + 15.0,
                panel_y + 150.0,
                12.0,
                Colors::SUCCESS,
            );
        } else {
            let blocker = if !has_mass_driver {
                "Requires: Mass Driver"
            } else {
                "Requires: a finished Seed Ship"
            };
            draw_ui_text(
                blocker,
                panel_x + 15.0,
                panel_y + 150.0,
                11.0,
                Colors::ERROR,
            );
        }

        // Handle click
        if is_mouse_button_pressed(MouseButton::Left) {
            if is_colonized && !is_current {
                action = InterplanetaryAction::SelectPlanet(index);
            } else if can_launch && !is_colonized {
                action = InterplanetaryAction::LaunchSeedShip(index);
            }
        }
    }

    // Legend
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

    // Instructions
    draw_ui_text(
        "Press ESC to return | M to toggle map",
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
