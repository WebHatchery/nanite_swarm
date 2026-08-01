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

    /// Share of neighbouring pairs that are the same kind of ground. Confetti
    /// scores about what independence predicts; regions score far higher.
    fn neighbour_agreement(grid: &Grid) -> f32 {
        let mut same = 0.0;
        let mut total = 0.0;
        for y in 0..grid.height as i32 {
            for x in 0..grid.width as i32 - 1 {
                let a = grid.get(GridPos::new(x, y)).unwrap().terrain;
                let b = grid.get(GridPos::new(x + 1, y)).unwrap().terrain;
                total += 1.0;
                if a == b {
                    same += 1.0;
                }
            }
        }
        same / total
    }

    #[test]
    fn every_shipped_world_gives_the_swarm_somewhere_to_build() {
        // Regions turned void into a chasm, which can ring the Core. Across
        // every world the game ships and a spread of seeds, the landing site
        // has to be worth landing on.
        let config = crate::data::GameConfig::default();
        for def in &crate::data::game_data().planets {
            for seed in 0..25u64 {
                let grid = Grid::generate(
                    def.width,
                    def.height,
                    seed,
                    &def.terrain,
                    &config.ore,
                    config.grid.min_start_region,
                );
                let center = GridPos::new(def.width as i32 / 2, def.height as i32 / 2);
                let region = grid.buildable_region(center);
                let share = region.len() as f32 / (def.width * def.height) as f32;
                assert!(
                    share >= config.grid.min_start_region - 0.001,
                    "{} on seed {} left only {:.0}% of the world reachable",
                    def.id,
                    seed,
                    share * 100.0
                );
            }
        }
    }

    #[test]
    fn a_world_that_is_already_open_is_left_exactly_as_it_was() {
        // Nothing but ground: the safety net must not touch a world that does
        // not need it.
        let open = weights(0.0, 0.0, 0.0, 0.0);
        let before = Grid::generate(20, 20, 3, &open, &OreConfig::default(), 0.0);
        let after = Grid::generate(20, 20, 3, &open, &OreConfig::default(), 0.9);
        for (a, b) in before.tiles.iter().zip(after.tiles.iter()) {
            assert_eq!(a.terrain, b.terrain);
        }
    }

    #[test]
    fn a_core_ringed_by_void_is_dug_out_rather_than_left_walled_in() {
        // A world that is nearly all chasm still has to be playable.
        let walled = weights(0.0, 0.0, 0.0, 0.85);
        let grid = Grid::generate(20, 20, 12, &walled, &OreConfig::default(), 0.3);
        let center = GridPos::new(10, 10);
        let region = grid.buildable_region(center);
        assert!(
            region.len() as f32 / 400.0 >= 0.3,
            "only {} tiles of 400 were reachable",
            region.len()
        );
    }

    #[test]
    fn a_world_comes_out_in_regions_rather_than_confetti() {
        let weights = weights(0.2, 0.2, 0.1, 0.1);
        let grid = Grid::new_with_terrain(30, 30, 99, &weights);

        // With independent rolls the chance two neighbours match is the sum of
        // the squared shares: about 0.25 here. Regions beat that comfortably.
        let agreement = neighbour_agreement(&grid);
        assert!(
            agreement > 0.6,
            "neighbours agreed {:.2} of the time — that is still confetti",
            agreement
        );
    }

    #[test]
    fn clustering_does_not_quietly_retune_the_weights() {
        let weights = weights(0.2, 0.15, 0.1, 0.05);
        let grid = Grid::new_with_terrain(30, 30, 7, &weights);

        let total = grid.tiles.len() as f32;
        for (terrain, declared) in [
            (TerrainType::Mountain, 0.2),
            (TerrainType::Forest, 0.15),
            (TerrainType::Water, 0.1),
            (TerrainType::Void, 0.05),
        ] {
            let share = grid
                .tiles
                .iter()
                .filter(|tile| tile.terrain == terrain)
                .count() as f32
                / total;
            assert!(
                (share - declared).abs() < 0.05,
                "{:?} covers {:.2} of the world against a declared {:.2}",
                terrain,
                share,
                declared
            );
        }
    }

    #[test]
    fn deposits_come_in_patches_worth_walking_to() {
        let grid = Grid::new_with_terrain(30, 30, 5, &weights(0.1, 0.1, 0.05, 0.05));
        // A rich tile should nearly always have a rich neighbour: a deposit
        // that is one tile wide is a lottery, not a decision.
        let mut rich = 0;
        let mut with_company = 0;
        for y in 0..30 {
            for x in 0..30 {
                let pos = GridPos::new(x, y);
                if grid.get(pos).unwrap().ore_richness <= 1.05 {
                    continue;
                }
                rich += 1;
                if pos
                    .neighbors()
                    .iter()
                    .filter_map(|next| grid.get(*next))
                    .any(|tile| tile.ore_richness > 1.05)
                {
                    with_company += 1;
                }
            }
        }
        assert!(rich > 20, "hardly any deposits at all: {}", rich);
        assert!(
            with_company as f32 / rich as f32 > 0.85,
            "only {} of {} rich tiles had a rich neighbour",
            with_company,
            rich
        );
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
