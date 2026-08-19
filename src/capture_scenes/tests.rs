use super::*;

#[test]
fn required_capture_scenes_are_registered() {
    assert_eq!(
        REQUIRED_CAPTURE_SCENES,
        ["mainmenu", "research", "logistics"]
    );
}
