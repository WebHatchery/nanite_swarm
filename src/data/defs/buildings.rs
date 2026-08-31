//! Building and terrain records loaded from the data files.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Cost {
    pub minerals: f32,
    pub energy: f32,
}

/// What a processing building turns into what, per second, while powered.
///
/// Inputs and outputs are maps of resource id to amount, so a third input is a
/// line of JSON rather than another named field and another branch.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RecipeDef {
    #[serde(default)]
    pub inputs: HashMap<String, f32>,
    #[serde(default)]
    pub outputs: HashMap<String, f32>,
    /// The one input drones have to carry here rather than draw from the pool.
    /// One, because a building has one hopper; the rest come from the pool.
    #[serde(default)]
    pub carried: Option<String>,
    #[serde(default)]
    pub carried_inputs: Vec<String>,
}

impl RecipeDef {
    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty() && self.outputs.is_empty()
    }

    pub fn carried_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.carried_inputs.iter().map(String::as_str).collect();
        if let Some(id) = self.carried.as_deref() {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        ids
    }
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
    /// If present, this building can only stand on worlds with the matching
    /// broad condition, such as `acid` or `cold`.
    #[serde(default)]
    pub planet_condition: Option<String>,
    #[serde(default)]
    pub water_cooling: bool,
    /// Present on processing buildings; absent means the building makes
    /// nothing out of anything.
    #[serde(default)]
    pub recipe: RecipeDef,
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
