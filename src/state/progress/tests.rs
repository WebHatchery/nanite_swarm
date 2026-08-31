use super::PlanetState;
use crate::data::GameConfig;
use crate::engine::{BuildingType, GridPos};

mod capacity;
mod effects;
mod guidance;
mod interaction;
mod offline;
mod timing;

fn state() -> PlanetState {
    PlanetState::new(2, 7, GameConfig::default())
}

fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() <= eps
}
