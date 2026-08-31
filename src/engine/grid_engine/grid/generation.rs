//! Deterministic terrain, ore, and landing-site generation for a grid.

use crate::data::{GameDataContext, OreConfig, TerrainWeights};
use crate::engine::terrain_gen;
use macroquad_toolkit::math::lerp;

use super::{Grid, GridPos, TerrainType, Tile};

impl Grid {
    /// Create a grid with procedural terrain from a world's terrain weights.
    ///
    /// Each weight is the share of tiles that terrain takes; whatever is left
    /// over is open ground, which is what makes one world a forest and another
    /// a field of holes.
    pub fn new_with_terrain(width: u32, height: u32, seed: u64, weights: &TerrainWeights) -> Self {
        Self::generate(width, height, seed, weights, &OreConfig::default(), 0.3)
    }

    /// The same, with the ore spread declared rather than assumed.
    pub fn generate(
        width: u32,
        height: u32,
        seed: u64,
        weights: &TerrainWeights,
        ore: &OreConfig,
        min_start_region: f32,
    ) -> Self {
        let size = (width * height) as usize;
        let center = GridPos::new(width as i32 / 2, height as i32 / 2);
        // The clear ground the Core lands on is not eligible for anything, so
        // it is left out of the shares as well as out of the fields.
        let shaped = |pos: GridPos| pos.distance(center) > 2;

        // One field per kind of ground, each with its own seed and its own
        // sense of scale, so a world's ridges and its forests are not the same
        // shape offset by a step.
        let bands: [(TerrainType, f32, u64, f32); 4] = [
            (
                TerrainType::Mountain,
                weights.mountain.max(0.0),
                0x1111,
                5.0,
            ),
            (TerrainType::Forest, weights.forest.max(0.0), 0x2222, 4.0),
            (TerrainType::Water, weights.water.max(0.0), 0x3333, 3.5),
            (TerrainType::Void, weights.void.max(0.0), 0x4444, 4.5),
        ];

        let positions: Vec<GridPos> = (0..size)
            .map(|i| GridPos::from_index(i, width))
            .filter(|pos| shaped(*pos))
            .collect();

        // Thresholds are quantiles of the field over this world, so each kind
        // of ground covers exactly the share it was given.
        let cuts: Vec<f32> = bands
            .iter()
            .map(|(_, share, band_seed, scale)| {
                let mut values: Vec<f32> = positions
                    .iter()
                    .map(|pos| terrain_gen::field(seed ^ band_seed, pos.x, pos.y, *scale))
                    .collect();
                terrain_gen::threshold_for_share(&mut values, *share)
            })
            .collect();

        let mut ore_values: Vec<f32> = positions
            .iter()
            .map(|pos| terrain_gen::field(seed ^ 0x5555, pos.x, pos.y, 3.0))
            .collect();
        let rich_cut = terrain_gen::threshold_for_share(&mut ore_values.clone(), ore.rich_chance);
        let lean_cut = 1.0
            - terrain_gen::threshold_for_share(
                &mut ore_values.iter().map(|v| 1.0 - v).collect::<Vec<f32>>(),
                ore.lean_chance,
            );

        let mut tiles = Vec::with_capacity(size);
        for i in 0..size {
            let pos = GridPos::from_index(i, width);

            let terrain = if !shaped(pos) {
                TerrainType::Empty
            } else {
                // First band whose field is over its cut wins, so overlapping
                // regions resolve the same way every time.
                bands
                    .iter()
                    .zip(cuts.iter())
                    .find(|((_, share, band_seed, scale), cut)| {
                        *share > 0.0
                            && terrain_gen::field(seed ^ band_seed, pos.x, pos.y, *scale) >= **cut
                    })
                    .map(|((terrain, _, _, _), _)| *terrain)
                    .unwrap_or(TerrainType::Empty)
            };

            // Ore rides its own field, so deposits come in patches worth
            // walking to rather than one lucky tile at a time.
            let ore_field = terrain_gen::field(seed ^ 0x5555, pos.x, pos.y, 3.0);
            let ore_richness = if !shaped(pos) {
                1.0
            } else if ore_field >= rich_cut {
                lerp(ore.rich_min, ore.rich_max, ore_field)
            } else if ore_field <= lean_cut {
                lerp(ore.lean_min, ore.lean_max, ore_field)
            } else {
                1.0
            };

            tiles.push(Tile {
                terrain,
                building: None,
                revealed: pos.distance(center) <= 4,
                filter: false,
                mountain_harvested: false,
                forest_cleared: false,
                biomass_amount: 0.0,
                ore_richness,
            });
        }

        let mut grid = Self {
            data: GameDataContext::default(),
            width,
            height,
            tiles,
            repeater_range: Self::default_repeater_range(),
        };
        grid.open_a_landing_site(center, min_start_region);
        grid
    }

    /// Clear a way out of the landing site until there is enough ground to
    /// build on.
    ///
    /// Regions made void a chasm rather than a scattering of holes, which is
    /// the point — but a chasm can also ring the Core, and a world the swarm
    /// cannot leave the first tile of is not a hard world, it is a broken one.
    /// Only the least is cleared: the barrier tile nearest the Core, one at a
    /// time, so the rest of the world keeps its shape.
    fn open_a_landing_site(&mut self, center: GridPos, min_share: f32) {
        let context = self.data.clone();
        let data = context.data();
        let target = (self.tiles.len() as f32 * min_share.clamp(0.0, 1.0)).round() as usize;
        for _ in 0..self.tiles.len() {
            let region = self.buildable_region(center);
            if region.len() >= target {
                return;
            }
            // Barriers touching the region, nearest the Core first, and by
            // index to break ties the same way every run.
            let mut barriers: Vec<GridPos> = Vec::new();
            for pos in &region {
                for next in pos.neighbors() {
                    let Some(index) = self.index_of(next) else {
                        continue;
                    };
                    if !self.tiles[index].terrain.is_buildable(data) && !barriers.contains(&next) {
                        barriers.push(next);
                    }
                }
            }
            barriers.sort_by_key(|pos| {
                (
                    pos.distance(center),
                    self.index_of(*pos).unwrap_or(usize::MAX),
                )
            });
            let Some(barrier) = barriers.first().copied() else {
                return;
            };
            if let Some(tile) = self.get_mut(barrier) {
                tile.terrain = TerrainType::Empty;
            }
        }
    }
}
