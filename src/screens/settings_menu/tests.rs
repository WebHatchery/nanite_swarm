use super::*;

#[test]
fn volume_stepper_uses_ten_percent_increments_and_clamps() {
    let mut volume = 0.3;
    volume = step_volume(volume, 1);
    assert_eq!(volume, 0.4);

    volume = step_volume(volume, -1);
    assert_eq!(volume, 0.3);

    volume = step_volume(volume, -8);
    assert_eq!(volume, 0.0);
    volume = step_volume(volume, 20);
    assert_eq!(volume, 1.0);
}
