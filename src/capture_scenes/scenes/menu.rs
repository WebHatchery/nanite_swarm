//! Menu and settings staging for the screenshot harness.

use crate::{slot_name, Game, GamePhase};

impl Game {
    pub(super) fn stage_menu_capture_scene(&mut self, scene: &str) -> bool {
        match scene {
            "mainmenu" => {
                self.phase = GamePhase::MainMenu;
                self.menu_notice = None;
                self.delete_confirmation_slot = None;
            }
            "desktop_mainmenu_short" | "mobile_mainmenu_narrow" => {
                self.phase = GamePhase::MainMenu;
                self.has_save = true;
                self.has_active_campaign = true;
                self.menu_notice = None;
                self.delete_confirmation_slot = None;
            }
            "save_failed_menu" | "mobile_save_failed_menu" => {
                self.phase = GamePhase::MainMenu;
                self.has_save = true;
                self.has_active_campaign = true;
                self.menu_notice =
                    Some("Save failed: progress is in memory. Tap SAVE to retry.".to_string());
                self.campaign.current_mut().save_failed = true;
                self.delete_confirmation_slot = None;
            }
            "mainmenu_confirm_delete" | "mobile_mainmenu_confirm_delete" => {
                self.phase = GamePhase::MainMenu;
                self.has_save = true;
                self.delete_confirmation_slot = Some(self.active_slot);
                self.menu_notice = Some(format!(
                    "Tap Delete Slot again to permanently clear {}.",
                    slot_name(self.active_slot)
                ));
            }
            "settings" | "desktop_settings_short" | "mobile_settings_narrow" => {
                self.phase = GamePhase::Settings;
                self.settings_tab = crate::screens::SettingsTab::Display;
            }
            "settings_audio"
            | "mobile_settings_audio"
            | "mobile_settings_audio_narrow"
            | "desktop_settings_audio_short" => {
                self.phase = GamePhase::Settings;
                self.settings_tab = crate::screens::SettingsTab::Audio;
            }
            _ => return false,
        }
        true
    }
}
