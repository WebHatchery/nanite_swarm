//! Neural network and tech progression

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Research node in the neural network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub data_cost: f32,
    pub prerequisites: Vec<String>,
    pub position: (f32, f32), // Visual position in neural network
    #[serde(default)]
    pub planet_condition: Option<String>,
}

impl ResearchNode {
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        cost: f32,
        prereqs: Vec<&str>,
        pos: (f32, f32),
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            data_cost: cost,
            prerequisites: prereqs.into_iter().map(|s| s.to_string()).collect(),
            position: pos,
            planet_condition: None,
        }
    }
}

/// The research tree containing all nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTree {
    pub nodes: Vec<ResearchNode>,
}

impl ResearchTree {
    pub fn from_data(data: &crate::data::GameData) -> Self {
        data.research_tree()
    }
    pub fn from_nodes(nodes: Vec<ResearchNode>) -> Self {
        Self { nodes }
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&ResearchNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Check if a node can be researched
    pub fn can_research(&self, id: &str, unlocked: &[String], available_data: f32) -> bool {
        if let Some(node) = self.get_node(id) {
            // Already unlocked?
            if unlocked.contains(&node.id) {
                return false;
            }
            // Prerequisites met?
            if !node.prerequisites.iter().all(|p| unlocked.contains(p)) {
                return false;
            }
            // Enough data?
            if available_data < node.data_cost {
                return false;
            }
            true
        } else {
            false
        }
    }

    /// The ordinary prerequisite/data check plus the active world's named
    /// condition. A global branch has no condition and remains available on
    /// every world.
    pub fn can_research_on(
        &self,
        id: &str,
        unlocked: &[String],
        available_data: f32,
        condition: Option<&str>,
    ) -> bool {
        let Some(node) = self.get_node(id) else {
            return false;
        };
        let condition_met = node
            .planet_condition
            .as_deref()
            .is_none_or(|required| condition.is_some_and(|active| active == required));
        condition_met && self.can_research(id, unlocked, available_data)
    }

    /// Check if a node can be selected for research (ignores current data)
    pub fn can_select(&self, id: &str, unlocked: &[String]) -> bool {
        if let Some(node) = self.get_node(id) {
            if unlocked.contains(&node.id) {
                return false;
            }
            if !node.prerequisites.iter().all(|p| unlocked.contains(p)) {
                return false;
            }
            true
        } else {
            false
        }
    }

    pub fn can_select_on(&self, id: &str, unlocked: &[String], condition: Option<&str>) -> bool {
        let Some(node) = self.get_node(id) else {
            return false;
        };
        node.planet_condition
            .as_deref()
            .is_none_or(|required| condition.is_some_and(|active| active == required))
            && self.can_select(id, unlocked)
    }

    /// Get all nodes that are available for research
    pub fn available_research(
        &self,
        unlocked: &[String],
        available_data: f32,
    ) -> Vec<&ResearchNode> {
        self.nodes
            .iter()
            .filter(|n| self.can_research(&n.id, unlocked, available_data))
            .collect()
    }

    /// Get connections for visualization (from -> to)
    pub fn get_connections(&self) -> Vec<(&ResearchNode, &ResearchNode)> {
        let mut connections = Vec::new();
        for node in &self.nodes {
            for prereq_id in &node.prerequisites {
                if let Some(prereq) = self.get_node(prereq_id) {
                    connections.push((prereq, node));
                }
            }
        }
        connections
    }
}

/// Research state for a player
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchState {
    pub unlocked: Vec<String>,
    pub current_research: Option<String>,
    pub research_progress: f32,
    /// Partial progress parked while another node is active. Research is
    /// still one job at a time, but changing the job is no longer destructive.
    #[serde(default)]
    pub parked_progress: HashMap<String, f32>,
}

impl ResearchState {
    pub fn from_data(data: &crate::data::GameData) -> Self {
        Self {
            unlocked: data.research.starting_unlocked.clone(),
            ..Self::default()
        }
    }

    /// Progress on a node, whether it is active or parked.
    pub fn progress_for(&self, tech_id: &str) -> f32 {
        if self.current_research.as_deref() == Some(tech_id) {
            self.research_progress
        } else {
            self.parked_progress.get(tech_id).copied().unwrap_or(0.0)
        }
    }

    /// Park the active node without throwing away its partial work.
    pub fn pause_research(&mut self) {
        let Some(tech_id) = self.current_research.take() else {
            self.research_progress = 0.0;
            return;
        };
        if self.research_progress > 0.0 {
            self.parked_progress.insert(tech_id, self.research_progress);
        } else {
            self.parked_progress.remove(&tech_id);
        }
        self.research_progress = 0.0;
    }

    /// Start researching a tech, restoring any work previously parked on it.
    pub fn start_research(
        &mut self,
        tech_id: &str,
        tree: &ResearchTree,
        _available_data: f32,
    ) -> bool {
        if !tree.can_select(tech_id, &self.unlocked) {
            return false;
        }
        // Tapping the glowing active node is inspection, not a reset button.
        if self.current_research.as_deref() == Some(tech_id) {
            return true;
        }
        if let Some(previous) = self.current_research.take() {
            if self.research_progress > 0.0 {
                self.parked_progress
                    .insert(previous, self.research_progress);
            } else {
                self.parked_progress.remove(&previous);
            }
        }
        self.current_research = Some(tech_id.to_string());
        self.research_progress = self.parked_progress.remove(tech_id).unwrap_or(0.0);
        true
    }

    /// Complete current research
    pub fn complete_research(&mut self) -> Option<String> {
        if let Some(tech) = self.current_research.take() {
            self.unlocked.push(tech.clone());
            self.research_progress = 0.0;
            self.parked_progress.remove(&tech);
            Some(tech)
        } else {
            None
        }
    }

    /// Check if a tech is unlocked
    pub fn is_unlocked(&self, tech_id: &str) -> bool {
        self.unlocked.contains(&tech_id.to_string())
    }
}

#[cfg(test)]
mod tests;
