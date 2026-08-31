//! Game data definitions loaded from JSON.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::ResearchTree;

mod buildings;
mod contracts;
mod planets;
mod research;
mod seed_ship;

pub use buildings::*;
pub use contracts::*;
pub use planets::*;
pub use research::*;
pub use seed_ship::*;

#[derive(Debug, Clone)]
pub struct GameData {
    pub buildings: Vec<BuildingDef>,
    pub buildings_by_id: HashMap<String, BuildingDef>,
    pub terrain: Vec<TerrainDef>,
    pub terrain_by_id: HashMap<String, TerrainDef>,
    pub research: ResearchData,
    pub seed_ship: SeedShipData,
    pub planets: Vec<PlanetDef>,
    pub tutorial: Vec<TutorialStepDef>,
    pub directives: DirectiveDataFile,
    pub achievements: Vec<AchievementDef>,
    pub core_stages: Vec<CoreStageDef>,
}

impl GameData {
    pub fn load() -> Self {
        let buildings_json = macroquad_toolkit::data_loader::load_text_with_fallback_sync(
            "assets/buildings.json",
            &[PathBuf::from("assets/buildings.json")],
            macroquad_toolkit::include_json_str!("../../assets/buildings.json"),
        )
        .expect("buildings data source");
        let terrain_json = macroquad_toolkit::data_loader::load_text_with_fallback_sync(
            "assets/terrain.json",
            &[PathBuf::from("assets/terrain.json")],
            macroquad_toolkit::include_json_str!("../../assets/terrain.json"),
        )
        .expect("terrain data source");
        let research_json = macroquad_toolkit::data_loader::load_text_with_fallback_sync(
            "assets/research.json",
            &[PathBuf::from("assets/research.json")],
            macroquad_toolkit::include_json_str!("../../assets/research.json"),
        )
        .expect("research data source");
        let seed_ship_json = macroquad_toolkit::data_loader::load_text_with_fallback_sync(
            "assets/seed_ship.json",
            &[PathBuf::from("assets/seed_ship.json")],
            macroquad_toolkit::include_json_str!("../../assets/seed_ship.json"),
        )
        .expect("seed ship data source");
        let planets_json = macroquad_toolkit::data_loader::load_text_with_fallback_sync(
            "assets/planets.json",
            &[PathBuf::from("assets/planets.json")],
            macroquad_toolkit::include_json_str!("../../assets/planets.json"),
        )
        .expect("planet data source");
        let tutorial_json = macroquad_toolkit::data_loader::load_text_with_fallback_sync(
            "assets/tutorial.json",
            &[PathBuf::from("assets/tutorial.json")],
            macroquad_toolkit::include_json_str!("../../assets/tutorial.json"),
        )
        .expect("tutorial data source");
        let directives_json = macroquad_toolkit::data_loader::load_text_with_fallback_sync(
            "assets/directives.json",
            &[PathBuf::from("assets/directives.json")],
            macroquad_toolkit::include_json_str!("../../assets/directives.json"),
        )
        .expect("directive data source");
        let achievements_json = macroquad_toolkit::data_loader::load_text_with_fallback_sync(
            "assets/achievements.json",
            &[PathBuf::from("assets/achievements.json")],
            macroquad_toolkit::include_json_str!("../../assets/achievements.json"),
        )
        .expect("achievement data source");
        let core_stages_json = macroquad_toolkit::data_loader::load_text_with_fallback_sync(
            "assets/core_stages.json",
            &[PathBuf::from("assets/core_stages.json")],
            macroquad_toolkit::include_json_str!("../../assets/core_stages.json"),
        )
        .expect("core stage data source");

        Self::from_json_strings(
            &buildings_json,
            &terrain_json,
            &research_json,
            &seed_ship_json,
            &planets_json,
            &tutorial_json,
            &directives_json,
            &achievements_json,
            &core_stages_json,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_json_strings(
        buildings_json: &str,
        terrain_json: &str,
        research_json: &str,
        seed_ship_json: &str,
        planets_json: &str,
        tutorial_json: &str,
        directives_json: &str,
        achievements_json: &str,
        core_stages_json: &str,
    ) -> Self {
        let building_file: BuildingDataFile =
            macroquad_toolkit::data_loader::load_embedded_json(buildings_json)
                .unwrap_or_else(|_| BuildingDataFile { buildings: vec![] });
        let terrain_file: TerrainDataFile =
            macroquad_toolkit::data_loader::load_embedded_json(terrain_json)
                .unwrap_or_else(|_| TerrainDataFile { terrain: vec![] });
        let research: ResearchData = macroquad_toolkit::data_loader::load_embedded_json(
            research_json,
        )
        .unwrap_or_else(|_| ResearchData {
            starting_unlocked: vec!["core".to_string(), "basic_mining".to_string()],
            nodes: vec![],
            stats: vec![],
        });

        let seed_ship: SeedShipData = macroquad_toolkit::data_loader::load_embedded_json(
            seed_ship_json,
        )
        .unwrap_or_else(|_| SeedShipData {
            intake_per_second: SeedShipCost::default(),
            stages: vec![],
            launch: LaunchSequenceDef::default(),
        });
        let planet_file: PlanetDataFile =
            macroquad_toolkit::data_loader::load_embedded_json(planets_json)
                .unwrap_or_else(|_| PlanetDataFile { planets: vec![] });

        let tutorial_file: TutorialDataFile =
            macroquad_toolkit::data_loader::load_embedded_json(tutorial_json)
                .unwrap_or_else(|_| TutorialDataFile { steps: vec![] });
        let directives: DirectiveDataFile = macroquad_toolkit::data_loader::load_embedded_json(
            directives_json,
        )
        .unwrap_or_else(|_| DirectiveDataFile {
            rotation_seconds: 600.0,
            directives: vec![],
        });
        let achievement_file: AchievementDataFile =
            macroquad_toolkit::data_loader::load_embedded_json(achievements_json).unwrap_or_else(
                |_| AchievementDataFile {
                    achievements: vec![],
                },
            );
        let core_stage_file: CoreStageDataFile =
            macroquad_toolkit::data_loader::load_embedded_json(core_stages_json)
                .unwrap_or_else(|_| CoreStageDataFile { stages: vec![] });
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
            tutorial: tutorial_file.steps,
            directives,
            achievements: achievement_file.achievements,
            core_stages: core_stage_file.stages,
        }
    }

    pub fn building(&self, id: &str) -> &BuildingDef {
        self.buildings_by_id.get(id).unwrap_or_else(|| {
            panic!(
                "assets/buildings.json: missing building identifier \"{}\"",
                id
            )
        })
    }

    /// The world at this campaign slot.
    pub fn planet(&self, index: usize) -> &PlanetDef {
        self.planets
            .get(index)
            .unwrap_or_else(|| panic!("assets/planets.json: missing planet index {}", index))
    }

    pub fn terrain(&self, id: &str) -> &TerrainDef {
        self.terrain_by_id
            .get(id)
            .unwrap_or_else(|| panic!("assets/terrain.json: missing terrain identifier \"{}\"", id))
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
mod tests;
