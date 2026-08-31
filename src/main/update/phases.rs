//! Screen-phase handlers that translate view actions into session changes.

use macroquad::prelude::get_frame_time;
use macroquad_toolkit::ui::ScrollArea;

use crate::screens::{
    render_campaign_complete_view, render_launch_view, render_main_menu, render_planetary_view,
    render_records_view, render_research_view, render_seed_ship_view, render_settings_menu,
    CampaignCompleteAction, LaunchAction, MenuAction, PlanetaryAction, RecordsAction,
    ResearchAction, ResearchViewContext, SeedShipAction, SettingsAction,
};
use crate::state::{self, GAME_NAME};
use crate::{display, save_path, slot_name, Game, GamePhase};

impl Game {
    pub(super) fn update_main_menu(&mut self) {
        let action = render_main_menu(
            self.has_save,
            self.has_active_campaign,
            self.menu_notice.as_deref(),
            slot_name(self.active_slot),
            self.delete_confirmation_slot == Some(self.active_slot),
        );
        if !matches!(action, MenuAction::Delete | MenuAction::None) {
            self.delete_confirmation_slot = None;
        }
        match action {
            MenuAction::NewGame => {
                self.menu_notice = None;
                self.session
                    .new_campaign(self.config.clone(), macroquad_toolkit::rng::random_u64());
                let slot_name = slot_name(self.session.active_slot);
                let preferred_speed = self.settings.default_speed;
                self.session.campaign.set_slot_name(slot_name);
                self.session.campaign.apply_preferred_speed(preferred_speed);
                self.research_state = crate::engine::ResearchState::from_data(self.data.data());
                self.sync_research_to_planet();
                self.sync_building_unlocks();
                self.phase = GamePhase::Playing;
            }
            MenuAction::Continue | MenuAction::Load => self.load_campaign(),
            MenuAction::Save => self.save_campaign(),
            MenuAction::CycleSlot => {
                self.active_slot = (self.active_slot + 1) % 3;
                self.session.refresh_save_presence();
                self.menu_notice = Some(format!("Selected {}", slot_name(self.active_slot)));
            }
            MenuAction::Delete => {
                if self.delete_confirmation_slot == Some(self.active_slot) {
                    self.delete_confirmation_slot = None;
                    if state::delete_save(&save_path(self.active_slot)).is_ok() {
                        self.has_save = false;
                        self.menu_notice = Some(format!("Cleared {}", slot_name(self.active_slot)));
                    } else {
                        self.menu_notice = Some("Could not clear the selected slot.".to_string());
                    }
                } else {
                    self.delete_confirmation_slot = Some(self.active_slot);
                    self.menu_notice = Some(format!(
                        "Tap Delete Slot again to permanently clear {}.",
                        slot_name(self.active_slot)
                    ));
                }
            }
            MenuAction::Settings => self.phase = GamePhase::Settings,
            #[cfg(not(target_arch = "wasm32"))]
            MenuAction::Quit => {
                self.save_campaign();
                macroquad::miniquad::window::quit();
            }
            MenuAction::None => {}
        }
    }

    pub(super) fn update_settings(&mut self) {
        let before = self.settings.clone();
        let action = render_settings_menu(&mut self.settings, &mut self.settings_tab);
        if self.settings != before {
            self.settings.sanitize();
            display::apply_display_settings(&self.settings, Some(&before));
            let _ = self.settings.save(GAME_NAME);
        }
        match action {
            SettingsAction::Back => self.phase = GamePhase::MainMenu,
            SettingsAction::PreviewAudio => self
                .campaign
                .current_mut()
                .emit_audio(state::AudioEvent::UiConfirm),
            SettingsAction::None => {}
        }
    }

    pub(super) fn update_playing(&mut self) {
        self.advance_simulation();
        let (session, textures, ui_theme) = (&mut self.session, &self.textures, &self.ui_theme);
        let (planet, directive) = session.campaign.current_and_directive();
        let action = render_planetary_view(planet, textures, directive, ui_theme);
        self.remember_preferred_speed();
        match action {
            PlanetaryAction::FastForwardToNextEvent => self.fast_forward_to_next_event(),
            PlanetaryAction::OpenResearch => self.open_research_screen(GamePhase::Playing),
            PlanetaryAction::OpenSeedShip => self.open_screen(GamePhase::SeedShip),
            PlanetaryAction::OpenInterplanetary => self.open_screen(GamePhase::Interplanetary),
            PlanetaryAction::OpenRecords => self.open_screen(GamePhase::Records),
            PlanetaryAction::RetrySave => self.save_campaign(),
            PlanetaryAction::OpenMenu => {
                self.campaign
                    .current_mut()
                    .emit_audio(state::AudioEvent::UiBack);
                self.save_campaign();
                self.phase = GamePhase::MainMenu;
            }
            PlanetaryAction::None => {}
        }
    }

