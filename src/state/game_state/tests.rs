use super::*;

#[test]
fn feature_overlap_keeps_narrow_rectangles_selectable() {
    let feature = PlanetFeature {
        id: "seam".to_string(),
        name: "Metal Seam".to_string(),
        description: "A narrow line of dense ore.".to_string(),
        bounds: (0.40, 0.40, 0.08, 0.08),
    };

    assert!(feature.contains_tile(GridPos::new(4, 4), 10, 10));
    assert!(!feature.contains_tile(GridPos::new(0, 0), 10, 10));
}

#[test]
fn feature_at_requires_a_revealed_tile() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    state.features = vec![PlanetFeature {
        id: "landing_crater".to_string(),
        name: "Landing Crater".to_string(),
        description: "The first impact.".to_string(),
        bounds: (0.45, 0.45, 0.15, 0.15),
    }];
    let pos = GridPos::new(11, 11);
    let tile = state
        .grid
        .get_mut(pos)
        .expect("test tile is in the Mars grid");
    tile.revealed = false;
    assert!(state.feature_at(pos).is_none());

    state.grid.get_mut(pos).unwrap().revealed = true;
    assert_eq!(
        state.feature_at(pos).map(|feature| feature.id.as_str()),
        Some("landing_crater")
    );
}

#[test]
fn iron_belt_exposes_its_ore_bonus_only_after_survey() {
    let mut state = PlanetState::new(0, 42, GameConfig::default());
    let feature = state.features.first().cloned().expect("Mercury feature");
    let pos = (0..state.grid.height)
        .flat_map(|y| (0..state.grid.width).map(move |x| GridPos::new(x as i32, y as i32)))
        .find(|pos| feature.contains_tile(*pos, state.grid.width, state.grid.height))
        .expect("feature overlaps the Mercury grid");

    if let Some(tile) = state.grid.get_mut(pos) {
        tile.revealed = false;
    }
    assert_eq!(state.feature_ore_multiplier_at(pos), None);

    state.grid.get_mut(pos).unwrap().revealed = true;
    assert_eq!(state.feature_ore_multiplier_at(pos), Some(1.5));
}

#[test]
fn acid_sea_exposes_its_corrosion_bonus_only_after_survey() {
    let mut state = PlanetState::new(1, 42, GameConfig::default());
    let feature = state.features.first().cloned().expect("Venus feature");
    let pos = (0..state.grid.height)
        .flat_map(|y| (0..state.grid.width).map(move |x| GridPos::new(x as i32, y as i32)))
        .find(|pos| feature.contains_tile(*pos, state.grid.width, state.grid.height))
        .expect("feature overlaps the Venus grid");

    state.grid.get_mut(pos).unwrap().revealed = false;
    assert_eq!(state.feature_acid_multiplier_at(pos), None);

    state.grid.get_mut(pos).unwrap().revealed = true;
    assert_eq!(state.feature_acid_multiplier_at(pos), Some(1.8));
}

#[test]
fn ring_shadow_exposes_its_cold_bonus_only_after_survey() {
    let mut state = PlanetState::new(4, 42, GameConfig::default());
    let feature = state.features.first().cloned().expect("Saturn feature");
    let pos = (0..state.grid.height)
        .flat_map(|y| (0..state.grid.width).map(move |x| GridPos::new(x as i32, y as i32)))
        .find(|pos| feature.contains_tile(*pos, state.grid.width, state.grid.height))
        .expect("feature overlaps the Saturn grid");

    state.grid.get_mut(pos).unwrap().revealed = false;
    assert_eq!(state.feature_freeze_multiplier_at(pos), None);

    state.grid.get_mut(pos).unwrap().revealed = true;
    assert_eq!(state.feature_freeze_multiplier_at(pos), Some(1.4));
}

#[test]
fn ice_rift_exposes_its_cooling_bonus_only_after_survey() {
    let mut state = PlanetState::new(3, 42, GameConfig::default());
    let feature = state.features.first().cloned().expect("Jupiter feature");
    let pos = (0..state.grid.height)
        .flat_map(|y| (0..state.grid.width).map(move |x| GridPos::new(x as i32, y as i32)))
        .find(|pos| feature.contains_tile(*pos, state.grid.width, state.grid.height))
        .expect("feature overlaps the Jupiter grid");

    state.grid.get_mut(pos).unwrap().revealed = false;
    assert_eq!(state.feature_cooling_multiplier_at(pos), None);

    state.grid.get_mut(pos).unwrap().revealed = true;
    assert_eq!(state.feature_cooling_multiplier_at(pos), Some(1.5));
}

#[test]
fn iron_belt_generation_keeps_a_drillable_tile_inside_the_feature() {
    for seed in [0, 1, 7, 42, 999] {
        let state = PlanetState::new(0, seed, GameConfig::default());
        let feature = state.features.first().expect("Mercury feature");

        assert!(
            state.grid.iter_tiles().any(|(pos, tile)| {
                tile.building.is_none()
                    && feature.contains_tile(pos, state.grid.width, state.grid.height)
                    && tile.terrain.is_buildable(state.data())
            }),
            "seed {seed} has no drillable Iron Belt tile"
        );
    }
}
