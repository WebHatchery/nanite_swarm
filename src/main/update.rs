//! Frame orchestration and screen-phase transitions.
//!
//! The binary entrypoint owns construction and the platform loop; this module
//! owns the application coordinator that advances the campaign and translates
//! screen actions into state changes.

use macroquad::prelude::*;
use macroquad_toolkit::ui::ScrollArea;

use crate::screens::{
    render_campaign_complete_view, render_interplanetary_view, render_launch_view,
    render_main_menu, render_planetary_view, render_records_view, render_research_view,
    render_seed_ship_view, render_settings_menu, CampaignCompleteAction, InterplanetaryAction,
    LaunchAction, MenuAction, PlanetaryAction, RecordsAction, ResearchAction, SeedShipAction,
    SettingsAction,
};
use crate::state::{self, GAME_NAME};
use crate::{display, screens};

use super::{save_path, slot_name, Game, GamePhase};

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
        let action = render_settings_menu(&mut self.settings, &mut self.settings_tab);
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
        let (session, textures, ui_theme) = (&mut self.session, &self.textures, &self.ui_theme);
        let (planet, directive) = session.campaign.current_and_directive();
        let action = render_planetary_view(planet, textures, directive, ui_theme);
        self.remember_preferred_speed();
        match action {
            PlanetaryAction::FastForwardToNextEvent => self.fast_forward_to_next_event(),
            PlanetaryAction::OpenResearch => self.open_screen(GamePhase::Research),
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

    fn update_research_screen(&mut self) {
        self.advance_simulation();
        let data_context = self.session.data.clone();
        let sheet = self.campaign.current().stat_sheet();
        match render_research_view(screens::ResearchViewContext {
            data: data_context.data(),
            research_state: &self.research_state,
            research_tree: &self.research_tree,
            data_available: self.campaign.current().resources.data,
            research_locked: self.campaign.current().research_lock_timer > 0.0,
            sheet: &sheet,
            active_condition: self.campaign.current().active_planet_condition(),
            viewport: &mut self.research_viewport,
        }) {
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
        let action = {
            let planet = self.campaign.current_mut();
            let mut scroll = std::mem::replace(&mut planet.seed_ship_scroll, ScrollArea::new());
            let action = render_seed_ship_view(planet, &mut scroll);
            planet.seed_ship_scroll = scroll;
            action
        };
        match action {
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
        let telemetry: [Option<state::WorldTelemetry>; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.world_telemetry(index));
        let colonized = self.campaign.colonized_flags();
        let pads: [usize; state::PLANET_COUNT] = std::array::from_fn(|index| {
            self.campaign
                .planet(index)
                .map(|planet| planet.landing_pads_online())
                .unwrap_or(0)
        });
        let orders: [Option<state::ExportOrder>; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.export_order_for(index));
        let drivers_online: [usize; state::PLANET_COUNT] = std::array::from_fn(|index| {
            self.campaign
                .planet(index)
                .map(|planet| planet.mass_drivers_online())
                .unwrap_or(0)
        });
        let pod_fractions: [f32; state::PLANET_COUNT] = std::array::from_fn(|index| {
            self.campaign
                .planet(index)
                .map(|planet| planet.pod_fraction())
                .unwrap_or(0.0)
        });
        let pending_pods: [usize; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.pending_pod_count(index));
        let pod_caps: [usize; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.pending_pod_cap(index));
        let overflow_pods: [usize; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.overflow_pod_count(index));
        let current_world = self.campaign.current_index();
        let mass_driver_researched = self.has_mass_driver();
        let seed_ship_ready = self.campaign.current().seed_ship.is_ready_to_launch();
        let editing_world = self.shipping_edit_world;
        let mut shipping_scroll = std::mem::replace(&mut self.shipping_scroll, ScrollArea::new());
        let campaign = &self.campaign;
        let data = &self.data;
        let view = screens::MapView {
            data: data.data(),
            current_planet: current_world,
            mass_driver_researched,
            seed_ship_ready,
            colonized: &colonized,
            stockpiles: &stockpiles,
            telemetry: &telemetry,
            current_drivers_online: drivers_online[current_world],
            drivers_online: &drivers_online,
            pads: &pads,
            pod_fractions: &pod_fractions,
            shipments: campaign.shipments(),
            orders: &orders,
            pending_pods: &pending_pods,
            pod_caps: &pod_caps,
            overflow_pods: &overflow_pods,
            editing_world,
        };
        let action = render_interplanetary_view(
            &view,
            &mut shipping_scroll,
            self.interplanetary_mobile_panel,
        );
        self.shipping_scroll = shipping_scroll;
        match action {
            InterplanetaryAction::Close => {
                // Freight orders are campaign state, not just map controls.
                // Save on exit so a browser player does not lose a route
                // configured immediately before leaving this screen.
                self.save_campaign();
                self.phase = GamePhase::Playing;
            }
            InterplanetaryAction::SelectPlanet(index) => {
                if self.campaign.travel_to(index) {
                    self.shipping_edit_world = index;
                    self.shipping_scroll.set_offset(0.0);
                    self.sync_research_to_planet();
                    self.sync_building_unlocks();
                    self.save_campaign();
                    self.phase = GamePhase::Playing;
                }
            }
            InterplanetaryAction::LaunchSeedShip(index) => {
                let origin = self.campaign.current_index();
                if self.has_mass_driver() && self.campaign.launch_seed_ship(index) {
                    self.shipping_edit_world = index;
                    self.shipping_scroll.set_offset(0.0);
                    self.sync_research_to_planet();
                    self.sync_building_unlocks();
                    self.save_campaign();
                    self.campaign.current_mut().arrival_notice_timer = 0.0;
                    self.launch = Some(state::LaunchSequence::new_with_data(
                        self.data.clone(),
                        origin,
                        index,
                    ));
                    self.phase = GamePhase::Launch;
                }
            }
            InterplanetaryAction::CycleExportCargo => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_cargo())
            }
            InterplanetaryAction::CycleExportTarget => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_target())
            }
            InterplanetaryAction::CycleExportPad => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_pad_for(editing_world))
            }
            InterplanetaryAction::CycleExportSchedule => self
                .save_shipping_edit(|campaign| campaign.cycle_export_schedule_for(current_world)),
            InterplanetaryAction::CycleExportPriority => self
                .save_shipping_edit(|campaign| campaign.cycle_export_priority_for(current_world)),
            InterplanetaryAction::ToggleExportSurplus => self
                .save_shipping_edit(|campaign| campaign.toggle_export_surplus_for(current_world)),
            InterplanetaryAction::SelectOrderWorld(index) => {
                self.shipping_edit_world = index;
                self.shipping_scroll.set_offset(0.0);
            }
            InterplanetaryAction::CycleRemoteExportCargo => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_cargo_for(editing_world))
            }
            InterplanetaryAction::CycleRemoteExportTarget => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_target_for(editing_world))
            }
            InterplanetaryAction::CycleRemoteExportSchedule => self
                .save_shipping_edit(|campaign| campaign.cycle_export_schedule_for(editing_world)),
            InterplanetaryAction::CycleRemoteExportPriority => self
                .save_shipping_edit(|campaign| campaign.cycle_export_priority_for(editing_world)),
            InterplanetaryAction::ToggleRemoteExportSurplus => self
                .save_shipping_edit(|campaign| campaign.toggle_export_surplus_for(editing_world)),
            InterplanetaryAction::ToggleMobileWorlds => {
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Worlds;
                self.shipping_scroll.set_offset(0.0);
            }
            InterplanetaryAction::ToggleMobileShipping => {
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Shipping;
                self.shipping_scroll.set_offset(0.0);
            }
            InterplanetaryAction::None => {}
        }
    }

    fn save_shipping_edit(&mut self, edit: impl FnOnce(&mut state::Campaign)) {
        edit(&mut self.campaign);
        self.save_campaign();
    }

    fn update_launch(&mut self) {
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

    fn draw_toasts(&mut self) {
        // Portrait campaign screens use nearly every pixel for their touch
        // targets. Keep notifications in the Records log there rather than
        // painting transient cards over buttons and scrollable content.
        if screen_width() < 600.0 {
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

    /// Advance the world and all campaign-level timers by the same simulated time.
    fn advance_simulation(&mut self) {
        let ticks = self.campaign.current_mut().advance(get_frame_time(), true);
        if ticks == 0 {
            return;
        }
        let simulated = ticks as f32 * state::TICK_SECONDS;
        self.advance_campaign_systems(simulated);
        self.finish_simulation_update();
    }

    /// Skip to the next meaningful timer without leaving campaign-level
    /// research, directives, background worlds, or freight behind.
    fn fast_forward_to_next_event(&mut self) {
        let Some(event) = self.campaign.current().next_interesting_event() else {
            return;
        };
        let mut remaining = event.seconds;
        while remaining > 0.0001 {
            let delta = remaining.min(state::TICK_SECONDS);
            self.campaign.current_mut().step(delta, false);
            self.advance_campaign_systems(delta);
            remaining -= delta;
        }
        self.campaign.current_mut().sim_accumulator = 0.0;
        self.finish_simulation_update();
    }

    fn advance_campaign_systems(&mut self, simulated: f32) {
        self.update_research(simulated);
        self.campaign.update_directive(simulated);
        self.campaign.update_background(simulated);
        self.campaign.update_shipments(simulated);
        self.campaign.sync_notification_history();
    }

    fn finish_simulation_update(&mut self) {
        self.check_campaign_complete();
        if self
            .campaign
            .due_for_autosave(self.settings.autosave_interval)
        {
            self.save_campaign();
        }
    }

    fn check_campaign_complete(&mut self) {
        if self.campaign.ending_seen || !self.campaign.is_complete() {
            return;
        }
        self.campaign.ending_seen = true;
        self.campaign
            .current_mut()
            .announce_achievement("system_consumed");
        self.save_campaign();
        self.phase = GamePhase::CampaignComplete;
    }

    fn save_campaign(&mut self) {
        match self.session.save_campaign() {
            Ok(()) => {
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
