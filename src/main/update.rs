//! Frame orchestration and screen-phase transitions.
//!
//! The binary entrypoint owns construction and the platform loop; this module
//! owns the application coordinator that advances the campaign and translates
//! screen actions into state changes.

use crate::screens;
use crate::state::{self, GAME_NAME};
use macroquad::prelude::*;

use super::{Game, GamePhase};

#[path = "update/interplanetary.rs"]
mod interplanetary;
#[path = "update/phases.rs"]
mod phases;
#[path = "update/simulation.rs"]
mod simulation;

impl Game {
    /// Check if mass driver technology is researched.
    pub(super) fn has_mass_driver(&self) -> bool {
        self.research_state.is_unlocked("mass_driver")
    }

    /// Run one application frame and dispatch input to the active screen.
    pub fn update(&mut self) {
        self.debug_overlay.record_frame(get_frame_time());
        self.debug_overlay.visible = self.settings.show_fps;
        self.refresh_audio_mix();

        match self.phase {
            GamePhase::MainMenu => self.update_main_menu(),
            GamePhase::Settings => self.update_settings(),
            GamePhase::Playing => self.update_playing(),
            GamePhase::Research => self.update_research_screen(),
            GamePhase::SeedShip => self.update_seed_ship_screen(),
            GamePhase::Records => self.update_records(),
            GamePhase::CampaignComplete => self.update_campaign_complete(),
            GamePhase::Interplanetary => self.update_interplanetary(),
            GamePhase::Launch => self.update_launch(),
        }

        self.draw_toasts();
        self.debug_overlay.draw(&[]);
    }

    fn draw_toasts(&mut self) {
        // Compact campaign screens use nearly every pixel for their touch
        // targets. Keep notifications in the Records log there rather than
        // painting transient cards over buttons and scrollable content.
        if screen_width() < 600.0
            || !screens::menu_toasts_fit_viewport(screen_width(), screen_height())
        {
            return;
        }
        match self.phase {
            GamePhase::SeedShip => {
                let anchor = screens::menu_anchor_low(screen_height());
                screens::draw_toasts(self.campaign.current(), anchor);
            }
            GamePhase::Research | GamePhase::Interplanetary => {
                let anchor = screens::menu_anchor(screen_width());
                screens::draw_toasts(self.campaign.current(), anchor);
            }
            _ => {}
        }
    }

    fn refresh_audio_mix(&mut self) {
        self.audio_mix = if self.phase == GamePhase::MainMenu {
            state::AudioMix {
                music_state: state::MusicState::Menu,
                swarm_scale: 0.0,
                sfx_volume: self.settings.effective_sfx_volume(),
                music_volume: self.settings.effective_music_volume(),
            }
        } else {
            state::AudioMix::for_planet(
                self.campaign.current(),
                self.settings.effective_sfx_volume(),
                self.settings.effective_music_volume(),
            )
        };
        let events = self.campaign.current_mut().take_audio_events();
        self.audio.update(
            self.audio_mix,
            events,
            is_mouse_button_pressed(MouseButton::Left)
                || is_mouse_button_pressed(MouseButton::Right)
                || touches()
                    .iter()
                    .any(|touch| matches!(touch.phase, TouchPhase::Started))
                || get_last_key_pressed().is_some(),
            get_frame_time(),
        );
    }

    fn save_campaign(&mut self) {
        match self.session.save_campaign() {
            Ok(()) => {
                self.campaign.current_mut().save_failed = false;
                if self
                    .menu_notice
                    .as_deref()
                    .is_some_and(|notice| notice.starts_with("Save failed:"))
                {
                    self.menu_notice = None;
                }
            }
            Err(error) => {
                eprintln!("Could not save campaign: {error}");
                self.campaign.current_mut().save_failed = true;
                self.menu_notice = Some(save_failure_notice());
            }
        }
    }

    fn remember_preferred_speed(&mut self) {
        let speed = self.campaign.current().time_scale;
        let Some(index) = state::TIME_SCALES
            .iter()
            .position(|candidate| (*candidate - speed).abs() < 0.001)
        else {
            return;
        };
        let index = index as i32;
        if self.settings.default_speed != index {
            self.settings.default_speed = index;
            let _ = self.settings.save(GAME_NAME);
        }
    }

    fn load_campaign(&mut self) {
        match self.session.load_campaign() {
            Ok(source) => {
                let preferred_speed = self.settings.default_speed;
                self.campaign.apply_preferred_speed(preferred_speed);
                self.campaign.adopt_planet_research();
                self.sync_research_from_planet();
                self.sync_building_unlocks();
                self.campaign.current_mut().restored_from_backup =
                    source == state::LoadSource::Backup;
                if source == state::LoadSource::Backup {
                    let generation = self.campaign.current().restored_from_backup_generation;
                    self.menu_notice = Some(format!(
                        "Recovered from backup generation {}",
                        generation.max(1)
                    ));
                }
                self.phase = GamePhase::Playing;
            }
            Err(error) => {
                eprintln!("Could not load campaign: {error}");
                self.session.refresh_save_presence();
                self.menu_notice = Some("Load failed: save is missing or corrupt.".to_string());
            }
        }
    }
}

fn save_failure_notice() -> String {
    "Save failed: progress is in memory. Tap SAVE to retry.".to_string()
}

#[cfg(test)]
mod tests;
