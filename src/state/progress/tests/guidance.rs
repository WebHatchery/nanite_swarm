use super::*;

#[test]
fn core_is_always_unlocked_others_require_explicit_unlock() {
    let mut state = PlanetState::default();
    assert!(state.is_building_unlocked(BuildingType::Core));
    assert!(!state.is_building_unlocked(BuildingType::Conduit));
    state.unlock_building(BuildingType::Conduit);
    assert!(state.is_building_unlocked(BuildingType::Conduit));
}

#[test]
fn hostile_arrival_guidance_walks_the_power_route_prerequisites() {
    let mut state = PlanetState::new(1, 7, GameConfig::default());
    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap RESEARCH for Power Grid.")
    );

    state.research.unlocked_techs.push("power_grid".to_string());
    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap RESEARCH for Geothermal Tapping.")
    );

    state
        .research
        .unlocked_techs
        .push("geothermal_tapping".to_string());
    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap BUILD, then place Geothermal Tap.")
    );
}

#[test]
fn acid_arrival_guidance_continues_to_the_corrosion_countermeasure() {
    let mut state = PlanetState::new(1, 7, GameConfig::default());
    for tech in ["power_grid", "geothermal_tapping"] {
        state.research.unlocked_techs.push(tech.to_string());
    }
    let core = state.grid.find_core().unwrap();
    let source = GridPos::new(core.x + 1, core.y);
    let tile = state.grid.get_mut(source).unwrap();
    tile.terrain = crate::engine::TerrainType::Empty;
    tile.revealed = true;
    assert!(state
        .grid
        .place_building(source, BuildingType::GeothermalTap));

    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap RESEARCH for Ceramic Plating.")
    );
}

#[test]
fn acid_arrival_guidance_names_the_shield_after_plating_is_researched() {
    let mut state = PlanetState::new(1, 7, GameConfig::default());
    for tech in ["power_grid", "geothermal_tapping", "ceramic_plating"] {
        state.research.unlocked_techs.push(tech.to_string());
    }
    let core = state.grid.find_core().unwrap();
    let geothermal = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(geothermal).unwrap().terrain = crate::engine::TerrainType::Empty;
    state
        .grid
        .place_building(geothermal, BuildingType::GeothermalTap);

    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap BUILD, then place Shield Generator.")
    );
}

#[test]
fn a_cold_world_keeps_heater_placement_visible_after_fusion_is_ready() {
    let mut state = PlanetState::new(3, 7, GameConfig::default());
    for tech in ["power_grid", "heater_nodes", "fusion_reactors"] {
        state.research.unlocked_techs.push(tech.to_string());
    }
    state.unlock_building(BuildingType::HeaterNode);
    state.unlock_building(BuildingType::FusionReactor);
    let core = state.grid.find_core().unwrap();
    let source = GridPos::new(core.x + 1, core.y);
    let tile = state.grid.get_mut(source).unwrap();
    tile.terrain = crate::engine::TerrainType::Empty;
    tile.revealed = true;
    assert!(state
        .grid
        .place_building(source, BuildingType::FusionReactor));

    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap BUILD, then place Heater Node.")
    );
}

#[test]
fn cold_arrival_guidance_starts_with_the_first_fusion_prerequisite() {
    let state = PlanetState::new(4, 7, GameConfig::default());
    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap RESEARCH for Power Grid.")
    );
}

#[test]
fn calm_worlds_without_a_construction_rule_need_no_arrival_guidance() {
    let state = PlanetState::new(2, 7, GameConfig::default());
    assert_eq!(state.arrival_guidance(), None);
}
