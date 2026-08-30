use super::{
    compact_jump_label, mobile_alert_button_label, mobile_secondary_rects, mobile_top_button_width,
    *,
};
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
    let rects = mobile_secondary_rects(360.0, 400.0);
    let core = rects[4];
    let help = rects[5];

    assert_eq!(core.x, 218.0);
    assert_eq!(core.w, 63.0);
    assert_eq!(help.x, 287.0);
    assert_eq!(help.w, 63.0);
    assert_eq!(core.y, help.y);
    assert!(help.x + help.w <= 360.0);
}

#[test]
fn narrow_mobile_command_bar_keeps_every_control_inside_the_viewport() {
    let rects = mobile_secondary_rects(240.0, 400.0);

    assert!(rects.iter().all(|rect| {
        rect.x >= 0.0 && rect.x + rect.w <= 240.0 && rect.y >= 400.0 && rect.y + rect.h <= 422.0
    }));
    assert!(rects
        .windows(2)
        .all(|pair| pair[0].x + pair[0].w <= pair[1].x));
    assert_eq!(mobile_top_button_width(240.0), 52.0);
    assert!(10.0 + mobile_top_button_width(240.0) * 4.0 + 4.0 * 3.0 <= 240.0);
}

#[test]
fn extra_narrow_mobile_command_bar_still_fits_after_scaling() {
    for screen_w in [180.0, 200.0] {
        let rects = mobile_secondary_rects(screen_w, 400.0);

        assert!(rects
            .iter()
            .all(|rect| rect.x >= 0.0 && rect.x + rect.w <= screen_w));
        assert!(rects
            .windows(2)
            .all(|pair| pair[0].x + pair[0].w <= pair[1].x));
    }
}

#[test]
fn narrow_mobile_alert_labels_keep_the_recovery_cause_visible() {
    assert_eq!(mobile_alert_button_label("JUMP DUST", 240.0), "DUST");
    assert_eq!(mobile_alert_button_label("JUMP ACID", 240.0), "ACID");
    assert_eq!(mobile_alert_button_label("JUMP MIXED", 240.0), "MIXED");
    assert_eq!(mobile_alert_button_label("JUMP DUST", 360.0), "JUMP DUST");
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
