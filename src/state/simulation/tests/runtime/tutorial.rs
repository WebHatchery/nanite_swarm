//! Opening tutorial progression coverage.

use super::state;
use crate::engine::{BuildingType, GridPos};

#[test]
fn tutorial_advances_when_first_drill_is_placed() {
    let mut state = state();
    assert_eq!(state.tutorial_step, 0);
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    state.update_tutorial();
    assert_eq!(state.tutorial_step, 1);
}