    fn open_screen(&mut self, phase: GamePhase) {
        self.campaign
            .current_mut()
            .emit_audio(state::AudioEvent::UiConfirm);
        self.phase = phase;
    }

    fn open_research_screen(&mut self, return_phase: GamePhase) {
        self.research_return_phase = return_phase;
        self.research_viewport.reset_for_entry();
        self.open_screen(GamePhase::Research);
    }

    pub(super) fn update_research_screen(&mut self) {
        self.advance_simulation();
        let data_context = self.session.data.clone();
        let (data_available, research_locked, tutorial_target, active_condition) = {
            let planet = self.campaign.current();
            (
                planet.resources.data,
                planet.research_lock_timer > 0.0,
                planet.tutorial_research_target().map(str::to_owned),
                planet.active_planet_condition().map(str::to_owned),
            )
        };
        let sheet = self.campaign.current().stat_sheet();
        match render_research_view(ResearchViewContext {
            data: data_context.data(),
            research_state: &self.research_state,
            research_tree: &self.research_tree,
            data_available,
            research_locked,
            tutorial_target: tutorial_target.as_deref(),
            sheet: &sheet,
            active_condition: active_condition.as_deref(),
            viewport: &mut self.research_viewport,
        }) {
            ResearchAction::Close => self.phase = self.research_return_phase,
            ResearchAction::StartResearch(tech_id) => {
                let condition = self.campaign.current().active_planet_condition();
                if self.research_tree.can_select_on(
                    &tech_id,
                    &self.research_state.unlocked,
                    condition,
                ) && self.research_state.start_research(
                    &tech_id,
                    &self.research_tree,
                    self.campaign.current().resources.data,
                ) {
                    // Selection is campaign state even before the first
                    // Data tick. Persist it immediately so opening the
                    // menu or closing the tab cannot erase the player's
                    // chosen research job.
                    self.sync_research_to_planet();
                }
            }
            ResearchAction::None => {}
        }
    }

    pub(super) fn update_seed_ship_screen(&mut self) {
        self.advance_simulation();
        let action = {
            let planet = self.campaign.current_mut();
            let mut scroll = std::mem::replace(&mut planet.seed_ship_scroll, ScrollArea::new());
            let action = render_seed_ship_view(planet, &mut scroll);
            planet.seed_ship_scroll = scroll;
            action
        };
        match action {
            SeedShipAction::Close => self.phase = GamePhase::Playing,
            SeedShipAction::OpenResearch => self.open_research_screen(GamePhase::SeedShip),
            SeedShipAction::ToggleCommitment => {
                self.campaign.current_mut().toggle_seed_ship_commitment();
                // Commitment changes the durable production plan. Save before
                // the player can leave the yard so a quick menu exit or tab
                // close cannot erase the decision.
                self.save_campaign();
            }
            SeedShipAction::None => {}
        }
    }

    pub(super) fn update_records(&mut self) {
        self.advance_simulation();
        self.campaign.sync_notification_history();
        let log = self.campaign.toast_history.clone();
        let directive_history = self.campaign.directive_history.clone();
        let consumption = self.campaign.total_consumption();
        let planet = self.campaign.current_mut();
        let records = planet.achievement_records();
        let name = planet.name.clone();
        let action = render_records_view(
            &name,
            &records,
            &log,
            &directive_history,
            consumption,
            &mut planet.log_scroll,
            &mut planet.records_scroll,
        );
        if action == RecordsAction::Close {
            self.phase = GamePhase::Playing;
        }
    }

    pub(super) fn update_campaign_complete(&mut self) {
        match render_campaign_complete_view(&self.campaign) {
            CampaignCompleteAction::KeepGoing => self.phase = GamePhase::Playing,
            CampaignCompleteAction::Close => {
                self.save_campaign();
                self.phase = GamePhase::MainMenu;
            }
            CampaignCompleteAction::None => {}
        }
    }

    pub(super) fn update_launch(&mut self) {
        let arrival_line = self.session.campaign.current().arrival_line().to_string();
        let origin_state = self
            .session
            .launch
            .as_ref()
            .and_then(|sequence| self.session.campaign.planet(sequence.origin()))
            .cloned();
        let capture_still = self.capture_still;
        let Some(sequence) = self.session.launch.as_mut() else {
            self.phase = GamePhase::Playing;
            return;
        };
        if !capture_still {
            sequence.advance(get_frame_time());
        }
        let action = render_launch_view(sequence, &arrival_line, origin_state.as_ref());
        if action == LaunchAction::Skip {
            sequence.skip();
        }
        if sequence.is_finished() {
            self.session.launch = None;
            self.phase = GamePhase::Playing;
        }
    }
}
