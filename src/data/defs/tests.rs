use super::*;

fn sample_data() -> GameData {
    let buildings_json = r#"{"buildings": [
        {"id": "core", "name": "Core", "description": "d", "cost": {"minerals": 0.0, "energy": 0.0},
         "power_generation": 4.0, "power_consumption": 0.0, "hotkey": null, "texture": "t", "icon": null,
         "build_menu_order": 0, "show_in_build_menu": false, "start_unlocked": true, "unlocked_by": null,
         "transmits_power": true, "generates_power": true, "consumes_power": false, "uses_efficiency": false}
    ]}"#;
    let terrain_json = r#"{"terrain": [
        {"id": "empty", "name": "Ground", "buildable": true, "harvestable": false,
         "harvest_rewards": {"minerals": 0.0, "biomass": 0.0}, "harvested_to": "empty",
         "preservation_bonus": null, "texture": "t", "color": [0.1, 0.1, 0.1, 1.0]}
    ]}"#;
    let planets_json = r#"{"planets": []}"#;
    let tutorial_json = r#"{"steps": []}"#;
    let directives_json = r#"{"rotation_seconds": 600.0, "directives": []}"#;
    let achievements_json = r#"{"achievements": []}"#;
    let core_stages_json = r#"{"stages": []}"#;
    let seed_ship_json =
        r#"{"intake_per_second": {"minerals": 1.0, "data": 1.0, "biomass": 1.0}, "stages": []}"#;
    let research_json = r#"{"starting_unlocked": ["core"], "nodes": [
        {"id": "core", "name": "Core", "description": "d", "data_cost": 0.0, "prerequisites": [], "position": [0.0, 0.0]}
    ]}"#;
    GameData::from_json_strings(
        buildings_json,
        terrain_json,
        research_json,
        seed_ship_json,
        planets_json,
        tutorial_json,
        directives_json,
        achievements_json,
        core_stages_json,
    )
}

#[test]
fn from_json_strings_indexes_buildings_and_terrain_by_id() {
    let data = sample_data();
    assert_eq!(data.building("core").name, "Core");
    assert_eq!(data.terrain("empty").name, "Ground");
    assert_eq!(data.buildings.len(), 1);
    assert_eq!(data.terrain.len(), 1);
}

#[test]
fn from_json_strings_falls_back_to_empty_on_malformed_json() {
    let data = GameData::from_json_strings(
        "not json", "not json", "not json", "not json", "not json", "not json", "not json",
        "not json", "not json",
    );
    assert!(data.buildings.is_empty());
    assert!(data.terrain.is_empty());
    // Research falls back to a minimal starting set rather than an empty tree.
    assert_eq!(
        data.research.starting_unlocked,
        vec!["core", "basic_mining"]
    );
}

#[test]
#[should_panic(expected = "Missing building def")]
fn building_panics_for_unknown_id() {
    let data = sample_data();
    data.building("nonexistent");
}

#[test]
fn research_tree_converts_node_defs_into_tree_nodes() {
    let data = sample_data();
    let tree = data.research_tree();
    assert_eq!(tree.nodes.len(), 1);
    assert_eq!(tree.nodes[0].id, "core");
}
