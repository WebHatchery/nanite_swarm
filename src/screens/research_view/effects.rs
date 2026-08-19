use crate::engine::describe_modifier;
use crate::ui::Colors;
use macroquad::prelude::Color;

/// What a node does, as lines for the hover panel.
pub(super) fn node_effects(node_id: &str) -> Vec<(String, Color)> {
    let data = crate::data::game_data();
    let mut lines = Vec::new();

    if let Some(def) = data.research.nodes.iter().find(|node| node.id == node_id) {
        for modifier in &def.modifiers {
            let Some(summary) = describe_modifier(modifier) else {
                continue;
            };
            let color = if summary.is_gain {
                Colors::SUCCESS
            } else {
                Colors::WARNING
            };
            lines.push((format!("{} {}", summary.label, summary.change), color));
        }
    }

    for building in &data.buildings {
        if building.unlocked_by.as_deref() == Some(node_id) {
            lines.push((format!("Unlocks {}", building.name), Colors::PRIMARY));
        }
    }

    if lines.is_empty() {
        lines.push(("No direct effect".to_string(), Colors::TEXT_DIM));
    }
    lines
}
