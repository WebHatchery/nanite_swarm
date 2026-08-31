use super::*;

#[test]
fn try_harvest_terrain_converts_mountain_and_grants_minerals() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 5, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Mountain;
    state.grid.reveal_around(pos, 1);
    let before = state.resources.minerals;

    assert!(state.can_harvest(pos));
    assert!(state.try_harvest_terrain(pos));
    assert!(state.resources.minerals > before);
    let tile = state.grid.get(pos).unwrap();
    assert_eq!(tile.terrain, TerrainType::Rough);
    assert!(tile.mountain_harvested);
    assert!(!state.can_harvest(pos));
}

#[test]
fn research_makes_the_same_mountain_worth_more() {
    let harvest = |techs: &[&str]| {
        let mut state = state();
        for tech in techs {
            state.research.unlocked_techs.push((*tech).to_string());
        }
        state.refresh_stats();
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 5, core.y);
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Mountain;
        state.grid.reveal_around(pos, 1);
        let before = state.resources.minerals;
        assert!(state.try_harvest_terrain(pos));
        state.resources.minerals - before
    };

    assert!(
        harvest(&["excavation_charges"]) > harvest(&[]),
        "the charges paid for nothing"
    );
}

#[test]
fn landing_crater_makes_a_revealed_mountain_worth_more() {
    let harvest = |in_crater| {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 5, core.y);
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Mountain;
        state.grid.get_mut(pos).unwrap().revealed = true;
        state.features[0].bounds = if in_crater {
            (0.65, 0.40, 0.30, 0.30)
        } else {
            (0.0, 0.0, 0.05, 0.05)
        };
        let before = state.resources.minerals;
        assert!(state.try_harvest_terrain(pos));
        state.resources.minerals - before
    };

    let ordinary = harvest(false);
    let crater = harvest(true);
    assert_eq!(ordinary, 50.0);
    assert_eq!(crater, 62.5);
}

#[test]
fn try_harvest_terrain_fails_on_unrevealed_tile() {
    let mut state = state();
    let far_pos = GridPos::new(0, 0);
    state.grid.get_mut(far_pos).unwrap().terrain = TerrainType::Mountain;
    assert!(!state.can_harvest(far_pos));
    assert!(!state.try_harvest_terrain(far_pos));
}

#[test]
fn try_convert_forest_to_filter_requires_forest_terrain() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 5, core.y);
    state.grid.get_mut(pos).unwrap().terrain = TerrainType::Forest;
    state.grid.reveal_around(pos, 1);

    assert!(state.try_convert_forest_to_filter(pos));
    let tile = state.grid.get(pos).unwrap();
    assert!(tile.filter);
    assert!(tile.forest_cleared);
    assert_eq!(tile.terrain, TerrainType::Rough);
    assert!(state
        .notifications
        .get_notifications()
        .iter()
        .any(|notification| notification.message.contains("dust filter -40% nearby")));

    // Already converted: no longer forest, so a second attempt fails.
    assert!(!state.try_convert_forest_to_filter(pos));
}
