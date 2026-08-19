//! Main grid gameplay screen

mod entity_render;
mod format;
mod hud;
mod input;
mod metrics;
mod ore_render;
mod seed_ship_render;
mod terrain_render;
mod upkeep_render;

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::directives::Directive;
use crate::state::PlanetState;
use crate::ui::color_from_rgba;
use macroquad::prelude::*;
use macroquad_toolkit::math::{lerp, pulse01_at};

use metrics::{is_cursor_over_ui, screen_to_grid, HudMetrics};

/// Actions from the planetary view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanetaryAction {
    None,
    OpenResearch,
    OpenSeedShip,
    OpenInterplanetary,
    OpenRecords,
    OpenMenu,
}

/// Render the planetary grid view
pub fn render_planetary_view(
    state: &mut PlanetState,
    textures: &GameTextures,
    directive: &Directive,
    theme: &UiTheme,
) -> PlanetaryAction {
    clear_background(color_from_rgba(&theme.colors.background));

    let screen_w = screen_width();
    let screen_h = screen_height();
    let motion = if macroquad_toolkit::settings::reduced_motion_enabled() {
        0.0
    } else {
        1.0
    };
    let pulse = pulse01_at(state.time_played, 2.5) * motion;
    let global_pulse = lerp(0.8, 1.0, pulse01_at(state.time_played, 2.0) * motion);
    let time = state.time_played as f32;

    // Move the camera first, then lay out the frame around where it ended up,
    // so drawing and hit-testing agree about which tile is under the cursor.
    let (mouse_x, mouse_y) = mouse_position();
    let layout = HudMetrics::for_screen(theme, screen_w, screen_h, state.camera, state.focus_mode);
    let cursor_over_ui = is_cursor_over_ui(mouse_x, mouse_y, screen_w, screen_h, layout);
    let touch = state.touch_gesture.update();
    if touch.active && !state.touch_gesture_routed {
        state.touch_gesture_routed = true;
        state.touch_camera_active =
            !is_cursor_over_ui(touch.center.x, touch.center.y, screen_w, screen_h, layout);
    }
    let touch_camera_active = state.touch_camera_active;
    input::handle_camera(
        state,
        layout,
        cursor_over_ui,
        screen_w,
        screen_h,
        touch,
        touch_camera_active,
    );

    let metrics = HudMetrics::for_screen(theme, screen_w, screen_h, state.camera, state.focus_mode);
    let hovered_pos = if cursor_over_ui {
        None
    } else {
        screen_to_grid(mouse_x, mouse_y, metrics)
            .filter(|pos| pos.in_bounds(state.grid.width, state.grid.height))
    };
    let touch_tap = touch
        .tap
        .filter(|_| touch_camera_active)
        .and_then(|position| screen_to_grid(position.x, position.y, metrics))
        .filter(|pos| pos.in_bounds(state.grid.width, state.grid.height));
    if !touch.active {
        state.touch_camera_active = false;
        state.touch_gesture_routed = false;
    }

    terrain_render::draw_planetary_background(screen_w, screen_h, time, state);
    terrain_render::draw_collapse_shake(state, screen_w, screen_h, time);
    terrain_render::draw_grid_tiles(state, textures, metrics, hovered_pos, pulse, global_pulse);
    terrain_render::draw_planet_features(state, metrics);
    terrain_render::draw_tutorial_route_hint(state, metrics, time);
    upkeep_render::draw_hazard_fields(state, metrics);
    // Under the wear tint and the drones: it is a property of the ground.
    ore_render::draw_ore(state, metrics);
    upkeep_render::draw_wear(state, metrics, time);
    upkeep_render::draw_coverage(state, metrics, hovered_pos);
    upkeep_render::draw_uncovered_hazards(state, metrics);
    upkeep_render::draw_severed_network(state, metrics, time);
    entity_render::draw_congestion(state, metrics, time);
    entity_render::draw_factory_warnings(state, metrics, theme, time);
    // Over the tiles and under the drones: the ship is the tallest thing on
    // the world, but the swarm still crawls in front of it.
    seed_ship_render::draw_seed_ship(state, metrics, time);
    entity_render::draw_drones(state, metrics, theme, time);
    entity_render::draw_particles(state, metrics);

    let ui_action = hud::draw_ui_panels(
        state,
        screen_w,
        screen_h,
        hovered_pos,
        directive,
        textures,
        theme,
        metrics,
    );

    upkeep_render::draw_notifications(state, metrics, screen_w);

    if ui_action != PlanetaryAction::None {
        ui_action
    } else {
        input::handle_input(state, hovered_pos, touch_tap)
    }
}
