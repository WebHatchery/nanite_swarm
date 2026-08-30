use super::{core_stage_ambient, CoreStageAmbient};

#[test]
fn core_ambient_appears_when_the_core_reaches_late_stages() {
    assert_eq!(core_stage_ambient(0), CoreStageAmbient::Surface);
    assert_eq!(core_stage_ambient(2), CoreStageAmbient::Surface);
    assert_eq!(core_stage_ambient(3), CoreStageAmbient::SpaceElevator);
    assert_eq!(core_stage_ambient(4), CoreStageAmbient::PlanetaryRing);
    assert_eq!(core_stage_ambient(99), CoreStageAmbient::PlanetaryRing);
}
