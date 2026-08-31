//! Text and vertical layout helpers for terrain inspection.

use crate::engine::{BuildingType, TerrainType};
use crate::screens::planetary_view::format::fit_text_to_width;
use crate::state::{FeatureEffects, PlanetFeature};
use macroquad::prelude::Color;
use macroquad_toolkit::ui::draw_ui_text;

pub(super) fn draw_local_effect_labels(
    show_icon: bool,
    row_base: f32,
    header_y: f32,
    labels: &[Option<&str>],
    x: f32,
    width: f32,
    color: Color,
) {
    if !show_icon || row_base <= header_y + 70.0 {
        return;
    }
    let mut y = header_y + 58.0;
    for label in labels.iter().flatten() {
        draw_ui_text(&fit_text_to_width(label, width, 9.0), x, y, 9.0, color);
        y += 12.0;
    }
}

pub(super) fn terrain_effect_label(
    terrain: Option<TerrainType>,
    building_type: BuildingType,
    terrain_bonus: Option<&str>,
) -> Option<String> {
    let supported = matches!(
        (terrain, building_type),
        (Some(TerrainType::Mountain), BuildingType::WindTurbine)
            | (Some(TerrainType::Forest), BuildingType::BiomassHarvester)
    );
    if supported {
        let terrain_name = match terrain {
            Some(TerrainType::Mountain) => "Mountain",
            Some(TerrainType::Forest) => "Forest",
            _ => return None,
        };
        Some(format!("Terrain: {terrain_name}  {}", terrain_bonus?))
    } else {
        None
    }
}

pub(super) fn forest_filter_label(filter: bool, multiplier: f32) -> Option<String> {
    filter.then(|| {
        let reduction = (1.0 - multiplier.clamp(0.0, 1.0)) * 100.0;
        format!("Dust filter: -{reduction:.0}% dust nearby")
    })
}

pub(super) fn feature_label_for(
    feature: Option<&PlanetFeature>,
    effects: Option<FeatureEffects>,
) -> Option<String> {
    let effects = effects.unwrap_or_default();
    feature.map(|feature| {
        feature_label(
            &feature.name,
            effects.ore_multiplier,
            effects.acid_multiplier,
            effects.freeze_multiplier,
            effects.cooling_multiplier,
            effects.harvest_multiplier,
        )
    })
}

pub(super) fn feature_survey_label(label: &str, progress: Option<(usize, usize)>) -> String {
    match progress {
        Some((revealed, total)) if revealed < total => {
            format!("{label}  // SURVEY {revealed}/{total}")
        }
        _ => label.to_string(),
    }
}

pub(super) fn feature_label(
    name: &str,
    ore_multiplier: Option<f32>,
    acid_multiplier: Option<f32>,
    freeze_multiplier: Option<f32>,
    cooling_multiplier: Option<f32>,
    harvest_multiplier: Option<f32>,
) -> String {
    let mut effects = Vec::new();
    if let Some(multiplier) = ore_multiplier {
        let bonus = ((multiplier - 1.0) * 100.0).round() as i32;
        effects.push(format!("+{bonus}% ORE"));
    }
    if let Some(multiplier) = acid_multiplier {
        let bonus = ((multiplier - 1.0) * 100.0).round() as i32;
        effects.push(format!("+{bonus}% ACID"));
    }
    if let Some(multiplier) = freeze_multiplier {
        let bonus = ((multiplier - 1.0) * 100.0).round() as i32;
        effects.push(format!("+{bonus}% COLD"));
    }
    if let Some(multiplier) = cooling_multiplier {
        let bonus = ((multiplier - 1.0) * 100.0).round() as i32;
        effects.push(format!("+{bonus}% COOLING"));
    }
    if let Some(multiplier) = harvest_multiplier {
        let bonus = ((multiplier - 1.0) * 100.0).round() as i32;
        effects.push(format!("+{bonus}% HARVEST"));
    }
    if effects.is_empty() {
        format!("Feature: {name}")
    } else {
        format!("Feature: {name}  {}", effects.join(" / "))
    }
}

pub(super) fn terrain_feature_lines(inspector_h: f32, has_feature: bool) -> usize {
    if !has_feature || inspector_h < 164.0 {
        0
    } else if inspector_h >= 206.0 {
        2
    } else {
        1
    }
}

pub(super) fn terrain_detail_offsets(inspector_h: f32) -> (f32, f32, f32) {
    if inspector_h < 130.0 {
        (60.0, 76.0, 0.0)
    } else if inspector_h < 180.0 {
        // Forests have two buttons, so their preservation line must clear the
        // compact action row rather than hiding behind it.
        (70.0, 86.0, 100.0)
    } else {
        (82.0, 106.0, 126.0)
    }
}
