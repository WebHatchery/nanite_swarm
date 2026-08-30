use super::*;

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
