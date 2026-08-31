//! Shared capture-session reset for deterministic screenshot staging.

use crate::{engine, state, Game, GamePhase};

impl Game {
    pub(super) fn reset_capture_scene(&mut self) {
        // A manifest is captured in one process, so start every image from the
        // same pristine campaign. Scene staging is allowed to build on that
        // campaign, but travel, research, and launch state must not cross the
        // boundary into the next image.
        let capture_config = self.config.clone();
        self.new_campaign(capture_config, 42);
        self.refresh_save_presence();
        self.menu_notice = None;
        self.launch = None;
        self.research_state = engine::ResearchState::from_data(self.data.data());
        self.settings_tab = crate::screens::SettingsTab::Display;
        self.research_return_phase = GamePhase::Playing;
        self.capture_still = true;
        // Scene-specific view state starts from the same quiet map as well.
        self.campaign.current_mut().focus_mode = false;
        self.campaign.current_mut().mobile_panel = state::MobilePanel::Map;
        self.campaign.current_mut().flow_overlay = false;
        self.campaign.current_mut().factory_deck_open = false;
        self.campaign.current_mut().show_help = false;
        self.campaign.current_mut().auto_clocking = false;
        self.campaign.current_mut().save_failed = false;
        self.has_active_campaign = false;
        self.campaign.current_mut().restored_from_backup = false;
        self.campaign.current_mut().restored_from_backup_generation = 0;
        self.research_state.current_research = None;
        self.research_state.research_progress = 0.0;
        self.research_state.parked_progress.clear();
        self.launch_confirmation_target = None;
    }
}
