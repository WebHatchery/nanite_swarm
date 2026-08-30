use super::*;

#[test]
fn a_planet_package_with_the_wrong_world_count_is_rejected() {
    let mut data = crate::data::GameData::load();
    data.planets.pop();

    let error = validate_game_data(&data).unwrap_err();

    assert_eq!(error, "assets/planets.json has 4 planets; expected 5");
}
