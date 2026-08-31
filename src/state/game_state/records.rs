//! Durable and session records attached to a planet.

use crate::engine::{BuildingType, GridPos};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseRecord {
    pub source: String,
    pub building: Option<BuildingType>,
    pub position: Option<GridPos>,
    pub world_time: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct GraphSample {
    pub power_produced: f32,
    pub power_consumed: f32,
    #[serde(default)]
    pub minerals_consumed: f32,
    pub alloy_produced: f32,
    pub alloy_consumed: f32,
    #[serde(default)]
    pub components_produced: f32,
    #[serde(default)]
    pub components_consumed: f32,
    pub data_produced: f32,
    pub data_consumed: f32,
}

/// Which telemetry family the compact bottom graph is showing.
///
/// This is session UI state rather than campaign state: a save reopens on the
/// familiar factory-flow view, while the player can tap through power and
/// Data whenever those signals matter more.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GraphMode {
    #[default]
    Flow,
    Power,
    Data,
}

impl GraphMode {
    pub fn next(self) -> Self {
        match self {
            Self::Flow => Self::Power,
            Self::Power => Self::Data,
            Self::Data => Self::Flow,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Flow => "FLOW",
            Self::Power => "POWER",
            Self::Data => "DATA",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlueprintEntry {
    pub offset: (i32, i32),
    pub building_type: BuildingType,
    #[serde(default)]
    pub overclocked: bool,
    #[serde(default)]
    pub input_priority: bool,
    #[serde(default)]
    pub standby: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum UndoEntry {
    Placed(GridPos),
    Removed(BuildingType, GridPos, bool, bool, bool),
    Relocated(GridPos, GridPos),
}
