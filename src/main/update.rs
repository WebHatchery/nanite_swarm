//! Frame orchestration and screen-phase transitions.
//!
//! The binary entrypoint owns construction and the platform loop; this module
//! owns the application coordinator that advances the campaign and translates
//! screen actions into state changes.

use macroquad::prelude::*;

use crate::screens::{
    render_campaign_complete_view, render_interplanetary_view, render_launch_view,
    render_main_menu, render_planetary_view, render_records_view, render_research_view,
    render_seed_ship_view, render_settings_menu, CampaignCompleteAction, InterplanetaryAction,
    LaunchAction, MenuAction, PlanetaryAction, RecordsAction, ResearchAction, SeedShipAction,
    SettingsAction,
};
use crate::state::{self, load_from_file, save_exists, save_to_file, GAME_NAME};
use crate::{display, screens};

use super::{save_path, slot_name, Game, GamePhase, SLOT_NAMES};

impl Game {
    /// Check if mass driver technology is researched.
    fn has_mass_driver(&self) -> bool {
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

    fn update_main_menu(&mut self) {
        match render_main_menu(
            self.has_save,
            self.menu_notice.as_deref(),
            slot_name(self.active_slot),
        ) {
            MenuAction::NewGame => {
                self.menu_notice = None;
                self.campaign =
                    state::Campaign::new(self.config.clone(), macroquad_toolkit::rng::random_u64());
                self.campaign.set_slot_name(slot_name(self.active_slot));
                self.campaign
                    .apply_preferred_speed(self.settings.default_speed);
                self.research_state = crate::engine::ResearchState::default();
                self.sync_research_to_planet();
                self.sync_building_unlocks();
                self.phase = GamePhase::Playing;
            }
            MenuAction::Continue | MenuAction::Load => self.load_campaign(),
            MenuAction::Save => self.save_campaign(),
            MenuAction::CycleSlot => {
                self.active_slot = (self.active_slot + 1) % SLOT_NAMES.len();
                self.has_save = save_exists(&save_path(self.active_slot));
                self.menu_notice = Some(format!("Selected {}", slot_name(self.active_slot)));
            }
            MenuAction::Delete => {
                if state::delete_save(&save_path(self.active_slot)).is_ok() {
                    self.has_save = false;
                    self.menu_notice = Some(format!("Cleared {}", slot_name(self.active_slot)));
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

    fn update_settings(&mut self) {
        let before = self.settings.clone();
        let action = render_settings_menu(&mut self.settings);
        if self.settings != before {
            self.settings.sanitize();
            display::apply_display_settings(&self.settings, Some(&before));
            let _ = self.settings.save(GAME_NAME);
        }
        if action == SettingsAction::Back {
            self.phase = GamePhase::MainMenu;
        }
    }

    fn update_playing(&mut self) {
        self.advance_simulation();
        let (planet, directive) = self.campaign.current_and_directive();
        let action = render_planetary_view(planet, &self.textures, directive, &self.ui_theme);
        self.remember_preferred_speed();
        match action {
            PlanetaryAction::OpenResearch => self.open_screen(GamePhase::Research),
            PlanetaryAction::OpenSeedShip => self.open_screen(GamePhase::SeedShip),
            PlanetaryAction::OpenInterplanetary => self.open_screen(GamePhase::Interplanetary),
            PlanetaryAction::OpenRecords => self.open_screen(GamePhase::Records),
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

    fn update_research_screen(&mut self) {
        self.advance_simulation();
        let sheet = self.campaign.current().stat_sheet();
        match render_research_view(
            &self.research_state,
            &self.research_tree,
            self.campaign.current().resources.data,
            self.campaign.current().research_lock_timer > 0.0,
            &sheet,
            self.campaign.current().active_planet_condition(),
            &mut self.research_viewport,
        ) {
            ResearchAction::Close => self.phase = GamePhase::Playing,
            ResearchAction::StartResearch(tech_id) => {
                let condition = self.campaign.current().active_planet_condition();
                if self.research_tree.can_select_on(
                    &tech_id,
                    &self.research_state.unlocked,
                    condition,
                ) {
                    let _ = self.research_state.start_research(
                        &tech_id,
                        &self.research_tree,
                        self.campaign.current().resources.data,
                    );
                }
            }
            ResearchAction::None => {}
        }
    }

    fn update_seed_ship_screen(&mut self) {
        self.advance_simulation();
        match render_seed_ship_view(self.campaign.current()) {
            SeedShipAction::Close => self.phase = GamePhase::Playing,
            SeedShipAction::ToggleCommitment => {
                self.campaign.current_mut().toggle_seed_ship_commitment();
            }
            SeedShipAction::None => {}
        }
    }

    fn update_records(&mut self) {
        self.advance_simulation();
        self.campaign.sync_notification_history();
        let log = self.campaign.toast_history.clone();
        let directive_history = self.campaign.directive_history.clone();
        let planet = self.campaign.current_mut();
        let records = planet.achievement_records();
        let name = planet.name.clone();
        let action = render_records_view(
            &name,
            &records,
            &log,
            &directive_history,
            &mut planet.log_scroll,
            &mut planet.records_scroll,
        );
        if action == RecordsAction::Close {
            self.phase = GamePhase::Playing;
        }
    }

    fn update_campaign_complete(&mut self) {
        match render_campaign_complete_view(&self.campaign) {
            CampaignCompleteAction::KeepGoing => self.phase = GamePhase::Playing,
            CampaignCompleteAction::Close => {
                self.save_campaign();
                self.phase = GamePhase::MainMenu;
            }
            CampaignCompleteAction::None => {}
        }
    }

    fn update_interplanetary(&mut self) {
        self.advance_simulation();
        let stockpiles: [Option<f32>; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.stockpile(index));
        let colonized = self.campaign.colonized_flags();
        let pads: [usize; state::PLANET_COUNT] = std::array::from_fn(|index| {
            self.campaign
                .planet(index)
                .map(|planet| planet.landing_pads_online())
                .unwrap_or(0)
        });
        let orders: [Option<state::ExportOrder>; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.export_order_for(index));
        let pending_pods: [usize; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.pending_pod_count(index));
        let pod_caps: [usize; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.pending_pod_cap(index));
        let overflow_pods: [usize; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.overflow_pod_count(index));
        let view = screens::MapView {
            current_planet: self.campaign.current_index(),
            has_mass_driver: self.has_mass_driver(),
            seed_ship_ready: self.campaign.current().seed_ship.is_ready_to_launch(),
            colonized: &colonized,
            stockpiles: &stockpiles,
            drivers_online: self.campaign.current().mass_drivers_online(),
            pads: &pads,
            export: self.campaign.export_order(),
            pod_fraction: self.campaign.current().pod_fraction(),
            shipments: self.campaign.shipments(),
            orders: &orders,
            pending_pods: &pending_pods,
            pod_caps: &pod_caps,
            overflow_pods: &overflow_pods,
            editing_world: self.shipping_edit_world,
        };
        match render_interplanetary_view(&view) {
            InterplanetaryAction::Close => self.phase = GamePhase::Playing,
            InterplanetaryAction::SelectPlanet(index) => {
                if self.campaign.travel_to(index) {
                    self.sync_research_to_planet();
                    self.sync_building_unlocks();
                    self.save_campaign();
                    self.phase = GamePhase::Playing;
                }
            }
            InterplanetaryAction::LaunchSeedShip(index) => {
                let origin = self.campaign.current_index();
                if self.has_mass_driver() && self.campaign.launch_seed_ship(index) {
                    self.sync_research_to_planet();
                    self.sync_building_unlocks();
                    self.save_campaign();
                    self.campaign.current_mut().arrival_notice_timer = 0.0;
                    self.launch = Some(state::LaunchSequence::new(origin, index));
                    self.phase = GamePhase::Launch;
                }
            }
            InterplanetaryAction::CycleExportCargo => self.campaign.cycle_export_cargo(),
            InterplanetaryAction::CycleExportTarget => self.campaign.cycle_export_target(),
            InterplanetaryAction::CycleExportPad => {
                self.campaign.cycle_export_pad_for(self.shipping_edit_world)
            }
            InterplanetaryAction::CycleExportSchedule => self
                .campaign
                .cycle_export_schedule_for(self.campaign.current_index()),
            InterplanetaryAction::CycleExportPriority => self
                .campaign
                .cycle_export_priority_for(self.campaign.current_index()),
            InterplanetaryAction::ToggleExportSurplus => self
                .campaign
                .toggle_export_surplus_for(self.campaign.current_index()),
            InterplanetaryAction::SelectOrderWorld(index) => self.shipping_edit_world = index,
            InterplanetaryAction::CycleRemoteExportCargo => self
                .campaign
                .cycle_export_cargo_for(self.shipping_edit_world),
            InterplanetaryAction::CycleRemoteExportTarget => self
                .campaign
                .cycle_export_target_for(self.shipping_edit_world),
            InterplanetaryAction::CycleRemoteExportSchedule => self
                .campaign
                .cycle_export_schedule_for(self.shipping_edit_world),
            InterplanetaryAction::CycleRemoteExportPriority => self
                .campaign
                .cycle_export_priority_for(self.shipping_edit_world),
            InterplanetaryAction::ToggleRemoteExportSurplus => self
                .campaign
                .toggle_export_surplus_for(self.shipping_edit_world),
            InterplanetaryAction::None => {}
        }
    }

    fn update_launch(&mut self) {
        let arrival_line = self.campaign.current().arrival_line();
        let origin_state = self
            .launch
            .as_ref()
            .and_then(|sequence| self.campaign.planet(sequence.origin()));
        let Some(sequence) = self.launch.as_mut() else {
            self.phase = GamePhase::Playing;
            return;
        };
        if !self.capture_still {
            sequence.advance(get_frame_time());
        }
        let action = render_launch_view(sequence, arrival_line, origin_state);
        if action == LaunchAction::Skip {
            sequence.skip();
        }
        if sequence.is_finished() {
            self.launch = None;
            self.phase = GamePhase::Playing;
        }
    }

    fn draw_toasts(&mut self) {
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
        self.campaign.current_mut().take_audio_events();
    }

    /// Advance the world and all campaign-level timers by the same simulated time.
    fn advance_simulation(&mut self) {
        let ticks = self.campaign.current_mut().advance(get_frame_time(), true);
        if ticks == 0 {
            return;
        }
        let simulated = ticks as f32 * state::TICK_SECONDS;
        self.update_research(simulated);
        self.campaign.update_directive(simulated);
        self.campaign.update_background(simulated);
        self.campaign.update_shipments(simulated);
        self.campaign.sync_notification_history();
        self.check_campaign_complete();
        if self
            .campaign
            .due_for_autosave(self.settings.autosave_interval)
        {
            self.save_campaign();
        }
    }

    fn check_campaign_complete(&mut self) {
        if self.ending_seen || !self.campaign.is_complete() {
            return;
        }
        self.ending_seen = true;
        self.campaign
            .current_mut()
            .announce_achievement("system_consumed");
        self.save_campaign();
        self.phase = GamePhase::CampaignComplete;
    }

    fn save_campaign(&mut self) {
        self.campaign.set_slot_name(slot_name(self.active_slot));
        match save_to_file(&mut self.campaign, &save_path(self.active_slot)) {
            Ok(()) => {
                self.has_save = true;
                self.campaign.mark_saved();
                self.campaign.current_mut().save_failed = false;
            }
            Err(_) => self.campaign.current_mut().save_failed = true,
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
        match load_from_file(&save_path(self.active_slot)) {
            Ok((campaign, source)) => {
                self.campaign = campaign;
                self.campaign
                    .apply_preferred_speed(self.settings.default_speed);
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
                self.has_save = true;
                self.phase = GamePhase::Playing;
            }
            Err(error) => {
                eprintln!("Could not load campaign: {error}");
                self.has_save = save_exists(&save_path(self.active_slot));
                self.menu_notice = Some("Load failed: save is missing or corrupt.".to_string());
            }
        }
    }
}
