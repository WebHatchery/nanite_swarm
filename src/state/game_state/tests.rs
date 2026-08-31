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
