use super::*;

#[test]
fn required_capture_scenes_are_registered() {
    assert_eq!(
        REQUIRED_CAPTURE_SCENES,
        [
            "mainmenu",
            "research",
            "logistics",
            "assembly",
            "focus",
            "freight",
            "flow",
            "factory_deck",
            "mobile_factory_deck",
            "mobile_build",
            "mobile_info",
            "mobile_help",
            "mobile_launch",
            "mobile_arrival",
            "hibernation"
        ]
    );
}
