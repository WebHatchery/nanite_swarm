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
