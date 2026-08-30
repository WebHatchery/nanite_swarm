use super::*;

#[test]
fn undo_without_history_explains_what_is_missing() {
    let mut state = state();

    assert!(!state.undo_last_action());
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message.contains("No action to undo")));
}

#[test]
fn saving_without_a_building_explains_the_required_selection() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let anchor = GridPos::new(core.x + 1, core.y);

    assert!(!state.save_blueprint(anchor, &[anchor]));
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| {
            notification
                .message
                .contains("Select at least one building")
        }));
}

#[test]
fn relocation_without_a_structure_explains_the_required_selection() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let empty = GridPos::new(core.x + 1, core.y);

    assert!(!state.begin_relocation(empty));
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message.contains("Select a structure")));
}

#[test]
fn the_core_cannot_be_armed_for_relocation() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();

    assert!(!state.begin_relocation(core));
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message.contains("Core cannot")));
}
