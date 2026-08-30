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
