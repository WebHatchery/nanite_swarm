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
use macroquad_toolkit::math::lerp;

/// The game grid containing all tiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    tiles: Vec<Tile>,
}

impl Grid {
    pub(super) const POWER_REPEATER_RANGE: u32 = 6;

    /// Create a new grid with default empty tiles
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            tiles: vec![Tile::default(); size],
        }
    }

    /// Create a grid with procedural terrain from a world's terrain weights.
    ///
    /// Each weight is the share of tiles that terrain takes; whatever is left
    /// over is open ground, which is what makes one world a forest and another
    /// a field of holes.
    pub fn new_with_terrain(width: u32, height: u32, seed: u64, weights: &TerrainWeights) -> Self {
        Self::generate(width, height, seed, weights, &OreConfig::default())
    }

    /// The same, with the ore spread declared rather than assumed.
    pub fn generate(
        width: u32,
        height: u32,
        seed: u64,
        weights: &TerrainWeights,
        ore: &OreConfig,
    ) -> Self {
        let mut rng = TerrainRng::new(seed);
        let size = (width * height) as usize;
        let mut tiles = Vec::with_capacity(size);

        let center_x = width as i32 / 2;
        let center_y = height as i32 / 2;

        // Cumulative thresholds, so a roll lands in exactly one band.
        let mountain = weights.mountain.max(0.0);
        let forest = mountain + weights.forest.max(0.0);
        let water = forest + weights.water.max(0.0);
        let void = water + weights.void.max(0.0);

        for i in 0..size {
            let pos = GridPos::from_index(i, width);
            let dist_from_center = pos.distance(GridPos::new(center_x, center_y));

            // Terrain distribution based on distance from center
            let terrain = if dist_from_center <= 2 {
                TerrainType::Empty // Clear area around Core
            } else {
                let roll: f32 = rng.next_f32();
                if roll < mountain {
                    TerrainType::Mountain
                } else if roll < forest {
                    TerrainType::Forest
                } else if roll < water {
                    TerrainType::Water
                } else if roll < void {
                    TerrainType::Void
                } else {
                    TerrainType::Empty
                }
            };

            // Reveal tiles near center
            let revealed = dist_from_center <= 4;

            // Ore is rolled for every tile, from the same seeded stream as
            // the terrain, so a world's ground is as reproducible as its shape.
            let ore_roll = rng.next_f32();
            let ore_richness = if ore_roll < ore.rich_chance {
                lerp(ore.rich_min, ore.rich_max, rng.next_f32())
            } else if ore_roll < ore.rich_chance + ore.lean_chance {
                lerp(ore.lean_min, ore.lean_max, rng.next_f32())
            } else {
                1.0
            };

            tiles.push(Tile {
                terrain,
                building: None,
                revealed,
                filter: false,
                mountain_harvested: false,
                forest_cleared: false,
                biomass_amount: 0.0,
                ore_richness,
            });
        }

        Self {
            width,
            height,
            tiles,
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
mod tests {
    use super::*;

    #[test]
    fn new_grid_is_empty_and_unrevealed() {
        let grid = Grid::new(4, 4);
        assert_eq!(grid.width, 4);
        assert_eq!(grid.height, 4);
        let tile = grid.get(GridPos::new(0, 0)).unwrap();
        assert_eq!(tile.terrain, TerrainType::Empty);
        assert!(!tile.revealed);
        assert!(grid.get(GridPos::new(4, 0)).is_none());
    }

    fn weights(mountain: f32, forest: f32, water: f32, void: f32) -> TerrainWeights {
        TerrainWeights {
            mountain,
            forest,
            water,
            void,
        }
    }

    #[test]
    fn new_with_terrain_clears_and_reveals_around_center() {
        let grid = Grid::new_with_terrain(20, 20, 42, &weights(0.15, 0.15, 0.05, 0.05));
        let center = GridPos::new(10, 10);
        let tile = grid.get(center).unwrap();
        assert_eq!(tile.terrain, TerrainType::Empty);
        assert!(tile.revealed);
    }

    #[test]
    fn place_building_requires_revealed_buildable_empty_tile() {
        let mut grid = Grid::new(4, 4);
        let pos = GridPos::new(1, 1);
        // Not revealed yet.
        assert!(!grid.can_place_building(pos, BuildingType::Drill));
        grid.reveal_around(pos, 1);
        assert!(grid.can_place_building(pos, BuildingType::Drill));
        assert!(grid.place_building(pos, BuildingType::Drill));
        // Tile is now occupied.
        assert!(!grid.can_place_building(pos, BuildingType::Drill));
    }

    #[test]
    fn wind_turbine_allowed_on_mountain_with_efficiency_bonus() {
        let mut grid = Grid::new(4, 4);
        let pos = GridPos::new(1, 1);
        grid.get_mut(pos).unwrap().terrain = TerrainType::Mountain;
        grid.reveal_around(pos, 1);
        assert!(grid.can_place_building(pos, BuildingType::WindTurbine));
        assert!(!grid.can_place_building(pos, BuildingType::Drill));
        assert!(grid.place_building(pos, BuildingType::WindTurbine));
        let building = grid.get(pos).unwrap().building.as_ref().unwrap();
        assert_eq!(building.efficiency, 2.0);
    }

    #[test]
    fn a_bridge_goes_where_nothing_else_can_and_nowhere_else() {
        let mut grid = Grid::new(4, 4);
        let void = GridPos::new(1, 1);
        let water = GridPos::new(2, 1);
        let ground = GridPos::new(3, 1);
        grid.get_mut(void).unwrap().terrain = TerrainType::Void;
        grid.get_mut(water).unwrap().terrain = TerrainType::Water;
        grid.reveal_around(GridPos::new(2, 1), 4);

        assert!(grid.place_building(void, BuildingType::Bridge));
        assert!(grid.place_building(water, BuildingType::Bridge));
        // Open ground does not need bridging and will not take one.
        assert!(!grid.can_place_building(ground, BuildingType::Bridge));
        // And the tile is taken now, the same as any other building.
        assert!(!grid.can_place_building(void, BuildingType::Bridge));
        assert!(!grid.can_place_building(void, BuildingType::Conduit));
    }

    #[test]
    fn a_bridge_is_a_piece_of_the_network_rather_than_a_permission_slip() {
        let mut grid = Grid::new(4, 4);
        let pos = GridPos::new(1, 1);
        grid.get_mut(pos).unwrap().terrain = TerrainType::Void;
        grid.reveal_around(pos, 2);
        assert!(grid.place_building(pos, BuildingType::Bridge));

        let building = grid.get(pos).unwrap().building.as_ref().unwrap();
        assert_eq!(building.building_type, BuildingType::Bridge);
        assert!(building.transmits_power(), "a bridge carries no power");
        assert!(building.carries_traffic(), "no drone can cross a bridge");
    }

    #[test]
    fn tearing_down_a_bridge_leaves_the_gap_it_spanned() {
        let mut grid = Grid::new(4, 4);
        let pos = GridPos::new(1, 1);
        grid.get_mut(pos).unwrap().terrain = TerrainType::Void;
        grid.reveal_around(pos, 2);
        grid.place_building(pos, BuildingType::Bridge);
        assert!(grid.remove_building(pos).is_some());
        assert!(grid.get(pos).unwrap().building.is_none());
        assert!(!grid.can_place_building(pos, BuildingType::Conduit));
        assert!(grid.can_place_building(pos, BuildingType::Bridge));
    }

    #[test]
    fn find_core_locates_the_core_building() {
        let mut grid = Grid::new(4, 4);
        let pos = GridPos::new(2, 2);
        grid.reveal_around(pos, 1);
        assert!(grid.find_core().is_none());
        grid.place_building(pos, BuildingType::Core);
        assert_eq!(grid.find_core(), Some(pos));
    }

    #[test]
    fn find_conduit_path_avoids_unbuildable_terrain() {
        let mut grid = Grid::new(5, 1);
        grid.reveal_around(GridPos::new(2, 0), 5);
        grid.get_mut(GridPos::new(2, 0)).unwrap().terrain = TerrainType::Void;
        let path = grid.find_conduit_path(GridPos::new(0, 0), GridPos::new(4, 0));
        assert!(path.is_none());
    }

    #[test]
    fn find_conduit_path_returns_empty_for_same_position() {
        let grid = Grid::new(4, 4);
        let path = grid.find_conduit_path(GridPos::new(1, 1), GridPos::new(1, 1));
        assert_eq!(path, Some(Vec::new()));
    }

    #[test]
    fn total_buildings_counts_occupied_tiles() {
        let mut grid = Grid::new(4, 4);
        assert_eq!(grid.total_buildings(), 0);
        let pos = GridPos::new(1, 1);
        grid.reveal_around(pos, 1);
        grid.place_building(pos, BuildingType::Drill);
        assert_eq!(grid.total_buildings(), 1);
    }
}
