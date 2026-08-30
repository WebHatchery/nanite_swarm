use super::*;

#[test]
fn delete_label_makes_the_second_tap_explicit() {
    let player_strings = crate::data::player_strings();

    assert_eq!(
        delete_slot_label(player_strings, false),
        player_strings.delete_slot
    );
    assert_eq!(
        delete_slot_label(player_strings, true),
        "Delete Slot (tap again)"
    );
}

#[test]
fn save_is_only_available_for_an_active_campaign() {
    assert!(!show_save_button(false));
    assert!(show_save_button(true));
}

#[test]
fn slot_label_exposes_whether_the_selected_slot_is_populated() {
    assert_eq!(
        slot_button_label("slot_1", true),
        "Slot: slot_1 [SAVED] (tap to switch)"
    );
    assert_eq!(
        slot_button_label("slot_2", false),
        "Slot: slot_2 [EMPTY] (tap to switch)"
    );
}

#[test]
fn short_desktop_menu_keeps_populated_commands_inside_the_view() {
    let layout = desktop_menu_layout(400.0, 0.0, 8);
    let last_y = layout.panel_y + layout.button_top + 7.0 * layout.button_spacing;

    assert!(layout.panel_y + layout.panel_h <= 400.0);
    assert!(last_y + layout.button_h <= 400.0);
    assert!((layout.panel_h - 286.0).abs() < 0.001);
    assert!((layout.button_h - 24.0).abs() < 0.001);
    assert!((layout.button_spacing - 29.0).abs() < 0.001);
}

#[test]
fn roomy_desktop_menu_keeps_the_existing_geometry() {
    let layout = desktop_menu_layout(720.0, 0.0, 6);

    assert!((layout.panel_h - 306.0).abs() < 0.001);
    assert!((layout.panel_y - 207.0).abs() < 0.001);
    assert!((layout.button_h - 30.0).abs() < 0.001);
    assert!((layout.button_spacing - 38.0).abs() < 0.001);
    assert!((layout.button_top - 60.0).abs() < 0.001);
}
