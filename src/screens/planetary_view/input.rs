//! Keyboard and mouse input handling for the planetary view

use crate::engine::{BuildingType, GridPos};
use crate::state::PlanetState;
use macroquad::prelude::*;
use macroquad_toolkit::input::TouchGestureFrame;

use super::format::keycode_from_hotkey;
use super::metrics::HudMetrics;
use super::PlanetaryAction;

fn placement_block_message(
    state: &PlanetState,
    pos: GridPos,
    building_type: BuildingType,
) -> Option<String> {
    state
        .placement_block_reason(pos, building_type)
        .map(|reason| reason.message(state, building_type))
}

fn try_place_selected_building(
    state: &mut PlanetState,
    pos: GridPos,
    building_type: BuildingType,
) -> bool {
    if let Some(message) = placement_block_message(state, pos, building_type) {
        state.notifications.warning(message);
        return false;
    }
    state.try_place_building(pos)
}

fn handle_existing_building_primary(state: &mut PlanetState, pos: GridPos, arm_smart_drag: bool) {
    state.selected_tile = Some(pos);
    if arm_smart_drag {
        state.smart_drag_source = Some(pos);
        state.drag_last_pos = Some(pos);
    }
}

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
    touch: TouchGestureFrame,
    touch_camera_active: bool,
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

    if touch_camera_active && touch.claimed {
        state.camera.pan_by(touch.pan.x, touch.pan.y);
        if touch.scale != 1.0 {
            let center = (
                touch.center.x - metrics.base_offset_x(),
                touch.center.y - metrics.base_offset_y(),
            );
            state.camera.zoom_by_scale(touch.scale, center);
        }
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
    touch_tap: Option<GridPos>,
    touch_drag_pos: Option<GridPos>,
    touch: TouchGestureFrame,
) -> PlanetaryAction {
    let mouse_input = touches().is_empty();
    let activated_pos = touch_tap.or(hovered_pos);
    let primary_pressed =
        touch_tap.is_some() || (mouse_input && is_mouse_button_pressed(MouseButton::Left));
    if state.box_select_mode
        && mouse_input
        && is_mouse_button_released(MouseButton::Left)
        && state.box_select_start.is_some()
    {
        if let Some(pos) = hovered_pos {
            state.finish_box_select(pos);
            return PlanetaryAction::None;
        }
    }
    // Building hotkeys
    let hotkeys: Vec<_> = state
        .data()
        .buildings
        .iter()
        .map(|def| (def.id.clone(), def.hotkey.clone()))
        .collect();
    for (id, hotkey) in hotkeys {
        let Some(hotkey) = hotkey.as_ref().and_then(|key| key.chars().next()) else {
            continue;
        };
        let Some(keycode) = keycode_from_hotkey(hotkey) else {
            continue;
        };
        if !is_key_pressed(keycode) {
            continue;
        }
        let Some(building_type) = BuildingType::from_id(&id) else {
            continue;
        };
        if state.is_building_unlocked(building_type) {
            state.select_building(building_type);
        }
    }

    if is_key_pressed(KeyCode::Space) {
        state.toggle_pause();
    }
    if is_key_pressed(KeyCode::X) {
        state.toggle_demolish_mode();
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

    // Demolition takes the same clicks as building, so it comes first and
    // returns rather than falling through into placement.
    if state.demolish_mode {
        if primary_pressed
            || (mouse_input
                && is_mouse_button_down(MouseButton::Left)
                && state.drag_last_pos != hovered_pos)
        {
            if let Some(pos) = activated_pos {
                state.try_sell_building(pos);
                state.drag_last_pos = Some(pos);
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            state.drag_last_pos = None;
            state.route_resource_blocked_at = None;
            state.route_path_blocked_at = None;
        }
        // Right click or Escape puts the wrecking ball down.
        if is_mouse_button_pressed(MouseButton::Right) || is_key_pressed(KeyCode::Escape) {
            state.cancel_interaction_modes();
            return PlanetaryAction::None;
        }
        if is_key_pressed(KeyCode::R) {
            return PlanetaryAction::OpenResearch;
        }
        if is_key_pressed(KeyCode::M) {
            return PlanetaryAction::OpenInterplanetary;
        }
        if is_key_pressed(KeyCode::A) {
            return PlanetaryAction::OpenRecords;
        }
        return PlanetaryAction::None;
    }

    // A one-finger touch drag is a route gesture while Conduit is selected, or
    // when it starts on an existing building. Seed the route from the first
    // tile, then let the path planner lay down each newly crossed segment. A
    // short touch that never becomes claimed falls through to tap-to-place.
    if state.selected_building == Some(BuildingType::Conduit)
        || state.smart_drag_source.is_some()
        || touch_drag_pos.is_some()
    {
        if let Some(pos) = touch_drag_pos {
            if state.drag_last_pos.is_none() {
                state.drag_last_pos = Some(pos);
                if state.selected_building != Some(BuildingType::Conduit)
                    && state
                        .grid
                        .get(pos)
                        .and_then(|tile| tile.building.as_ref())
                        .is_some()
                {
                    state.smart_drag_source = Some(pos);
                    state.selected_tile = Some(pos);
                }
            }
            if touch.claimed {
                if let Some(start_pos) = state.drag_last_pos {
                    let routed = if state.smart_drag_source.is_some() {
                        state.try_place_smart_conduit_path(start_pos, pos)
                    } else {
                        start_pos != pos && state.try_place_conduit_path(start_pos, pos)
                    };
                    if routed {
                        state.drag_last_pos = Some(pos);
                    }
                }
            }
        } else if !touch.active {
            // The release frame carries no active touch position.
            state.drag_last_pos = None;
            state.route_resource_blocked_at = None;
            state.route_path_blocked_at = None;
            state.smart_drag_source = None;
        }
    }

    // Place building on click
    if primary_pressed {
        if let Some(pos) = activated_pos {
            if state.box_select_mode {
                state.finish_box_select(pos);
                return PlanetaryAction::None;
            }
            if let Some(source) = state.relocation_source {
                state.relocate_building(source, pos);
                state.relocation_source = None;
                state.selected_tile = Some(pos);
                return PlanetaryAction::None;
            }
            let mut placed = false;
            let starts_smart_drag = state
                .grid
                .get(pos)
                .and_then(|tile| tile.building.as_ref())
                .is_some();
            if starts_smart_drag {
                // A touch tap is an inspection/select action. Smart Drag only
                // arms for a mouse press or an already-claimed gesture; a
                // short touch must not leave a route source behind.
                handle_existing_building_primary(state, pos, mouse_input || touch.claimed);
            } else if let Some(building_type) = state.selected_building {
                placed = try_place_selected_building(state, pos, building_type);
            }

            if placed {
                state.drag_last_pos = Some(pos);
            } else {
                state.selected_tile = Some(pos);
            }
        }
    }

    // Drag placement while holding left mouse
    if mouse_input && is_mouse_button_down(MouseButton::Left) {
        if let Some(pos) = hovered_pos {
            if state.drag_last_pos != Some(pos) {
                if state.selected_building == Some(BuildingType::Conduit) {
                    if let Some(start_pos) = state.drag_last_pos {
                        if state.try_place_conduit_path(start_pos, pos) {
                            state.drag_last_pos = Some(pos);
                        }
                    }
                } else if state.smart_drag_source.is_some() {
                    if let Some(start_pos) = state.drag_last_pos {
                        if state.try_place_smart_conduit_path(start_pos, pos) {
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
        state.route_resource_blocked_at = None;
        state.route_path_blocked_at = None;
        state.smart_drag_source = None;
    }

    // Right click to cancel selection or harvest
    if is_mouse_button_pressed(MouseButton::Right) {
        if let Some(pos) = hovered_pos {
            if !state.try_harvest_terrain(pos) {
                state.cancel_interaction_modes();
            }
        } else {
            state.cancel_interaction_modes();
        }
    }

    // Navigation keys
    if is_key_pressed(KeyCode::Escape) {
        state.cancel_interaction_modes();
        return PlanetaryAction::OpenMenu;
    }
    if is_key_pressed(KeyCode::R) {
        return PlanetaryAction::OpenResearch;
    }
    if is_key_pressed(KeyCode::M) {
        return PlanetaryAction::OpenInterplanetary;
    }
    if is_key_pressed(KeyCode::A) {
        return PlanetaryAction::OpenRecords;
    }

    PlanetaryAction::None
}

#[cfg(test)]
mod tests;
