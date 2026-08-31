use super::*;

#[test]
fn the_default_balance_contract_is_valid() {
    assert!(GameConfig::default().validate().is_ok());
}

#[test]
fn a_negative_drill_rate_is_rejected_before_simulation() {
    let mut config = GameConfig::default();
    config.buildings.drill_output_rate = -1.0;

    let error = config.validate().unwrap_err();

    assert_eq!(
        error,
        "buildings.drill_output_rate must be finite and positive"
    );
}

#[test]
fn inverted_grid_bounds_are_rejected_before_generation() {
    let mut config = GameConfig::default();
    config.grid.max_width = config.grid.initial_width - 1;

    let error = config.validate().unwrap_err();

    assert_eq!(
        error,
        "grid maximum dimensions must include the initial grid"
    );
}

#[test]
fn a_missing_mass_driver_resource_is_rejected_before_shipping() {
    let mut config = GameConfig::default();
    config.mass_driver.cargo.clear();

    let error = config.validate().unwrap_err();

    assert_eq!(
        error,
        "mass_driver.cargo must contain at least one resource"
    );
}

#[test]
fn an_unknown_mass_driver_resource_is_rejected_before_shipping() {
    let mut config = GameConfig::default();
    config.mass_driver.cargo = vec!["crystals".to_string()];

    let error = config.validate().unwrap_err();

    assert_eq!(
        error,
        "mass_driver.cargo contains unknown resource \"crystals\""
    );
}

#[test]
fn a_nonphysical_mass_driver_resource_is_rejected_before_shipping() {
    let mut config = GameConfig::default();
    config.mass_driver.cargo = vec!["data".to_string()];

    let error = config.validate().unwrap_err();

    assert_eq!(error, "mass_driver.cargo resource \"data\" is not physical");
}

#[test]
fn a_nonfinite_collapse_loss_is_rejected_before_a_failure() {
    let mut config = GameConfig::default();
    config.collapse.max_data_loss = f32::NAN;

    let error = config.validate().unwrap_err();

    assert_eq!(
        error,
        "collapse.max_data_loss must be finite and between 0 and 1"
    );
}

#[test]
fn a_filter_multiplier_above_one_is_rejected_before_upkeep() {
    let mut config = GameConfig::default();
    config.upkeep.filter_multiplier = 1.1;

    let error = config.validate().unwrap_err();

    assert_eq!(
        error,
        "upkeep.filter_multiplier must be finite and between 0 and 1"
    );
}

#[test]
fn a_forest_buffer_multiplier_above_one_is_rejected_before_upkeep() {
    let mut config = GameConfig::default();
    config.upkeep.forest_buffer_multiplier = 1.1;

    let error = config.validate().unwrap_err();

    assert_eq!(
        error,
        "upkeep.forest_buffer_multiplier must be finite and between 0 and 1"
    );
}

#[test]
fn a_hazard_counter_strength_above_one_is_rejected_before_hazard_simulation() {
    let mut config = GameConfig::default();
    config.upkeep.hazard_counter_strength = 1.1;

    let error = config.validate().unwrap_err();

    assert_eq!(
        error,
        "upkeep.hazard_counter_strength must be finite and between 0 and 1"
    );
}
