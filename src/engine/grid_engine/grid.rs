//! The game grid: construction, placement, and spatial queries

use crate::data::TerrainWeights;
use macroquad_toolkit::grid::bfs_path as toolkit_bfs_path;
use macroquad_toolkit::rng::SeededRng;
use serde::{Deserialize, Serialize};

use super::building::Building;
use super::building_type::BuildingType;
use super::grid_pos::GridPos;
use super::terrain::TerrainType;
use super::tile::Tile;

type TerrainRng = SeededRng;

use crate::data::OreConfig;
use crate::engine::terrain_gen;
use macroquad_toolkit::math::lerp;

/// The game grid containing all tiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    tiles: Vec<Tile>,
    /// Tiles power carries before it needs a repeater. Held here rather than
    /// read from a constant so research can widen it; kept in step by
    /// `PlanetState::refresh_stats`.
    #[serde(default = "Grid::default_repeater_range")]
    pub repeater_range: u32,
}

impl Grid {
    pub(super) fn default_repeater_range() -> u32 {
        6
    }

    /// Create a new grid with default empty tiles
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            tiles: vec![Tile::default(); size],
            repeater_range: Self::default_repeater_range(),
        }
    }

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
            width,
            height,
            tiles,
            repeater_range: Self::default_repeater_range(),
        };
        grid.open_a_landing_site(center, min_start_region);
        grid
    }

    /// Buildable ground reachable on foot from `from`, including it.
    ///
    /// Not the same question as whether a drone can get somewhere: this is
    /// about where the swarm can put anything at all.
    pub fn buildable_region(&self, from: GridPos) -> Vec<GridPos> {
        let mut seen = vec![false; self.tiles.len()];
        let mut region = Vec::new();
        let mut frontier = vec![from];
        if let Some(index) = self.index_of(from) {
            seen[index] = true;
        }
        while let Some(pos) = frontier.pop() {
            region.push(pos);
            for next in pos.neighbors() {
                let Some(index) = self.index_of(next) else {
                    continue;
                };
                if seen[index] {
                    continue;
                }
                if !self.tiles[index].terrain.is_buildable() {
                    continue;
                }
                seen[index] = true;
                frontier.push(next);
            }
        }
        region
    }

    fn index_of(&self, pos: GridPos) -> Option<usize> {
        pos.in_bounds(self.width, self.height)
            .then(|| (pos.y as u32 * self.width + pos.x as u32) as usize)
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
                    if !self.tiles[index].terrain.is_buildable() && !barriers.contains(&next) {
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

    /// Get tile at position (returns None if out of bounds)
    pub fn get(&self, pos: GridPos) -> Option<&Tile> {
        if pos.in_bounds(self.width, self.height) {
            Some(&self.tiles[pos.to_index(self.width)])
        } else {
            None
        }
    }

    /// Get mutable tile at position
    pub fn get_mut(&mut self, pos: GridPos) -> Option<&mut Tile> {
        if pos.in_bounds(self.width, self.height) {
            let index = pos.to_index(self.width);
            Some(&mut self.tiles[index])
        } else {
            None
        }
    }

    /// Check if a building can be placed at position
    pub fn can_place_building(&self, pos: GridPos, building_type: BuildingType) -> bool {
        if let Some(tile) = self.get(pos) {
            if !tile.revealed {
                return false;
            }
            if tile.filter {
                return false;
            }
            if tile.building.is_some() {
                return false;
            }
            // Conduits cannot overlap any existing building and must be on buildable terrain
            if building_type == BuildingType::Conduit {
                return tile.terrain.is_buildable();
            }
            // A Bridge is the network's answer to ground that will not hold
            // anything: it goes where nothing else can, and nowhere else.
            if building_type == BuildingType::Bridge {
                return matches!(tile.terrain, TerrainType::Water | TerrainType::Void);
            }
            if building_type == BuildingType::BiomassHarvester {
                return tile.terrain == TerrainType::Forest && !tile.filter;
            }
            // Special case: Wind turbines can go on mountains
            if building_type == BuildingType::WindTurbine {
                return matches!(tile.terrain, TerrainType::Empty | TerrainType::Mountain);
            }
            tile.terrain.is_buildable()
        } else {
            false
        }
    }

    /// Place a building at position
    pub fn place_building(&mut self, pos: GridPos, building_type: BuildingType) -> bool {
        if !self.can_place_building(pos, building_type) {
            return false;
        }

        if let Some(tile) = self.get_mut(pos) {
            let mut building = Building::new(building_type, pos);

            // Wind turbines on mountains get efficiency bonus
            if building_type == BuildingType::WindTurbine && tile.terrain == TerrainType::Mountain {
                building.efficiency = 2.0; // +100% bonus
            }

            tile.building = Some(building);
            true
        } else {
            false
        }
    }

    /// Remove a building at position
    pub fn remove_building(&mut self, pos: GridPos) -> Option<Building> {
        if let Some(tile) = self.get_mut(pos) {
            tile.building.take()
        } else {
            None
        }
    }

    /// Reveal tiles around a position
    pub fn reveal_around(&mut self, center: GridPos, radius: u32) {
        for tile_pos in
            macroquad_toolkit::grid::tiles_in_radius(center.to_tile_pos(), radius as i32)
        {
            let pos = GridPos::from_tile_pos(tile_pos);
            if pos.in_bounds(self.width, self.height) {
                if let Some(tile) = self.get_mut(pos) {
                    tile.revealed = true;
                }
            }
        }
    }

    /// Find the Core building position
    pub fn find_core(&self) -> Option<GridPos> {
        for (i, tile) in self.tiles.iter().enumerate() {
            if let Some(ref building) = tile.building {
                if building.building_type == BuildingType::Core {
                    return Some(GridPos::from_index(i, self.width));
                }
            }
        }
        None
    }

    /// Get all buildings of a specific type
    pub fn find_buildings(&self, building_type: BuildingType) -> Vec<GridPos> {
        self.tiles
            .iter()
            .enumerate()
            .filter_map(|(i, tile)| {
                tile.building
                    .as_ref()
                    .filter(|b| b.building_type == building_type)
                    .map(|_| GridPos::from_index(i, self.width))
            })
            .collect()
    }

    /// Iterator over all tiles with positions
    pub fn iter_tiles(&self) -> impl Iterator<Item = (GridPos, &Tile)> {
        self.tiles
            .iter()
            .enumerate()
            .map(move |(i, tile)| (GridPos::from_index(i, self.width), tile))
    }

    /// Iterator over all tiles with mutable access
    pub fn iter_tiles_mut(&mut self) -> impl Iterator<Item = (GridPos, &mut Tile)> {
        let width = self.width;
        self.tiles
            .iter_mut()
            .enumerate()
            .map(move |(i, tile)| (GridPos::from_index(i, width), tile))
    }

    pub fn initialize_forest_biomass(&mut self, amount: f32) {
        for (_, tile) in self.iter_tiles_mut() {
            if tile.terrain == TerrainType::Forest {
                tile.biomass_amount = amount;
            }
        }
    }

    /// Find a conduit path that avoids blocked tiles
    pub fn find_conduit_path(&self, from: GridPos, to: GridPos) -> Option<Vec<GridPos>> {
        if from == to {
            return Some(Vec::new());
        }

        let is_passable = |pos: GridPos, grid: &Grid| {
            if let Some(tile) = grid.get(pos) {
                if !tile.revealed {
                    return false;
                }
                if tile.filter {
                    return false;
                }
                // Void and water are crossable only where a Bridge already
                // stands; the run is planned through what is there, not
                // through what could be built there.
                let bridged = tile
                    .building
                    .as_ref()
                    .is_some_and(|building| building.building_type == BuildingType::Bridge);
                if !tile.terrain.is_buildable() && !bridged {
                    return false;
                }
                match tile.building.as_ref() {
                    None => true,
                    Some(building) => matches!(
                        building.building_type,
                        BuildingType::Conduit | BuildingType::Bridge
                    ),
                }
            } else {
                false
            }
        };

        toolkit_bfs_path(
            from.to_tile_pos(),
            to.to_tile_pos(),
            false,
            |pos| GridPos::from_tile_pos(pos).in_bounds(self.width, self.height),
            |pos| is_passable(GridPos::from_tile_pos(pos), self),
        )
        .map(|path| {
            path.into_iter()
                .skip(1)
                .map(GridPos::from_tile_pos)
                .collect()
        })
    }

    /// Count total buildings on the grid
    pub fn total_buildings(&self) -> usize {
        self.tiles
            .iter()
            .filter(|tile| tile.building.is_some())
            .count()
    }
}

#[cfg(test)]
mod tests;
