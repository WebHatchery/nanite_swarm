//! Research progress attached to a planetary state.

use crate::data::GameData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Research node status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProgress {
    pub unlocked_techs: Vec<String>,
    pub current_research: Option<String>,
    pub research_progress: f32,
    /// Partial work on nodes not currently selected for research.
    #[serde(default)]
    pub parked_progress: HashMap<String, f32>,
}

impl Default for ResearchProgress {
    fn default() -> Self {
        Self {
            unlocked_techs: Vec::new(),
            current_research: None,
            research_progress: 0.0,
            parked_progress: HashMap::new(),
        }
    }
}

impl ResearchProgress {
    pub fn from_data(data: &GameData) -> Self {
        Self {
            unlocked_techs: data.research.starting_unlocked.clone(),
            ..Self::default()
        }
    }
}
