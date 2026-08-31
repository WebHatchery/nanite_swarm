//! Named planetary landmarks and their local simulation effects.

use super::PlanetState;
use crate::data::PlanetDef;
use crate::engine::{Grid, GridPos};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetFeature {
    pub id: String,
    pub name: String,
    pub description: String,
    pub bounds: (f32, f32, f32, f32),
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FeatureEffects {
    pub(crate) ore_multiplier: Option<f32>,
    pub(crate) acid_multiplier: Option<f32>,
    pub(crate) freeze_multiplier: Option<f32>,
    pub(crate) cooling_multiplier: Option<f32>,
    pub(crate) harvest_multiplier: Option<f32>,
}

fn valid_multiplier(multiplier: Option<f32>) -> Option<f32> {
    multiplier.filter(|value| value.is_finite() && *value > 1.0)
}

impl PlanetFeature {
    /// Whether a grid tile touches this feature's normalized map rectangle.
    ///
    /// Features are authored in the same normalized space as their map
    /// overlay, so selection remains stable when worlds have different
    /// dimensions. Testing tile overlap rather than only its center keeps a
    /// narrow feature selectable even when it falls between tile centers.
    pub fn contains_tile(&self, pos: GridPos, grid_width: u32, grid_height: u32) -> bool {
        if pos.x < 0 || pos.y < 0 || grid_width == 0 || grid_height == 0 {
            return false;
        }
        let (x, y, width, height) = self.bounds;
        let tile_left = pos.x as f32 / grid_width as f32;
        let tile_top = pos.y as f32 / grid_height as f32;
        let tile_right = (pos.x + 1) as f32 / grid_width as f32;
        let tile_bottom = (pos.y + 1) as f32 / grid_height as f32;
        tile_left < x + width && tile_right > x && tile_top < y + height && tile_bottom > y
    }
}

impl PlanetState {
    /// Return the named feature under a revealed tile, if there is one.
    /// Unrevealed tiles deliberately return nothing so the feature overlay
    /// cannot turn the inspector into a fog-of-war leak.
    pub fn feature_at(&self, pos: GridPos) -> Option<&PlanetFeature> {
        self.grid
            .get(pos)
            .filter(|tile| tile.revealed)
            .and_then(|_| {
                self.features
                    .iter()
                    .find(|feature| feature.contains_tile(pos, self.grid.width, self.grid.height))
            })
    }

    pub(crate) fn feature_effects_at(&self, pos: GridPos) -> Option<FeatureEffects> {
        let feature = self.feature_at(pos)?;
        let definition = self
            .data()
            .planet(self.planet_index)
            .features
            .iter()
            .find(|definition| definition.id == feature.id);
        Some(
            definition.map_or_else(FeatureEffects::default, |definition| FeatureEffects {
                ore_multiplier: valid_multiplier(definition.ore_multiplier),
                acid_multiplier: valid_multiplier(definition.acid_multiplier),
                freeze_multiplier: valid_multiplier(definition.freeze_multiplier),
                cooling_multiplier: valid_multiplier(definition.cooling_multiplier),
                harvest_multiplier: valid_multiplier(definition.harvest_multiplier),
            }),
        )
    }

    /// The authored ore bonus for a revealed feature tile, if it has one.
    ///
    /// The saved feature keeps only its stable identity and generated bounds;
    /// looking the multiplier up in the current data keeps older saves small
    /// and lets balance changes remain data-driven.
    pub fn feature_ore_multiplier_at(&self, pos: GridPos) -> Option<f32> {
        self.feature_effects_at(pos)?.ore_multiplier
    }

    /// The authored acid exposure for a revealed feature tile, if it has one.
    pub fn feature_acid_multiplier_at(&self, pos: GridPos) -> Option<f32> {
        self.feature_effects_at(pos)?.acid_multiplier
    }

    /// The authored cold exposure for a revealed feature tile, if it has one.
    pub fn feature_freeze_multiplier_at(&self, pos: GridPos) -> Option<f32> {
        self.feature_effects_at(pos)?.freeze_multiplier
    }

    /// The authored water-cooling multiplier for a revealed feature tile, if
    /// it has one.
    pub fn feature_cooling_multiplier_at(&self, pos: GridPos) -> Option<f32> {
        self.feature_effects_at(pos)?.cooling_multiplier
    }

    /// The authored harvest multiplier for a revealed feature tile, if it has
    /// one.
    pub fn feature_harvest_multiplier_at(&self, pos: GridPos) -> Option<f32> {
        self.feature_effects_at(pos)?.harvest_multiplier
    }
}

pub(super) fn generated_features(def: &PlanetDef, seed: u64) -> Vec<PlanetFeature> {
    def.features
        .iter()
        .map(|feature| {
            let x = macroquad_toolkit::noise::seeded_value(seed ^ feature.seed_offset, 3, 7, 5.0);
            let y = macroquad_toolkit::noise::seeded_value(seed ^ feature.seed_offset, 11, 13, 5.0);
            PlanetFeature {
                id: feature.id.clone(),
                name: feature.name.clone(),
                description: feature.description.clone(),
                bounds: (
                    (x * (1.0 - feature.width)).clamp(0.0, 1.0 - feature.width),
                    (y * (1.0 - feature.height)).clamp(0.0, 1.0 - feature.height),
                    feature.width,
                    feature.height,
                ),
            }
        })
        .collect()
}

/// Keep a mechanical feature over at least one tile where its effect works.
///
/// Procedural terrain is allowed to make most of a feature unusable, but a
/// small authored bonus should never be generated into a region where its
/// target action cannot be performed. The feature remains deterministic: only
/// its position moves to the nearest compatible tile when needed.
pub(super) fn anchor_effect_features(features: &mut [PlanetFeature], def: &PlanetDef, grid: &Grid) {
    for feature in features {
        let Some(definition) = def
            .features
            .iter()
            .find(|definition| definition.id == feature.id)
        else {
            continue;
        };
        let has_effect = [
            definition.ore_multiplier,
            definition.acid_multiplier,
            definition.freeze_multiplier,
            definition.cooling_multiplier,
            definition.harvest_multiplier,
        ]
        .into_iter()
        .flatten()
        .any(|multiplier| multiplier > 1.0);
        if !has_effect {
            continue;
        }
        let harvest_effect = definition
            .harvest_multiplier
            .is_some_and(|multiplier| multiplier > 1.0);
        let has_target_tile = grid.iter_tiles().any(|(pos, tile)| {
            tile.building.is_none()
                && feature.contains_tile(pos, grid.width, grid.height)
                && feature_target_is_usable(pos, tile, definition, harvest_effect, grid)
        });
        if has_target_tile {
            continue;
        }

        let (x, y, width, height) = (
            feature.bounds.0,
            feature.bounds.1,
            feature.bounds.2,
            feature.bounds.3,
        );
        let center = (x + width * 0.5, y + height * 0.5);
        let nearest = grid
            .iter_tiles()
            .filter(|(pos, tile)| {
                tile.building.is_none()
                    && feature_target_is_usable(*pos, tile, definition, harvest_effect, grid)
            })
            .min_by(|(left, _), (right, _)| {
                let left_distance = normalized_tile_distance(*left, center, grid);
                let right_distance = normalized_tile_distance(*right, center, grid);
                left_distance.total_cmp(&right_distance)
            });
        let Some((pos, _)) = nearest else {
            continue;
        };
        feature.bounds = (
            ((pos.x as f32 + 0.5) / grid.width as f32 - width * 0.5).clamp(0.0, 1.0 - width),
            ((pos.y as f32 + 0.5) / grid.height as f32 - height * 0.5).clamp(0.0, 1.0 - height),
            width,
            height,
        );
    }
}

fn feature_target_is_usable(
    pos: GridPos,
    tile: &crate::engine::Tile,
    definition: &crate::data::PlanetFeatureDef,
    harvest_effect: bool,
    grid: &Grid,
) -> bool {
    if harvest_effect {
        return tile.terrain.is_harvestable(grid.data());
    }
    if !tile.terrain.is_buildable(grid.data()) {
        return false;
    }
    let cooling_effect = definition
        .cooling_multiplier
        .is_some_and(|multiplier| multiplier > 1.0);
    !cooling_effect
        || pos.neighbors().iter().any(|neighbor| {
            grid.get(*neighbor)
                .is_some_and(|adjacent| adjacent.terrain == crate::engine::TerrainType::Water)
        })
}

fn normalized_tile_distance(pos: GridPos, center: (f32, f32), grid: &Grid) -> f32 {
    let tile_x = (pos.x as f32 + 0.5) / grid.width as f32;
    let tile_y = (pos.y as f32 + 0.5) / grid.height as f32;
    let dx = tile_x - center.0;
    let dy = tile_y - center.1;
    dx * dx + dy * dy
}

/// Resolve a local hazard multiplier without requiring a mutable world borrow.
/// Simulation loops use this while they are already iterating over the grid or
/// drone list, so the feature and definition snapshots stay explicit.
pub(crate) fn feature_hazard_multiplier(
    features: &[PlanetFeature],
    definitions: &[crate::data::PlanetFeatureDef],
    pos: GridPos,
    grid_width: u32,
    grid_height: u32,
    hazard: &str,
) -> f32 {
    let Some(feature) = features
        .iter()
        .find(|feature| feature.contains_tile(pos, grid_width, grid_height))
    else {
        return 1.0;
    };
    let multiplier = definitions
        .iter()
        .find(|definition| definition.id == feature.id)
        .and_then(|definition| match hazard {
            "acid" => definition.acid_multiplier,
            "cold" => definition.freeze_multiplier,
            _ => None,
        })
        .unwrap_or(1.0);
    if multiplier.is_finite() {
        multiplier.max(1.0)
    } else {
        1.0
    }
}

/// Resolve a local water-cooling multiplier without requiring a mutable world
/// borrow while the heat loop owns the grid iterator.
pub(crate) fn feature_cooling_multiplier(
    features: &[PlanetFeature],
    definitions: &[crate::data::PlanetFeatureDef],
    pos: GridPos,
    grid_width: u32,
    grid_height: u32,
) -> f32 {
    let Some(feature) = features
        .iter()
        .find(|feature| feature.contains_tile(pos, grid_width, grid_height))
    else {
        return 1.0;
    };
    let multiplier = definitions
        .iter()
        .find(|definition| definition.id == feature.id)
        .and_then(|definition| definition.cooling_multiplier)
        .unwrap_or(1.0);
    if multiplier.is_finite() {
        multiplier.max(1.0)
    } else {
        1.0
    }
}
