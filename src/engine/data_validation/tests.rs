use super::*;

fn sample_data() -> crate::data::GameData {
    crate::data::GameData::load()
}

#[test]
fn a_negative_build_cost_is_rejected_before_it_can_refund_resources() {
    let mut data = sample_data();
    data.buildings[1].cost.minerals = -1.0;

    let error = validate_game_data(&data).unwrap_err();

    assert_eq!(
        error,
        "assets/buildings.json building \"drill\" has invalid construction cost"
    );
}

#[test]
fn a_nonfinite_research_modifier_is_rejected_before_it_reaches_stats() {
    let mut data = sample_data();
    data.research.nodes[4].modifiers[0].value = f32::NAN;

    let error = validate_game_data(&data).unwrap_err();

    assert_eq!(
        error,
        "research node \"efficient_drills\": modifier value for \"drill_output\" must be finite"
    );
}

#[test]
fn a_negative_seed_ship_stage_cost_is_rejected_before_progression() {
    let mut data = sample_data();
    data.seed_ship.stages[0].cost.data = -1.0;

    let error = validate_game_data(&data).unwrap_err();

    assert_eq!(
        error,
        "assets/seed_ship.json stage \"cradle\" has an invalid cost"
    );
}
