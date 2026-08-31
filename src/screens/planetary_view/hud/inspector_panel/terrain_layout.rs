//! Text and vertical layout helpers for terrain inspection.

pub(super) fn feature_label(
    name: &str,
    ore_multiplier: Option<f32>,
    acid_multiplier: Option<f32>,
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
