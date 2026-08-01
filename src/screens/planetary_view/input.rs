//! Keyboard and mouse input handling for the planetary view

use crate::data;
use crate::engine::{BuildingType, GridPos};
use crate::state::PlanetState;
use macroquad::prelude::*;

use super::format::keycode_from_hotkey;
use super::metrics::HudMetrics;
use super::PlanetaryAction;

/// Drag the map with the middle mouse button, zoom with the wheel.
///
/// Runs before anything else reads the grid, so drawing and hit-testing both
/// see where the player just moved the view to.
pub(super) fn handle_camera(
    state: &mut PlanetState,
    metrics: HudMetrics,
    cursor_over_ui: bool,
    screen_w: f32,
    screen_h: f32,
) {
    let (mouse_x, mouse_y) = mouse_position();

    if is_mouse_button_down(MouseButton::Middle) {
        if let Some((last_x, last_y)) = state.camera_drag_anchor {
            state.camera.pan_by(mouse_x - last_x, mouse_y - last_y);
        }
        state.camera_drag_anchor = Some((mouse_x, mouse_y));
    } else {
        state.camera_drag_anchor = None;
    }

    let (_, wheel_y) = mouse_wheel();
    if wheel_y != 0.0 && !cursor_over_ui {
        // Wheel deltas are platform-flavoured; only the direction is reliable.
        let notches = wheel_y.signum();
        let cursor = (
            mouse_x - metrics.base_offset_x(),
            mouse_y - metrics.base_offset_y(),
        );
        state.camera.zoom_by(notches, cursor);
    }

    let grid_size = (
        state.grid.width as f32 * metrics.tile_size,
        state.grid.height as f32 * metrics.tile_size,
    );
    state
        .camera
        .clamp_to_viewport(metrics.viewport(screen_w, screen_h), grid_size);
}

/// Handle keyboard and mouse input
pub(super) fn handle_input(
    state: &mut PlanetState,
    hovered_pos: Option<GridPos>,
) -> PlanetaryAction {
    // Building hotkeys
    for def in &data::game_data().buildings {
        let Some(hotkey) = def.hotkey.as_ref().and_then(|key| key.chars().next()) else {
            continue;
        };
        let Some(keycode) = keycode_from_hotkey(hotkey) else {
            continue;
        };
        if !is_key_pressed(keycode) {
            continue;
        }
        let Some(building_type) = BuildingType::from_id(&def.id) else {
            continue;
        };
        if state.is_building_unlocked(building_type) {
            state.select_building(building_type);
        }
    }

    if is_key_pressed(KeyCode::F1) {
        state.show_help = !state.show_help;
    }
    if is_key_pressed(KeyCode::T) {
        state.tutorial_hidden = !state.tutorial_hidden;
    }

    // Harvest terrain with H key
    if is_key_pressed(KeyCode::H) {
        if let Some(pos) = hovered_pos {
            state.try_harvest_terrain(pos);
        }
    }

    // Convert forest to dust filter
    if is_key_pressed(KeyCode::F) {
        if let Some(pos) = hovered_pos {
            state.try_convert_forest_to_filter(pos);
        }
    }

    // Place building on click
    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(pos) = hovered_pos {
            let mut placed = false;
            if let Some(building_type) = state.selected_building {
                if state.grid.can_place_building(pos, building_type) {
                    placed = state.try_place_building(pos);
                }
            }

            if placed {
                state.drag_last_pos = Some(pos);
            } else {
                state.selected_tile = Some(pos);
            }
        }
    }

    // Drag placement while holding left mouse
    if is_mouse_button_down(MouseButton::Left) {
        if let Some(pos) = hovered_pos {
            if state.drag_last_pos != Some(pos) {
                if state.selected_building == Some(BuildingType::Conduit) {
                    if let Some(start_pos) = state.drag_last_pos {
                        if state.try_place_conduit_path(start_pos, pos) {
                            state.drag_last_pos = Some(pos);
                        }
                    }
                } else {
                    state.drag_last_pos = Some(pos);
                    state.try_place_building(pos);
                }
            }
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        state.drag_last_pos = None;
    }

    // Right click to cancel selection or harvest
    if is_mouse_button_pressed(MouseButton::Right) {
        if let Some(pos) = hovered_pos {
            if !state.try_harvest_terrain(pos) {
                state.clear_selection();
            }
        } else {
            state.clear_selection();
        }
        state.selected_tile = None;
    }

    // Navigation keys
    if is_key_pressed(KeyCode::Escape) {
        state.selected_tile = None;
        return PlanetaryAction::OpenMenu;
    }
    if is_key_pressed(KeyCode::R) {
        return PlanetaryAction::OpenResearch;
    }
    if is_key_pressed(KeyCode::M) {
        return PlanetaryAction::OpenInterplanetary;
    }

    PlanetaryAction::None
}
