//! Main grid gameplay screen

mod entity_render;
mod factory_deck;
mod factory_render;
mod flow_render;
mod format;
mod hud;
mod input;
mod metrics;
mod ore_render;
mod seed_ship_render;
mod selection_render;
mod terrain_render;
mod upkeep_render;

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::directives::Directive;
use crate::state::PlanetState;
use crate::ui::color_from_rgba;
use macroquad::prelude::*;
use macroquad_toolkit::math::{lerp, pulse01_at};

use metrics::{is_cursor_over_ui_with_help, screen_to_grid, HudMetrics};

/// Actions from the planetary view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanetaryAction {
    None,
    FastForwardToNextEvent,
    OpenResearch,
    OpenSeedShip,
    OpenInterplanetary,
    OpenRecords,
    OpenMenu,
    RetrySave,
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
    let layout = HudMetrics::for_screen_with_panel(
        theme,
        screen_w,
        screen_h,
        state.camera,
        state.focus_mode,
        state.desktop_panel,
    );
    let cursor_over_ui = is_cursor_over_ui_with_help(
        mouse_x,
        mouse_y,
        screen_w,
        screen_h,
        layout,
        state.mobile_panel,
        state.show_help,
    );
    let touch = state.touch_gesture.update();
    if touch.active && !state.touch_gesture_routed {
        state.touch_gesture_routed = true;
        state.touch_camera_active = !is_cursor_over_ui_with_help(
            touch.center.x,
            touch.center.y,
            screen_w,
            screen_h,
            layout,
            state.mobile_panel,
            state.show_help,
        );
        let starts_on_building = state.touch_camera_active
            && touches().len() == 1
            && screen_to_grid(touch.center.x, touch.center.y, layout)
                .and_then(|pos| state.grid.get(pos))
                .is_some_and(|tile| tile.building.is_some());
        state.touch_route_active = state.touch_camera_active
            && touches().len() == 1
            && !state.demolish_mode
            && !state.factory_deck_open
            && (state.selected_building == Some(crate::engine::BuildingType::Conduit)
                || starts_on_building);
    }
    let touch_grid_active = state.touch_camera_active;
    // A selected Conduit, or a one-finger touch that starts on a building,
    // turns a map drag into route planning. The latter is Smart Drag: the
    // existing building is the source and the destination receives the path.
    // The player can still pan from empty ground, or tap CANCEL before moving
    // the map by touch.
    let touch_route_active = state.touch_route_active;
    let touch_camera_active = touch_grid_active && !touch_route_active && !state.factory_deck_open;
    input::handle_camera(
        state,
        layout,
        cursor_over_ui,
        screen_w,
        screen_h,
        touch,
        touch_camera_active,
    );

    let metrics = HudMetrics::for_screen_with_panel(
        theme,
        screen_w,
        screen_h,
        state.camera,
        state.focus_mode,
        state.desktop_panel,
    );
    let hovered_pos = if cursor_over_ui || state.factory_deck_open {
        None
    } else {
        screen_to_grid(mouse_x, mouse_y, metrics)
            .filter(|pos| pos.in_bounds(state.grid.width, state.grid.height))
    };
    let touch_tap = touch
        .tap
        .filter(|_| touch_grid_active && !state.factory_deck_open)
        .and_then(|position| screen_to_grid(position.x, position.y, metrics))
        .filter(|pos| pos.in_bounds(state.grid.width, state.grid.height));
    let touch_drag_pos = touch
        .active
        .then_some(touch.center)
        .filter(|_| touch_route_active)
        .and_then(|position| screen_to_grid(position.x, position.y, metrics))
        .filter(|pos| pos.in_bounds(state.grid.width, state.grid.height));
    if !touch.active {
        state.touch_camera_active = false;
        state.touch_route_active = false;
        state.touch_gesture_routed = false;
    }

    terrain_render::draw_planetary_background(screen_w, screen_h, time, state);
    terrain_render::draw_collapse_shake(state, screen_w, screen_h, time);
    terrain_render::draw_grid_tiles(state, textures, metrics, hovered_pos, pulse, global_pulse);
    terrain_render::draw_core_stage_planetary_backdrop(state, screen_w, screen_h, time);
    terrain_render::draw_core_stage_ambient_for_map(state, metrics, time);
    selection_render::draw_box_selection(state, metrics, hovered_pos);
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
    factory_render::draw_processor_buffers(state, metrics, theme, time);
    flow_render::draw(state, metrics, theme, time);
    factory_deck::draw_factory_depth_art(state, metrics, theme, time);
    entity_render::draw_factory_warnings(state, metrics, theme, time);
    // Over the tiles and under the drones: the ship is the tallest thing on
    // the world, but the swarm still crawls in front of it.
    seed_ship_render::draw_seed_ship(state, metrics, time);
    entity_render::draw_drones(state, metrics, theme, time);
    entity_render::draw_particles(state, metrics);
    flow_render::draw_mobile_ledger(state, metrics, theme);

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

    if let Some(position) = factory_deck::draw_factory_depth_overlay(state, metrics, theme) {
        hud::center_camera_on_position(state, position, metrics, screen_w, screen_h);
    }

    upkeep_render::draw_notifications(state, metrics, screen_w);

    if ui_action != PlanetaryAction::None {
        ui_action
    } else {
        input::handle_input(state, hovered_pos, touch_tap, touch_drag_pos, touch)
    }
}
