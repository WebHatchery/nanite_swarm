use super::BuildingType;

fn data() -> crate::data::GameDataContext {
    crate::data::test_data()
}

const ALL: [BuildingType; 18] = [
    BuildingType::Core,
    BuildingType::Drill,
    BuildingType::Conduit,
    BuildingType::Bridge,
    BuildingType::PowerNode,
    BuildingType::WindTurbine,
    BuildingType::ServerBank,
    BuildingType::Sweeper,
    BuildingType::Storage,
    BuildingType::BiomassHarvester,
    BuildingType::Smelter,
    BuildingType::Assembler,
    BuildingType::HeaterNode,
    BuildingType::ShieldGenerator,
    BuildingType::MassDriver,
    BuildingType::LandingPad,
    BuildingType::GeothermalTap,
    BuildingType::FusionReactor,
];

#[test]
fn id_roundtrips_through_from_id() {
    for building in ALL {
        assert_eq!(BuildingType::from_id(building.id()), Some(building));
    }
}

#[test]
fn from_id_rejects_unknown_strings() {
    assert_eq!(BuildingType::from_id("nonexistent"), None);
}

#[test]
fn drill_has_hotkey_and_positive_cost() {
    let (minerals, energy) = BuildingType::Drill.cost(data().data());
    assert!(minerals > 0.0);
    assert!(energy > 0.0);
    assert_eq!(BuildingType::Drill.hotkey(data().data()), Some('1'));
}

#[test]
fn core_is_free_and_has_no_hotkey() {
    let (minerals, energy) = BuildingType::Core.cost(data().data());
    assert_eq!(minerals, 0.0);
    assert_eq!(energy, 0.0);
    assert_eq!(BuildingType::Core.hotkey(data().data()), None);
}

#[test]
fn power_delta_is_generation_minus_consumption() {
    assert!(BuildingType::WindTurbine.power_delta(data().data()) > 0.0);
    assert!(BuildingType::ServerBank.power_delta(data().data()) < 0.0);
    assert_eq!(BuildingType::Conduit.power_delta(data().data()), 0.0);
    assert!(BuildingType::GeothermalTap.power_delta(data().data()) > 0.0);
    assert!(BuildingType::FusionReactor.power_delta(data().data()) > 0.0);
}
