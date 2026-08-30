use super::{compact_jump_label, mobile_command_aux_rects, *};
use crate::data::GameConfig;
use crate::state::PlanetState;

#[test]
fn mobile_jump_names_upkeep_when_it_is_the_active_recovery() {
    let mut state = PlanetState::new(1, 42, GameConfig::default());
    let core = state.grid.find_core().expect("generated Core");

    state
        .grid
        .get_mut(core)
        .expect("core tile")
        .building
        .as_mut()
        .expect("core building")
        .acid_wear = 75.0;
    assert_eq!(compact_jump_label(&state, false), "JUMP ACID");
    assert_eq!(compact_jump_label(&state, true), "JUMP");
}

#[test]
fn mobile_command_bar_keeps_core_and_help_touch_targets_inside_portrait_width() {
    let (core, help) = mobile_command_aux_rects(360.0, 400.0);

    assert_eq!(core.x, 218.0);
    assert_eq!(core.w, 63.0);
    assert_eq!(help.x, 287.0);
    assert_eq!(help.w, 63.0);
    assert_eq!(core.y, help.y);
    assert!(help.x + help.w <= 360.0);
}

#[test]
fn mobile_help_stops_above_the_command_bar_on_small_phones() {
    let height = mobile_help_height(480.0, 64.0, 90.0);

    assert_eq!(height, 316.0);
    assert!(90.0 + height <= 480.0 - 64.0 - 10.0);
}

#[test]
fn mobile_help_can_use_the_taller_phone_without_growing_forever() {
    let height = mobile_help_height(640.0, 64.0, 90.0);

    assert_eq!(height, 430.0);
    assert!(90.0 + height < 640.0 - 64.0);
}
