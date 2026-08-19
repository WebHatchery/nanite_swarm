//! Nanite Swarm - A self-replicating AI simulation
//!
//! Entry point, game loop, and phase transitions.

#![allow(clippy::too_many_arguments, clippy::wrong_self_convention)]

use macroquad::prelude::*;
use macroquad_toolkit::capture;
use macroquad_toolkit::debug::DebugOverlay;
use macroquad_toolkit::settings::GameSettings;

mod assets;
mod capture_scenes;
mod data;
mod directives;
mod display;
mod engine;
pub mod release;
mod screens;
mod state;
mod ui;

use assets::GameTextures;
use data::{load_game_config, load_game_data, load_ui_theme, set_game_data};
use engine::{ResearchState, ResearchTree};
use screens::{
    render_campaign_complete_view, render_interplanetary_view, render_launch_view,
    render_main_menu, render_planetary_view, render_records_view, render_research_view,
    render_seed_ship_view, render_settings_menu, CampaignCompleteAction, InterplanetaryAction,
    LaunchAction, MenuAction, PlanetaryAction, RecordsAction, ResearchAction, SeedShipAction,
    SettingsAction,
};
use state::{load_from_file, save_exists, save_to_file, Campaign, LaunchSequence, GAME_NAME};

/// Game phases/screens
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamePhase {
    MainMenu,
    Playing,
    Research,
    SeedShip,
    Interplanetary,
    Records,
    Settings,
    /// A ship is on its way. The world is not being simulated while it plays.
    Launch,
    /// The system is spent. Reachable once and then only by choice.
    CampaignComplete,
}

/// Main game state container
pub struct Game {
    phase: GamePhase,
    campaign: Campaign,
    research_tree: ResearchTree,
    research_state: ResearchState,
    settings: GameSettings,
    audio_mix: state::AudioMix,
    debug_overlay: DebugOverlay,
    has_save: bool,
    menu_notice: Option<String>,
    /// The ending has been shown once; seeing it again is the player's choice.
    ending_seen: bool,
    /// The launch being played out, if one is.
    launch: Option<LaunchSequence>,
    /// Staging a still frame for the screenshot harness. Anything that runs on
    /// real time has to hold where the scene put it, or the frame that gets
    /// written is wherever the capture's frame rate happened to carry it.
    capture_still: bool,
    textures: GameTextures,
    config: data::GameConfig,
    ui_theme: data::UiTheme,
    research_viewport: screens::ResearchViewport,
    shipping_edit_world: usize,
    active_slot: usize,
}

const SAVE_PATH: &str = "save.json";
const SLOT_NAMES: [&str; 3] = ["slot_1", "slot_2", "slot_3"];

fn slot_name(index: usize) -> &'static str {
    SLOT_NAMES[index % SLOT_NAMES.len()]
}

fn save_path(index: usize) -> String {
    format!("{}_{}", SAVE_PATH, slot_name(index))
}

impl Game {
    pub async fn new() -> Self {
        // Rajdhani's dynamic glyph atlas can resize several times on the first
        // text-heavy gameplay frame. WebGL then tries to flush batches that
        // still refer to the deleted atlas textures. Macroquad's built-in font
        // has a stable, prebuilt atlas and avoids that first-frame corruption.
        macroquad_toolkit::ui::use_macroquad_default_ui_font();
        #[cfg(not(target_arch = "wasm32"))]
        let config = load_game_config();
        #[cfg(target_arch = "wasm32")]
        let config = load_game_config().await;

        #[cfg(not(target_arch = "wasm32"))]
        let ui_theme = load_ui_theme();
        #[cfg(target_arch = "wasm32")]
        let ui_theme = load_ui_theme().await;

        #[cfg(not(target_arch = "wasm32"))]
        let game_data = load_game_data();
        #[cfg(target_arch = "wasm32")]
        let game_data = load_game_data().await;

        set_game_data(game_data);

        // Settings survive between sessions and are applied before the first
        // frame, so the player's text scale and fullscreen choice are already
        // in effect rather than snapping in when they visit the menu.
        let mut settings = GameSettings::load(GAME_NAME);
        settings.sanitize();
        display::apply_display_settings(&settings, None);

        let mut game = Self {
            phase: GamePhase::MainMenu,
            campaign: Campaign::new(config.clone(), 42),
            research_tree: ResearchTree::default(),
            research_state: ResearchState::default(),
            settings,
            audio_mix: state::AudioMix::default(),
            debug_overlay: DebugOverlay::new(),
            has_save: save_exists(&save_path(0)),
            menu_notice: None,
            ending_seen: false,
            launch: None,
            capture_still: false,
            textures: GameTextures::load().await,
            config,
            ui_theme,
            research_viewport: screens::ResearchViewport::default(),
            shipping_edit_world: state::STARTING_PLANET,
            active_slot: 0,
        };
        game.campaign
            .apply_preferred_speed(game.settings.default_speed);
        game
    }

