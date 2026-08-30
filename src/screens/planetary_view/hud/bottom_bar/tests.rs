use super::{alert_button, has_flow_alert, zoom_button_rects, ClockAction};
use crate::data::GameConfig;
use crate::state::{GraphMode, PlanetState};

#[test]
fn alert_button_points_at_the_relevant_recovery_surface() {
    assert_eq!(alert_button(false, false, false, false, false), None);
    assert_eq!(
        alert_button(false, false, true, false, false),
        Some(("FLOW", ClockAction::ToggleFlow))
    );
    assert_eq!(
        alert_button(false, false, true, true, false),
        Some(("FLOW ON", ClockAction::ToggleFlow))
    );
    assert_eq!(
        alert_button(true, false, true, false, true),
        Some(("RETRY", ClockAction::RetrySave))
    );
    assert_eq!(
        alert_button(false, true, false, false, true),
        Some(("SAVE NOW", ClockAction::RetrySave))
    );
    assert_eq!(
        alert_button(true, true, false, false, false),
        Some(("RETRY", ClockAction::RetrySave)),
        "a failed retry stays the most urgent recovery"
    );
    assert_eq!(
        alert_button(false, false, true, false, true),
        Some(("JUMP", ClockAction::FocusAlert))
    );
}

#[test]
fn depleted_battery_exposes_the_flow_recovery_surface() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    state.battery_seconds = 0.0;

    assert!(has_flow_alert(&state));
    assert_eq!(
        alert_button(false, false, true, false, false),
        Some(("FLOW", ClockAction::ToggleFlow))
    );
}

#[test]
fn zoom_buttons_fit_both_wide_and_compact_command_slots() {
    for (slot_x, slot_w, expected_width) in [(100.0, 64.0, 24.0), (100.0, 30.0, 14.0)] {
        let (zoom_out, zoom_in) = zoom_button_rects(slot_x, 200.0, slot_w);
        assert_eq!(zoom_out.w, expected_width);
        assert_eq!(zoom_in.w, expected_width);
        assert!(zoom_out.x >= slot_x);
        assert!(zoom_in.x + zoom_in.w <= slot_x + slot_w);
        assert!(zoom_out.x + zoom_out.w < zoom_in.x);
    }
}

#[test]
fn graph_mode_cycles_through_flow_power_and_data() {
    assert_eq!(GraphMode::default(), GraphMode::Flow);
    assert_eq!(GraphMode::Flow.next(), GraphMode::Power);
    assert_eq!(GraphMode::Power.next(), GraphMode::Data);
    assert_eq!(GraphMode::Data.next(), GraphMode::Flow);
    assert_eq!(GraphMode::Power.label(), "POWER");
}
