//! Planet terrain, hazards, features, and construction constraints.

use serde::Deserialize;

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
    #[serde(default)]
    pub pending_pod_cap: usize,
    #[serde(default)]
    pub features: Vec<PlanetFeatureDef>,
    #[serde(default)]
    pub hazard_fields: Vec<HazardFieldDef>,
    #[serde(default)]
    pub constraints: PlanetConstraintsDef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlanetFeatureDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub seed_offset: u64,
    /// Multiplies ore cut by a drill placed on this feature's revealed tiles.
    #[serde(default)]
    pub ore_multiplier: Option<f32>,
    /// Multiplies local acid exposure for machinery on this feature.
    #[serde(default)]
    pub acid_multiplier: Option<f32>,
    /// Multiplies local cold exposure for drones crossing this feature.
    #[serde(default)]
    pub freeze_multiplier: Option<f32>,
    /// Multiplies water cooling for water-cooled machinery on this feature.
    #[serde(default)]
    pub cooling_multiplier: Option<f32>,
    /// Multiplies terrain harvest rewards on this feature.
    #[serde(default)]
    pub harvest_multiplier: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HazardFieldDef {
    pub id: String,
    pub name: String,
    pub hazard: String,
    pub center: [f32; 2],
    pub radius: f32,
    #[serde(default)]
    pub strength: f32,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PlanetConstraintsDef {
    #[serde(default)]
    pub required_buildings: Vec<String>,
    #[serde(default)]
    pub required_research: Vec<String>,
    #[serde(default)]
    pub minimum_power_generation: f32,
    #[serde(default)]
    pub minimum_power_balance: f32,
}
