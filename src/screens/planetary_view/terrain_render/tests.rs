use super::*;
use crate::data::GameConfig;

fn test_feature() -> PlanetFeature {
    PlanetFeature {
        id: "surveyed_seam".to_string(),
        name: "Surveyed Seam".to_string(),
        description: "A line worth finding.".to_string(),
        bounds: (0.40, 0.40, 0.20, 0.20),
    }
}

#[test]
fn feature_overlay_waits_until_every_feature_tile_is_surveyed() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let feature = test_feature();
    state.features = vec![feature.clone()];
    assert!(!feature_is_surveyed(&state, &feature));

    let positions: Vec<_> = state
        .grid
        .iter_tiles()
        .filter_map(|(pos, _)| {
            feature
                .contains_tile(pos, state.grid.width, state.grid.height)
                .then_some(pos)
        })
        .collect();
    for pos in positions {
        state.grid.get_mut(pos).unwrap().revealed = true;
    }
    assert!(feature_is_surveyed(&state, &feature));
}