    /// Check if mass driver technology is researched
    fn has_mass_driver(&self) -> bool {
        self.research_state.is_unlocked("mass_driver")
    }

    pub fn update(&mut self) {
        self.debug_overlay.record_frame(get_frame_time());
        self.debug_overlay.visible = self.settings.show_fps;
        self.refresh_audio_mix();

        match self.phase {
            GamePhase::MainMenu => {
                match render_main_menu(
                    self.has_save,
                    self.menu_notice.as_deref(),
                    slot_name(self.active_slot),
                ) {
                    MenuAction::NewGame => {
                        self.menu_notice = None;
                        self.campaign = Campaign::new(
                            self.config.clone(),
                            macroquad_toolkit::rng::random_u64(),
                        );
                        self.campaign.set_slot_name(slot_name(self.active_slot));
                        self.campaign
                            .apply_preferred_speed(self.settings.default_speed);
                        self.research_state = ResearchState::default();
                        self.sync_research_to_planet();
                        self.sync_building_unlocks();
                        self.phase = GamePhase::Playing;
                    }
                    MenuAction::Continue => {
                        self.load_campaign();
                    }
                    MenuAction::Load => {
                        self.load_campaign();
                    }
                    MenuAction::Save => {
                        self.save_campaign();
                    }
                    MenuAction::CycleSlot => {
                        self.active_slot = (self.active_slot + 1) % SLOT_NAMES.len();
                        self.has_save = save_exists(&save_path(self.active_slot));
                        self.menu_notice =
                            Some(format!("Selected {}", slot_name(self.active_slot)));
                    }
                    MenuAction::Delete => {
                        if state::delete_save(&save_path(self.active_slot)).is_ok() {
                            self.has_save = false;
                            self.menu_notice =
                                Some(format!("Cleared {}", slot_name(self.active_slot)));
                        }
                    }
                    MenuAction::Settings => {
                        self.phase = GamePhase::Settings;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    MenuAction::Quit => {
                        self.save_campaign();
                        macroquad::miniquad::window::quit();
                    }
                    MenuAction::None => {}
                }
            }
            GamePhase::Settings => {
                let before = self.settings.clone();
                let action = render_settings_menu(&mut self.settings);
                if self.settings != before {
                    // Applied as it changes so the player can see what a text
                    // scale actually does, and written down so it is still
                    // true next session.
                    self.settings.sanitize();
                    display::apply_display_settings(&self.settings, Some(&before));
                    let _ = self.settings.save(GAME_NAME);
                }
                if action == SettingsAction::Back {
                    self.phase = GamePhase::MainMenu;
                }
            }
            GamePhase::Playing => {
                self.advance_simulation();

                let (planet, directive) = self.campaign.current_and_directive();
                let action =
                    render_planetary_view(planet, &self.textures, directive, &self.ui_theme);
                self.remember_preferred_speed();
                match action {
                    PlanetaryAction::OpenResearch => {
                        self.campaign
                            .current_mut()
                            .emit_audio(state::AudioEvent::UiConfirm);
                        self.phase = GamePhase::Research;
                    }
                    PlanetaryAction::OpenSeedShip => {
                        self.campaign
                            .current_mut()
                            .emit_audio(state::AudioEvent::UiConfirm);
                        self.phase = GamePhase::SeedShip;
                    }
                    PlanetaryAction::OpenInterplanetary => {
                        self.campaign
                            .current_mut()
                            .emit_audio(state::AudioEvent::UiConfirm);
                        self.phase = GamePhase::Interplanetary;
                    }
                    PlanetaryAction::OpenRecords => {
                        self.campaign
                            .current_mut()
                            .emit_audio(state::AudioEvent::UiConfirm);
                        self.phase = GamePhase::Records;
                    }
                    PlanetaryAction::OpenMenu => {
                        self.campaign
                            .current_mut()
                            .emit_audio(state::AudioEvent::UiBack);
                        // Leaving the world is a good moment to write it down,
                        // and the menu used to only *claim* a save existed.
                        self.save_campaign();
                        self.phase = GamePhase::MainMenu;
                    }
                    PlanetaryAction::None => {}
                }
            }
            GamePhase::Research => {
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
                    ResearchAction::Close => {
                        self.phase = GamePhase::Playing;
                    }
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
            GamePhase::SeedShip => {
                self.advance_simulation();
                match render_seed_ship_view(self.campaign.current()) {
                    SeedShipAction::Close => {
                        self.phase = GamePhase::Playing;
                    }
                    SeedShipAction::ToggleCommitment => {
                        self.campaign.current_mut().toggle_seed_ship_commitment();
                    }
                    SeedShipAction::None => {}
                }
            }
            GamePhase::Records => {
                // The world keeps running while the player reads what it has
                // done, the same as the map and the research tree.
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
            GamePhase::CampaignComplete => match render_campaign_complete_view(&self.campaign) {
                CampaignCompleteAction::KeepGoing => {
                    self.phase = GamePhase::Playing;
                }
                CampaignCompleteAction::Close => {
                    self.save_campaign();
                    self.phase = GamePhase::MainMenu;
                }
                CampaignCompleteAction::None => {}
            },
            GamePhase::Interplanetary => {
                // The map is a view of a running system, not a pause button.
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
                    InterplanetaryAction::Close => {
                        self.phase = GamePhase::Playing;
                    }
                    InterplanetaryAction::SelectPlanet(index) => {
                        // The world being left keeps everything it had.
                        if self.campaign.travel_to(index) {
                            // The arriving world needs the campaign's research.
                            self.sync_research_to_planet();
                            self.sync_building_unlocks();
                            self.save_campaign();
                            self.phase = GamePhase::Playing;
                        }
                    }
                    InterplanetaryAction::LaunchSeedShip(index) => {
                        // The ship is spent carrying the swarm to a new world.
                        let origin = self.campaign.current_index();
                        if self.has_mass_driver() && self.campaign.launch_seed_ship(index) {
                            self.sync_research_to_planet();
                            self.sync_building_unlocks();
                            self.save_campaign();
                            // The campaign has already moved; what follows is
                            // only the telling of it. The vignette delivers the
                            // arrival line itself, so the top-bar notice that
                            // free travel uses would only repeat it.
                            self.campaign.current_mut().arrival_notice_timer = 0.0;
                            self.launch = Some(LaunchSequence::new(origin, index));
                            self.phase = GamePhase::Launch;
                        }
                    }
                    InterplanetaryAction::CycleExportCargo => {
                        self.campaign.cycle_export_cargo();
                    }
                    InterplanetaryAction::CycleExportTarget => {
                        self.campaign.cycle_export_target();
                    }
                    InterplanetaryAction::CycleExportPad => {
                        self.campaign.cycle_export_pad_for(self.shipping_edit_world);
                    }
                    InterplanetaryAction::CycleExportSchedule => {
                        self.campaign
                            .cycle_export_schedule_for(self.campaign.current_index());
                    }
                    InterplanetaryAction::CycleExportPriority => {
                        self.campaign
                            .cycle_export_priority_for(self.campaign.current_index());
                    }
                    InterplanetaryAction::ToggleExportSurplus => {
                        self.campaign
                            .toggle_export_surplus_for(self.campaign.current_index());
                    }
                    InterplanetaryAction::SelectOrderWorld(index) => {
                        self.shipping_edit_world = index;
                    }
                    InterplanetaryAction::CycleRemoteExportCargo => {
                        self.campaign
                            .cycle_export_cargo_for(self.shipping_edit_world);
                    }
                    InterplanetaryAction::CycleRemoteExportTarget => {
                        self.campaign
                            .cycle_export_target_for(self.shipping_edit_world);
                    }
                    InterplanetaryAction::CycleRemoteExportSchedule => {
                        self.campaign
                            .cycle_export_schedule_for(self.shipping_edit_world);
                    }
                    InterplanetaryAction::CycleRemoteExportPriority => {
                        self.campaign
                            .cycle_export_priority_for(self.shipping_edit_world);
                    }
                    InterplanetaryAction::ToggleRemoteExportSurplus => {
                        self.campaign
                            .toggle_export_surplus_for(self.shipping_edit_world);
                    }
                    InterplanetaryAction::None => {}
                }
            }
            GamePhase::Launch => {
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
        }

        // Toasts follow the player between screens. Anything that landed
        // while they were reading the tree or the map used to be missed
        // entirely, which is exactly when a finished directive matters.
        match self.phase {
            // The ship screen fills the middle, so its toasts go in the corner.
            // The ship screen fills the middle, so its toasts go in the corner.
            // Records draws none at all: the log on it is every message the
            // toasts would be repeating, and it does not fade.
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

        self.debug_overlay.draw(&[]);
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
        // The interface is intentionally drained even before a sound backend
        // is present, so events never grow without bound in long sessions.
        self.campaign.current_mut().take_audio_events();
    }

    /// Advance the world by whole simulation ticks. Research and directives run
    /// on exactly the time the planet simulated, so nothing drifts apart when
    /// the frame rate moves or a catch-up backlog is dropped.
    fn advance_simulation(&mut self) {
        let ticks = self.campaign.current_mut().advance(get_frame_time(), true);
        if ticks == 0 {
            return;
        }
        let simulated = ticks as f32 * state::TICK_SECONDS;
        self.update_research(simulated);
        self.campaign.update_directive(simulated);
        // The worlds nobody is looking at keep working.
        self.campaign.update_background(simulated);
        // And what they threw is still crossing the system.
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

    /// Show the ending the first time the system runs out of worlds.
    ///
    /// Only once: after that the player can go back to it from the map, but
    /// the game does not keep interrupting a finished campaign they have
    /// chosen to carry on playing.
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

    /// Write the campaign down, and say so on screen either way.
    ///
    /// A silent failed save is worse than no autosave at all: the player would
    /// carry on believing their world was safe.
    fn save_campaign(&mut self) {
        self.campaign.set_slot_name(slot_name(self.active_slot));
        match save_to_file(&mut self.campaign, &save_path(self.active_slot)) {
            Ok(()) => {
                self.has_save = true;
                self.campaign.mark_saved();
                self.campaign.current_mut().save_failed = false;
            }
            Err(_) => {
                self.campaign.current_mut().save_failed = true;
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
        match load_from_file(&save_path(self.active_slot)) {
            Ok((campaign, source)) => {
                self.campaign = campaign;
                self.campaign
                    .apply_preferred_speed(self.settings.default_speed);
                // A save written before research was campaign-wide keeps it
                // on the planet; take it from there once.
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

    fn update_research(&mut self, delta_time: f32) {
        let Some(current_id) = self.research_state.current_research.clone() else {
            self.sync_building_unlocks();
            self.sync_research_to_planet();
            return;
        };
        if self.campaign.current().research_lock_timer > 0.0 {
            return;
        }
        let Some(node) = self.research_tree.get_node(&current_id) else {
            self.research_state.current_research = None;
            self.research_state.research_progress = 0.0;
            self.sync_research_to_planet();
            return;
        };
        if !self.research_tree.can_select_on(
            &current_id,
            &self.research_state.unlocked,
            self.campaign.current().active_planet_condition(),
        ) {
            self.research_state.current_research = None;
            self.research_state.research_progress = 0.0;
            self.campaign
                .current_mut()
                .notifications
                .warning("Research branch unavailable on this world");
            self.sync_research_to_planet();
            return;
        }

        let remaining = (node.data_cost - self.research_state.research_progress).max(0.0);
        if remaining <= 0.0 {
            let name = node.name.clone();
            self.research_state.complete_research();
            self.announce_research(&name);
            self.sync_building_unlocks();
            self.sync_research_to_planet();
            return;
        }

        let available = self.campaign.current().resources.data;
        if available <= 0.0 {
            return;
        }

        let planet = self.campaign.current();
        let rate = planet.stats.apply(
            engine::StatId::ResearchRate,
            planet.config.resources.research_rate,
        );
        let spend = (rate * delta_time).min(available).min(remaining);
        self.campaign.current_mut().resources.data -= spend;
        self.research_state.research_progress += spend;

        if self.research_state.research_progress >= node.data_cost {
            let name = node.name.clone();
            self.research_state.complete_research();
            self.announce_research(&name);
        }

        self.sync_building_unlocks();
        self.sync_research_to_planet();
    }

    /// Finished research used to land with no more sign than a node changing
    /// colour on a screen the player was probably not looking at.
    fn announce_research(&mut self, name: &str) {
        let planet = self.campaign.current_mut();
        planet
            .notifications
            .success(format!("Research complete: {}", name));
        planet.emit_audio(state::AudioEvent::Research);
    }

    /// Take the campaign's research into the research screen's state.
    fn sync_research_from_planet(&mut self) {
        self.research_state.unlocked = self.campaign.research.unlocked_techs.clone();
        for tech in &data::game_data().research.starting_unlocked {
            if !self.research_state.unlocked.contains(tech) {
                self.research_state.unlocked.push(tech.clone());
            }
        }
        self.research_state.current_research = self.campaign.research.current_research.clone();
        self.research_state.research_progress = self.campaign.research.research_progress;
    }

    /// Write the research screen's state back to the campaign, and push it at
    /// every world the campaign holds.
    fn sync_research_to_planet(&mut self) {
        self.campaign.research.unlocked_techs = self.research_state.unlocked.clone();
        self.campaign.research.current_research = self.research_state.current_research.clone();
        self.campaign.research.research_progress = self.research_state.research_progress;
        self.campaign.sync_research();
    }

    /// Announce anything that has just become available here.
    ///
    /// The unlocking itself is the campaign's job now, across every world;
    /// this only notices what changed on the world the player is looking at,
    /// because that is who the toast is for.
    fn sync_building_unlocks(&mut self) {
        let before: Vec<engine::BuildingType> = data::game_data()
            .buildings
            .iter()
            .filter_map(|def| engine::BuildingType::from_id(&def.id))
            .filter(|kind| self.campaign.current().is_building_researched(*kind))
            .collect();

        self.campaign.sync_research();

        for def in &data::game_data().buildings {
            let Some(building_type) = engine::BuildingType::from_id(&def.id) else {
                continue;
            };
            if def.start_unlocked || before.contains(&building_type) {
                continue;
            }
            let planet = self.campaign.current_mut();
            // Only the moment it opens up is worth a toast, not every frame it
            // stays open, and not the ones this world refuses anyway.
            if planet.is_building_researched(building_type)
                && !planet.is_building_banned(building_type)
            {
                let name = def.name.clone();
                planet.notifications.info(format!("Available: {}", name));
            }
        }
    }
}

fn window_conf() -> Conf {
    capture::capture_window_conf("NANITE_SWARM", "Nanite Swarm", 1280, 720)
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    // Screenshot harness: when NANITE_SWARM_CAPTURE_PATH is set, seed a scene,
    // simulate deterministic frames, write a PNG, and exit. Each render_*
    // screen function clears its own background, so there is nothing extra
    // to move into the closure.
    if let Some(configs) = capture::CaptureConfig::all_from_env("NANITE_SWARM") {
        for config in configs {
            game.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |_dt| {
                game.update();
                macroquad_toolkit::ui::end_frame_neighbours();
            })
            .await;
        }
        return;
    }

    loop {
        game.update();
        macroquad_toolkit::ui::end_frame_neighbours();
        next_frame().await;
    }
}
