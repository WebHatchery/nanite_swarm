use super::{alert_button, ClockAction};

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
