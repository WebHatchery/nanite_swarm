//! The active campaign session and its persistence boundary.
//!
//! `Game` owns rendering and frame dispatch. This type owns the campaign that
//! survives those frames, its slot, and the load/save mechanics that restore
//! it. Keeping that work together makes a second independently loaded session
//! possible without giving the application coordinator another persistence
//! responsibility.

use std::io;

use crate::data::{GameConfig, GameDataContext};
use crate::state::{self, Campaign, LaunchSequence, LoadSource};

const SAVE_PATH: &str = "save.json";
const SLOT_NAMES: [&str; 3] = ["slot_1", "slot_2", "slot_3"];

pub struct GameSession {
    pub campaign: Campaign,
    pub data: GameDataContext,
    pub has_save: bool,
    pub menu_notice: Option<String>,
    /// The ending has been shown once; seeing it again is the player's choice.
    pub ending_seen: bool,
    /// The launch being played out, if one is.
    pub launch: Option<LaunchSequence>,
    pub shipping_edit_world: usize,
    pub active_slot: usize,
}

impl GameSession {
    pub fn new(config: GameConfig, data: GameDataContext, seed: u64) -> Self {
        Self {
            campaign: Campaign::new_with_data(config, seed, data.clone()),
            data,
            has_save: state::save_exists(&Self::save_path(0)),
            menu_notice: None,
            ending_seen: false,
            launch: None,
            shipping_edit_world: state::STARTING_PLANET,
            active_slot: 0,
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
        self.ending_seen = false;
    }

    pub fn save_campaign(&mut self) -> Result<(), io::Error> {
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
        self.campaign = campaign;
        self.has_save = true;
        Ok(source)
    }

    pub fn refresh_save_presence(&mut self) {
        self.has_save = state::save_exists(&self.active_save_path());
    }
}
