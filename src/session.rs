//! The active campaign session and its persistence boundary.
//!
//! `Game` owns rendering and frame dispatch. This type owns the campaign that
//! survives those frames, its slot, and the load/save mechanics that restore
//! it. Keeping that work together makes a second independently loaded session
//! possible without giving the application coordinator another persistence
//! responsibility.

use std::io;

use crate::data::{GameConfig, GameDataContext};
use crate::state::{self, Campaign, InterplanetaryMobilePanel, LaunchSequence, LoadSource};
use macroquad_toolkit::ui::ScrollArea;

const SAVE_PATH: &str = "save.json";
const SLOT_NAMES: [&str; 3] = ["slot_1", "slot_2", "slot_3"];

pub struct GameSession {
    pub campaign: Campaign,
    pub data: GameDataContext,
    pub has_save: bool,
    /// Whether the in-memory campaign is a player-selected session that may
    /// safely be written to the active slot.
    pub has_active_campaign: bool,
    pub menu_notice: Option<String>,
    /// The launch being played out, if one is.
    pub launch: Option<LaunchSequence>,
    pub shipping_edit_world: usize,
    /// Scroll position for the interplanetary freight control panel.
    pub shipping_scroll: ScrollArea,
    /// Which section is open on the portrait solar-system screen.
    pub interplanetary_mobile_panel: InterplanetaryMobilePanel,
    pub active_slot: usize,
    /// The active slot must be explicitly confirmed before deletion.
    pub delete_confirmation_slot: Option<usize>,
}

impl GameSession {
    pub fn new(config: GameConfig, data: GameDataContext, seed: u64) -> Self {
        Self {
            campaign: Campaign::new_with_data(config, seed, data.clone()),
            data,
            has_save: state::save_exists(&Self::save_path(0)),
            has_active_campaign: false,
            menu_notice: None,
            launch: None,
            shipping_edit_world: state::STARTING_PLANET,
            shipping_scroll: ScrollArea::new(),
            interplanetary_mobile_panel: InterplanetaryMobilePanel::Worlds,
            active_slot: 0,
            delete_confirmation_slot: None,
        }
    }

    pub fn slot_name(index: usize) -> &'static str {
        SLOT_NAMES[index % SLOT_NAMES.len()]
    }

    pub fn save_path(index: usize) -> String {
        format!("{}_{}", SAVE_PATH, Self::slot_name(index))
    }

    pub fn active_save_path(&self) -> String {
        Self::save_path(self.active_slot)
    }

    pub fn new_campaign(&mut self, config: GameConfig, seed: u64) {
        self.campaign = Campaign::new_with_data(config, seed, self.data.clone());
        self.has_active_campaign = true;
        self.shipping_edit_world = state::STARTING_PLANET;
        self.shipping_scroll.set_offset(0.0);
        self.interplanetary_mobile_panel = InterplanetaryMobilePanel::Worlds;
        self.delete_confirmation_slot = None;
    }

    pub fn save_campaign(&mut self) -> Result<(), io::Error> {
        if !self.has_active_campaign {
            return Ok(());
        }
        self.campaign
            .set_slot_name(Self::slot_name(self.active_slot));
        let path = self.active_save_path();
        state::save_to_file(&mut self.campaign, &path)?;
        self.has_save = true;
        self.campaign.mark_saved();
        Ok(())
    }

    pub fn load_campaign(&mut self) -> Result<LoadSource, io::Error> {
        let (mut campaign, source) = state::load_from_file(&self.active_save_path())?;
        campaign.attach_data(self.data.clone());
        self.shipping_edit_world = campaign.current_index();
        self.shipping_scroll.set_offset(0.0);
        self.interplanetary_mobile_panel = InterplanetaryMobilePanel::Worlds;
        self.delete_confirmation_slot = None;
        self.campaign = campaign;
        self.has_save = true;
        self.has_active_campaign = true;
        Ok(source)
    }

    pub fn refresh_save_presence(&mut self) {
        self.has_save = state::save_exists(&self.active_save_path());
    }
}

#[cfg(test)]
mod tests;
