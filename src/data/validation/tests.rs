use super::*;
use crate::data::{PlanetDef, TerrainWeights};

fn sample_data() -> GameData {
    GameData::load()
}

#[test]
fn valid_definition_package_passes_schema_validation() {
    assert!(validate_game_data(&sample_data()).is_ok());
}

#[test]
fn duplicate_ids_are_reported_by_the_data_domain() {
    let mut data = sample_data();
    data.terrain.push(data.terrain[0].clone());

    let error = validate_game_data(&data).unwrap_err();

    assert_eq!(
        error,
        "assets/terrain.json contains duplicate identifier \"empty\""
    );
}

#[test]
fn missing_cross_file_references_keep_their_source_context() {
    let mut data = sample_data();
    data.research.starting_unlocked = vec!["missing_tech".to_string()];

    let error = validate_game_data(&data).unwrap_err();

    assert_eq!(
        error,
        "assets/research.json starting unlock references missing node \"missing_tech\""
    );
}

#[test]
fn invalid_planet_shape_is_rejected_before_runtime_use() {
    let mut data = sample_data();
    data.planets.push(PlanetDef {
        id: "broken".to_string(),
        name: "Broken".to_string(),
        description: String::new(),
        difficulty: String::new(),
        width: 0,
        height: 4,
        orbit_radius: 1.0,
        size: 1.0,
        color: [1.0; 4],
        terrain: TerrainWeights::default(),
        hazards: Default::default(),
        banned_buildings: vec![],
        arrival: String::new(),
        pending_pod_cap: 1,
        features: vec![],
        hazard_fields: vec![],
        constraints: Default::default(),
    });

    let error = validate_game_data(&data).unwrap_err();

    assert_eq!(
        error,
        "assets/planets.json planet \"broken\" has an empty map"
    );
}
