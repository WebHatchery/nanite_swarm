//! Game data definitions loaded from JSON.

#[cfg(target_arch = "wasm32")]
use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

use crate::data::load_json;
use crate::engine::{ResearchNode, ResearchTree};

#[derive(Debug, Clone, Deserialize)]
pub struct Cost {
    pub minerals: f32,
    pub energy: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildingDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: Cost,
    pub power_generation: f32,
    pub power_consumption: f32,
    pub hotkey: Option<String>,
    pub texture: String,
    #[serde(default)]
    pub icon: Option<String>,
    pub build_menu_order: i32,
    pub show_in_build_menu: bool,
    pub start_unlocked: bool,
    pub unlocked_by: Option<String>,
    pub transmits_power: bool,
    /// Whether drones may travel through this building's tile. The logistics
    /// network is the conduit spaghetti the player lays down, not open ground.
    #[serde(default)]
    pub carries_traffic: bool,
    pub generates_power: bool,
    pub consumes_power: bool,
    pub uses_efficiency: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HarvestRewards {
    pub minerals: f32,
    pub biomass: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TerrainDef {
    pub id: String,
    pub name: String,
    pub buildable: bool,
    pub harvestable: bool,
    pub harvest_rewards: HarvestRewards,
    pub harvested_to: String,
    pub preservation_bonus: Option<String>,
    pub texture: String,
    pub color: [f32; 4],
}

/// One declared effect of a research node. Interpreted by
/// [`crate::engine::parse_modifier`]; validated when the tree is loaded.
#[derive(Debug, Clone, Deserialize)]
pub struct ModifierDef {
    pub stat: String,
    pub op: String,
    pub value: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResearchNodeDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub data_cost: f32,
    pub prerequisites: Vec<String>,
    pub position: (f32, f32),
    #[serde(default)]
    pub modifiers: Vec<ModifierDef>,
}

impl ResearchNodeDef {
    pub fn to_node(&self) -> ResearchNode {
        ResearchNode {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            data_cost: self.data_cost,
            prerequisites: self.prerequisites.clone(),
            position: self.position,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResearchData {
    pub starting_unlocked: Vec<String>,
    pub nodes: Vec<ResearchNodeDef>,
}

/// What one Seed Ship stage swallows before it is done.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct SeedShipCost {
    pub minerals: f32,
    pub data: f32,
    pub biomass: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedShipStageDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: SeedShipCost,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedShipData {
    /// Ceiling on how fast the yard can swallow each resource, per second.
    pub intake_per_second: SeedShipCost,
    pub stages: Vec<SeedShipStageDef>,
}

/// Share of a world's tiles given over to each non-empty terrain. Whatever is
/// left over is open ground.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct TerrainWeights {
    pub mountain: f32,
    pub forest: f32,
    pub water: f32,
    pub void: f32,
}

/// What a world does to the machinery left standing on it. Each is a strength
/// from 0 (absent) upward; research counters scale them back down.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, serde::Serialize)]
pub struct PlanetHazards {
    /// Corrodes anything carrying the network, on top of ordinary dust.
    pub acid_rain: f32,
    /// Share of drone speed the cold takes away.
    pub freeze: f32,
}

impl PlanetHazards {
    /// Whether this world is hostile at all.
    pub fn any(&self) -> bool {
        self.acid_rain > 0.0 || self.freeze > 0.0
    }
}

/// One world in the solar campaign: how it generates, how it looks on the map,
/// and what it refuses to let the swarm build.
#[derive(Debug, Clone, Deserialize)]
pub struct PlanetDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub difficulty: String,
    pub width: u32,
    pub height: u32,
    pub orbit_radius: f32,
    pub size: f32,
    pub color: [f32; 4],
    pub terrain: TerrainWeights,
    #[serde(default)]
    pub hazards: PlanetHazards,
    #[serde(default)]
    pub banned_buildings: Vec<String>,
    pub arrival: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlanetDataFile {
    pub planets: Vec<PlanetDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildingDataFile {
    pub buildings: Vec<BuildingDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TerrainDataFile {
    pub terrain: Vec<TerrainDef>,
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub buildings: Vec<BuildingDef>,
    pub buildings_by_id: HashMap<String, BuildingDef>,
    pub terrain: Vec<TerrainDef>,
    pub terrain_by_id: HashMap<String, TerrainDef>,
    pub research: ResearchData,
    pub seed_ship: SeedShipData,
    pub planets: Vec<PlanetDef>,
}

impl GameData {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Self {
        let buildings_json = fs::read_to_string("assets/buildings.json")
            .unwrap_or_else(|_| include_str!("../../assets/buildings.json").to_string());
        let terrain_json = fs::read_to_string("assets/terrain.json")
            .unwrap_or_else(|_| include_str!("../../assets/terrain.json").to_string());
        let research_json = fs::read_to_string("assets/research.json")
            .unwrap_or_else(|_| include_str!("../../assets/research.json").to_string());
        let seed_ship_json = fs::read_to_string("assets/seed_ship.json")
            .unwrap_or_else(|_| include_str!("../../assets/seed_ship.json").to_string());
        let planets_json = fs::read_to_string("assets/planets.json")
            .unwrap_or_else(|_| include_str!("../../assets/planets.json").to_string());

        Self::from_json_strings(
            &buildings_json,
            &terrain_json,
            &research_json,
            &seed_ship_json,
            &planets_json,
        )
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn load_async() -> Self {
        let buildings_json = load_string("assets/buildings.json")
            .await
            .unwrap_or_else(|_| include_str!("../../assets/buildings.json").to_string());
        let terrain_json = load_string("assets/terrain.json")
            .await
            .unwrap_or_else(|_| include_str!("../../assets/terrain.json").to_string());
        let research_json = load_string("assets/research.json")
            .await
            .unwrap_or_else(|_| include_str!("../../assets/research.json").to_string());
        let seed_ship_json = load_string("assets/seed_ship.json")
            .await
            .unwrap_or_else(|_| include_str!("../../assets/seed_ship.json").to_string());
        let planets_json = load_string("assets/planets.json")
            .await
            .unwrap_or_else(|_| include_str!("../../assets/planets.json").to_string());

        Self::from_json_strings(
            &buildings_json,
            &terrain_json,
            &research_json,
            &seed_ship_json,
            &planets_json,
        )
    }

    fn from_json_strings(
        buildings_json: &str,
        terrain_json: &str,
        research_json: &str,
        seed_ship_json: &str,
        planets_json: &str,
    ) -> Self {
        let building_file: BuildingDataFile =
            load_json(buildings_json).unwrap_or_else(|_| BuildingDataFile { buildings: vec![] });
        let terrain_file: TerrainDataFile =
            load_json(terrain_json).unwrap_or_else(|_| TerrainDataFile { terrain: vec![] });
        let research: ResearchData = load_json(research_json).unwrap_or_else(|_| ResearchData {
            starting_unlocked: vec!["core".to_string(), "basic_mining".to_string()],
            nodes: vec![],
        });

        // A typo in a modifier would otherwise be a tech that silently does
        // nothing - the exact bug this system exists to stop.
        for node in &research.nodes {
            for modifier in &node.modifiers {
                if let Err(problem) = crate::engine::parse_modifier(modifier) {
                    panic!("research node \"{}\": {}", node.id, problem);
                }
            }
        }

        let seed_ship: SeedShipData = load_json(seed_ship_json).unwrap_or_else(|_| SeedShipData {
            intake_per_second: SeedShipCost::default(),
            stages: vec![],
        });

        let planet_file: PlanetDataFile =
            load_json(planets_json).unwrap_or_else(|_| PlanetDataFile { planets: vec![] });

        let mut buildings_by_id = HashMap::new();
        for def in &building_file.buildings {
            buildings_by_id.insert(def.id.clone(), def.clone());
        }

        let mut terrain_by_id = HashMap::new();
        for def in &terrain_file.terrain {
            terrain_by_id.insert(def.id.clone(), def.clone());
        }

        Self {
            buildings: building_file.buildings,
            buildings_by_id,
            terrain: terrain_file.terrain,
            terrain_by_id,
            research,
            seed_ship,
            planets: planet_file.planets,
        }
    }

    pub fn building(&self, id: &str) -> &BuildingDef {
        self.buildings_by_id
            .get(id)
            .unwrap_or_else(|| panic!("Missing building def for id: {}", id))
    }

    /// The world at this campaign slot.
    pub fn planet(&self, index: usize) -> &PlanetDef {
        self.planets
            .get(index)
            .unwrap_or_else(|| panic!("Missing planet def for index: {}", index))
    }

    pub fn terrain(&self, id: &str) -> &TerrainDef {
        self.terrain_by_id
            .get(id)
            .unwrap_or_else(|| panic!("Missing terrain def for id: {}", id))
    }

    pub fn research_tree(&self) -> ResearchTree {
        ResearchTree::from_nodes(
            self.research
                .nodes
                .iter()
                .map(|node| node.to_node())
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
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
        let seed_ship_json = r#"{"intake_per_second": {"minerals": 1.0, "data": 1.0, "biomass": 1.0}, "stages": []}"#;
        let research_json = r#"{"starting_unlocked": ["core"], "nodes": [
            {"id": "core", "name": "Core", "description": "d", "data_cost": 0.0, "prerequisites": [], "position": [0.0, 0.0]}
        ]}"#;
        GameData::from_json_strings(
            buildings_json,
            terrain_json,
            research_json,
            seed_ship_json,
            planets_json,
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
        let data =
            GameData::from_json_strings("not json", "not json", "not json", "not json", "not json");
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
}
