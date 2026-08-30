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
