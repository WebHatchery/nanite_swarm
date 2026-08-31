use crate::data::GameConfig;
use crate::state::PlanetState;

fn state() -> PlanetState {
    PlanetState::new(2, 42, GameConfig::default())
}

mod clock;
mod collapse;
mod tutorial;
mod upkeep;
