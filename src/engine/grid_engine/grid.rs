//! The game grid: construction, placement, and spatial queries

use crate::data::{GameData, GameDataContext};
use macroquad_toolkit::grid::bfs_path as toolkit_bfs_path;
use serde::{Deserialize, Serialize};

use super::building::Building;
use super::building_type::BuildingType;
use super::grid_pos::GridPos;
use super::terrain::TerrainType;
use super::tile::Tile;

mod generation;

/// The game grid containing all tiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    /// Immutable definitions for grid-level simulation. This is session state,
    /// not save data, and is reattached when a campaign is loaded.
    #[serde(skip, default)]
    pub(super) data: GameDataContext,
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
            data: GameDataContext::default(),
            width,
            height,
            tiles: vec![Tile::default(); size],
            repeater_range: Self::default_repeater_range(),
        }
    }

    pub fn data(&self) -> &GameData {
        self.data.data()
    }

    pub fn attach_data(&mut self, data: GameDataContext) {
        self.data = data.clone();
        for (_, tile) in self.iter_tiles_mut() {
            if let Some(building) = tile.building.as_mut() {
                building.attach_data(data.clone());
            }
        }
    }

    /// Buildable ground reachable on foot from `from`, including it.
    ///
    /// Not the same question as whether a drone can get somewhere: this is
    /// about where the swarm can put anything at all.
    pub fn buildable_region(&self, from: GridPos) -> Vec<GridPos> {
        let data = self.data();
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
                if !self.tiles[index].terrain.is_buildable(data) {
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
        let data = self.data();
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
                return tile.terrain.is_buildable(data);
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
            tile.terrain.is_buildable(data)
        } else {
            false
        }
    }

    /// Place a building at position
    pub fn place_building(&mut self, pos: GridPos, building_type: BuildingType) -> bool {
        if !self.can_place_building(pos, building_type) {
            return false;
        }
        let data = self.data.clone();

        if let Some(tile) = self.get_mut(pos) {
            let mut building = Building::new_with_data(data, building_type, pos);

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

    /// Find a conduit path that avoids blocked tiles.
    ///
    /// Existing buildings may be the route's endpoints. This lets a player
    /// drag from the Core to a producer without first placing a sacrificial
    /// conduit tile on either side.
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
                // stands, except for an existing building used as an endpoint;
                // the run is planned through what is there, not through what
                // could be built there.
                let bridged = tile
                    .building
                    .as_ref()
                    .is_some_and(|building| building.building_type == BuildingType::Bridge);
                let endpoint = pos == from || pos == to;
                let anchored_building = endpoint && tile.building.is_some();
                if !tile.terrain.is_buildable(grid.data()) && !bridged && !anchored_building {
                    return false;
                }
                match tile.building.as_ref() {
                    None => true,
                    Some(building) => endpoint || building.carries_traffic(),
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
