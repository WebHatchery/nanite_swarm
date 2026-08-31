//! Map interaction modes owned by a planetary state.

use crate::engine::BuildingType;

use super::super::game_state::PlanetState;

impl PlanetState {
    /// Select a building type for placement.
    pub fn select_building(&mut self, building_type: BuildingType) {
        if self.is_building_unlocked(building_type) {
            self.selected_building = Some(building_type);
            // Building and demolishing are the same click, so they cannot both
            // be armed.
            self.demolish_mode = false;
            self.relocation_source = None;
            self.box_select_mode = false;
            self.box_select_start = None;
            self.bulk_purge_armed = false;
            self.purge_armed = None;
            self.drag_last_pos = None;
            self.route_resource_blocked_at = None;
            self.route_path_blocked_at = None;
            self.smart_drag_source = None;
        }
    }

    /// Arm or disarm demolition. Picking it up puts the build cursor down.
    pub fn toggle_demolish_mode(&mut self) {
        self.demolish_mode = !self.demolish_mode;
        if self.demolish_mode {
            self.selected_building = None;
            self.relocation_source = None;
            self.box_select_mode = false;
            self.box_select_start = None;
            self.box_selected.clear();
            self.bulk_purge_armed = false;
            self.purge_armed = None;
            self.drag_last_pos = None;
            self.route_resource_blocked_at = None;
            self.route_path_blocked_at = None;
            self.smart_drag_source = None;
        }
    }

    /// Clear building selection.
    pub fn clear_selection(&mut self) {
        self.selected_building = None;
    }

    /// Cancel every pending map interaction without changing the saved
    /// blueprint or undo history. This is the shared path for the visible
    /// CANCEL control and for pointer cancellation, so a touch cannot leave a
    /// relocation or destructive confirmation armed behind the UI.
    pub fn cancel_interaction_modes(&mut self) {
        self.selected_building = None;
        self.selected_tile = None;
        self.demolish_mode = false;
        self.relocation_source = None;
        self.box_select_mode = false;
        self.box_select_start = None;
        self.box_selected.clear();
        self.purge_armed = None;
        self.bulk_purge_armed = false;
        self.drag_last_pos = None;
        self.route_resource_blocked_at = None;
        self.route_path_blocked_at = None;
        self.smart_drag_source = None;
        self.touch_camera_active = false;
        self.touch_route_active = false;
        self.touch_gesture_routed = false;
    }
}
