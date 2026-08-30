use super::{alert_button, ClockAction};

#[test]
fn alert_button_points_at_the_relevant_recovery_surface() {
    assert_eq!(alert_button(false, false, false), None);
    assert_eq!(
        alert_button(false, true, false),
        Some(("FLOW", ClockAction::ToggleFlow))
    );
    assert_eq!(
        alert_button(false, true, true),
        Some(("FLOW ON", ClockAction::ToggleFlow))
    );
    assert_eq!(
        alert_button(true, true, false),
        Some(("RETRY", ClockAction::RetrySave))
    );
}
