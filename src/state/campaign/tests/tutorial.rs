use super::*;

#[test]
fn a_new_world_does_not_restart_the_opening_tutorial() {
    let mut campaign = campaign();
    assert!(!campaign.current().tutorial_done);

    for index in [0, 1, 3, 4] {
        assert!(campaign.colonize(index), "world {index} should be new");
        let world = campaign.planet(index).expect("new world should exist");
        assert!(world.tutorial_done, "world {index} restarted the lesson");
        assert!(world.tutorial_current().is_none());
        assert!(world.tutorial_highlight().is_none());
    }
}
