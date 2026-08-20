//! State management
//!
//! This module handles game state and persistence.

#![allow(unused)]

mod achievements;
mod actions;
mod audio;
mod auto_clock;
mod camera;
mod campaign;
mod core_stage;
mod factory;
mod game_state;
mod launch;
mod logistics;
mod migrations;
mod milestone;
mod particles;
#[cfg(test)]
mod performance;
mod persistence;
mod progress;
mod seed_ship;
mod shipping;
mod simulation;
mod stat_sheet;
mod tutorial;

pub use achievements::*;
pub use audio::*;
pub use camera::*;
pub use campaign::*;
pub use core_stage::*;
pub use factory::*;
pub use game_state::*;
pub use launch::*;
pub use milestone::*;
pub use persistence::*;
pub use seed_ship::*;
pub use shipping::*;
pub use simulation::{TICK_SECONDS, TIME_SCALES};
pub use stat_sheet::*;
pub use tutorial::TutorialGoal;
