//! System-wide completion details shared by the ending layouts.

use crate::state::{Campaign, ConsumptionTotals, PLANET_COUNT};

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

pub(super) fn completion_consumption(campaign: &Campaign) -> ConsumptionTotals {
    (0..PLANET_COUNT)
        .filter_map(|index| campaign.planet(index))
        .map(|planet| planet.resources.consumed)
        .fold(ConsumptionTotals::default(), |mut total, consumed| {
            total.add(consumed);
            total
        })
}

pub(super) fn consumption_entries(campaign: &Campaign) -> [(&'static str, f32); 6] {
    let totals = completion_consumption(campaign);
    [
        ("MINERALS", totals.minerals),
        ("POWER", totals.energy),
        ("DATA", totals.data),
        ("BIOMASS", totals.biomass),
        ("ALLOY", totals.alloy),
        ("COMPONENTS", totals.components),
    ]
}

pub(super) fn format_consumption(amount: f32) -> String {
    if amount >= 1_000.0 {
        format!("{:.1}k", amount / 1_000.0)
    } else if amount >= 100.0 {
        format!("{amount:.0}")
    } else if amount >= 10.0 {
        format!("{amount:.1}")
    } else {
        format!("{amount:.2}")
    }
}

pub(super) fn compact_consumption_line(campaign: &Campaign) -> String {
    consumption_entries(campaign)
        .iter()
        .map(|(label, amount)| {
            format!(
                "{} {}",
                label.chars().next().unwrap_or('?'),
                format_consumption(*amount)
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

#[cfg(test)]
mod tests;
