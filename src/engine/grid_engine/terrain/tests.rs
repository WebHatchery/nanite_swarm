use super::TerrainType;

#[test]
fn id_roundtrips_through_from_id() {
    let all = [
        TerrainType::Empty,
        TerrainType::Mountain,
        TerrainType::Forest,
        TerrainType::Water,
        TerrainType::Rough,
        TerrainType::Void,
    ];
    for terrain in all {
        assert_eq!(TerrainType::from_id(terrain.id()), Some(terrain));
    }
}

#[test]
fn from_id_rejects_unknown_strings() {
    assert_eq!(TerrainType::from_id("lava"), None);
}

#[test]
fn mountain_harvests_into_rough_with_mineral_reward() {
    let mountain = TerrainType::Mountain;
    assert!(mountain.is_harvestable());
    assert!(!mountain.is_buildable());
    assert_eq!(mountain.harvested(), TerrainType::Rough);
    let (minerals, biomass) = mountain.harvest_rewards();
    assert!(minerals > 0.0);
    assert_eq!(biomass, 0.0);
}

#[test]
fn forest_harvests_into_empty_with_biomass_reward() {
    let forest = TerrainType::Forest;
    assert_eq!(forest.harvested(), TerrainType::Empty);
    let (minerals, biomass) = forest.harvest_rewards();
    assert_eq!(minerals, 0.0);
    assert!(biomass > 0.0);
}

#[test]
fn empty_and_rough_are_buildable_but_not_harvestable() {
    assert!(TerrainType::Empty.is_buildable());
    assert!(!TerrainType::Empty.is_harvestable());
    assert!(TerrainType::Rough.is_buildable());
    assert!(!TerrainType::Rough.is_harvestable());
}

#[test]
fn water_and_void_are_unbuildable() {
    assert!(!TerrainType::Water.is_buildable());
    assert!(!TerrainType::Void.is_buildable());
}
