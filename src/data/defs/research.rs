//! Research records and their conversion into the engine tree.

use serde::Deserialize;

use crate::engine::ResearchNode;

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
    #[serde(default)]
    pub planet_condition: Option<String>,
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
            planet_condition: self.planet_condition.clone(),
        }
    }
}

/// What a stat is called where a player can see it, and which direction is
/// good news. Without this a modifier can only be shown as its raw id.
#[derive(Debug, Clone, Deserialize)]
pub struct StatLabelDef {
    pub id: String,
    pub label: String,
    /// True for stats a tech is trying to push *down* - dust, power draw, how
    /// much of a hazard still gets through.
    #[serde(default)]
    pub lower_is_better: bool,
    /// How the value reads on a stat sheet: `per_second`, `count`, `share`,
    /// or anything else for a plain number.
    #[serde(default)]
    pub unit: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResearchData {
    pub starting_unlocked: Vec<String>,
    pub nodes: Vec<ResearchNodeDef>,
    #[serde(default)]
    pub stats: Vec<StatLabelDef>,
}
