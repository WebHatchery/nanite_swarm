//! System-wide completion details shared by the ending layouts.

use crate::state::{Campaign, PLANET_COUNT};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CompletionWorldRow {
    pub(super) name: String,
    pub(super) stage_label: String,
    pub(super) structures: usize,
}

pub(super) fn completion_world_rows(campaign: &Campaign) -> Vec<CompletionWorldRow> {
    (0..PLANET_COUNT)
        .filter_map(|index| {
            let planet = campaign.planet(index)?;
            let definition = campaign.data().data().planet(index);
            let stage_count = planet.core_stages().len().max(1);
            Some(CompletionWorldRow {
                name: definition.name.clone(),
                stage_label: format!("C{}/{}", planet.core_stage_index() + 1, stage_count),
                structures: planet.grid.total_buildings(),
            })
        })
        .collect()
}

pub(super) fn compact_core_stage_line(rows: &[CompletionWorldRow]) -> String {
    rows.iter()
        .map(|row| {
            let short_name: String = row.name.chars().take(2).collect();
            format!("{short_name} {}", row.stage_label)
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

#[cfg(test)]
mod tests;
