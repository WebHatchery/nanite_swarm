use super::*;
use crate::data::GameConfig;
use crate::engine::{BuildingType, GridPos, ResourceType};

fn campaign() -> Campaign {
    Campaign::new(GameConfig::default(), 7)
}

/// A save store that lives in memory, so the rotation rules can be tested
/// without going near a real save.
#[derive(Default)]
struct MapStore {
    entries: std::collections::HashMap<String, String>,
    fail_writes_to: Option<String>,
}

impl SaveStore for MapStore {
    fn read(&self, key: &str) -> Option<String> {
        self.entries.get(key).cloned()
    }

    fn write(&mut self, key: &str, content: &str) -> Result<(), String> {
        if self.fail_writes_to.as_deref() == Some(key) {
            return Err("disk is on fire".to_string());
        }
        self.entries.insert(key.to_string(), content.to_string());
        Ok(())
    }
}

#[test]
fn the_first_save_writes_no_backup_because_there_is_nothing_to_keep() {
    let mut store = MapStore::default();
    let mut campaign = campaign();
    save_campaign(&mut store, "save", &mut campaign).unwrap();

    assert!(store.read("save").is_some());
    assert!(store.read("save_backup").is_none());
}

#[test]
fn saving_keeps_the_previous_save_as_the_backup() {
    let mut store = MapStore::default();
    let mut first = campaign();
    first.current_mut().resources.biomass = 11.0;
    save_campaign(&mut store, "save", &mut first).unwrap();

    let mut second = campaign();
    second.current_mut().resources.biomass = 22.0;
    save_campaign(&mut store, "save", &mut second).unwrap();

    let (current, source) = load_campaign(&store, "save").unwrap();
    assert_eq!(source, LoadSource::Primary);
    assert_eq!(current.current().resources.biomass, 22.0);

    let backup = load_from_json(&store.read("save_backup").unwrap()).unwrap();
    assert_eq!(backup.current().resources.biomass, 11.0);
}

#[test]
fn save_slots_keep_their_worlds_isolated() {
    let mut store = MapStore::default();
    let mut first = campaign();
    first.current_mut().resources.biomass = 11.0;
    save_campaign(&mut store, "slot_1", &mut first).unwrap();

    let mut second = campaign();
    second.current_mut().resources.biomass = 22.0;
    save_campaign(&mut store, "slot_2", &mut second).unwrap();

    assert_eq!(
        load_campaign(&store, "slot_1")
            .unwrap()
            .0
            .current()
            .resources
            .biomass,
        11.0
    );
    assert_eq!(
        load_campaign(&store, "slot_2")
            .unwrap()
            .0
            .current()
            .resources
            .biomass,
        22.0
    );
}

#[test]
fn a_corrupt_save_is_recovered_from_the_backup() {
    let mut store = MapStore::default();
    let mut good = campaign();
    good.current_mut().resources.biomass = 33.0;
    save_campaign(&mut store, "save", &mut good).unwrap();
    // Second save rotates the good one into the backup...
    let mut newer = campaign();
    save_campaign(&mut store, "save", &mut newer).unwrap();
    // ...and then the main save goes bad.
    store.write("save", "{ this is not a save }").unwrap();

    let (recovered, source) = load_campaign(&store, "save").unwrap();
    assert_eq!(source, LoadSource::Backup);
    assert_eq!(recovered.current().resources.biomass, 33.0);
}

#[test]
fn a_missing_save_with_a_good_backup_still_loads() {
    let mut store = MapStore::default();
    let mut good = campaign();
    good.current_mut().resources.biomass = 44.0;
    save_campaign(&mut store, "save", &mut good).unwrap();
    let mut newer = campaign();
    save_campaign(&mut store, "save", &mut newer).unwrap();
    store.entries.remove("save");

    let (recovered, source) = load_campaign(&store, "save").unwrap();
    assert_eq!(source, LoadSource::Backup);
    assert_eq!(recovered.current().resources.biomass, 44.0);
}

#[test]
fn two_bad_copies_is_an_error_rather_than_a_silent_new_game() {
    let mut store = MapStore::default();
    store.write("save", "rubbish").unwrap();
    store.write("save_backup", "also rubbish").unwrap();
    assert!(load_campaign(&store, "save").is_err());
}

#[test]
fn a_failed_rotation_does_not_quietly_overwrite_the_good_save() {
    let mut store = MapStore::default();
    let mut first = campaign();
    first.current_mut().resources.biomass = 55.0;
    save_campaign(&mut store, "save", &mut first).unwrap();

    store.fail_writes_to = Some("save_backup".to_string());
    let mut second = campaign();
    second.current_mut().resources.biomass = 66.0;
    assert!(save_campaign(&mut store, "save", &mut second).is_err());

    // The old save is still there and still readable.
    let (kept, _) = load_campaign(&store, "save").unwrap();
    assert_eq!(kept.current().resources.biomass, 55.0);
}

#[test]
fn json_roundtrip_preserves_grid_and_resources() {
    let mut campaign = campaign();
    let core = campaign.current().grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    campaign.current_mut().grid.reveal_around(pos, 1);
    campaign.current_mut().select_building(BuildingType::Drill);
    campaign.current_mut().try_place_building(pos);

    let json = save_to_json(&mut campaign).unwrap();
    let loaded = load_from_json(&json).unwrap();

    assert_eq!(loaded.current().name, "Mars");
    assert_eq!(loaded.current().grid.width, campaign.current().grid.width);
    assert!(loaded.current().grid.get(pos).unwrap().building.is_some());
    assert_eq!(loaded.current().drones.total_count(), 1);
}

#[test]
fn processor_operating_mode_survives_a_save_roundtrip() {
    let mut campaign = campaign();
    let core = campaign.current().grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    let planet = campaign.current_mut();
    planet.grid.reveal_around(pos, 1);
    planet.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
    assert!(planet.grid.place_building(pos, BuildingType::Assembler));
    planet
        .grid
        .get_mut(pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .overclocked = true;
    planet
        .grid
        .get_mut(pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .input_priority = true;
    planet
        .grid
        .get_mut(pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .standby = true;
    planet.auto_clocking = true;

    let json = save_to_json(&mut campaign).unwrap();
    let loaded = load_from_json(&json).unwrap();

    assert!(
        loaded
            .current()
            .grid
            .get(pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .overclocked
    );
    assert!(loaded.current().auto_clocking);
    assert!(
        loaded
            .current()
            .grid
            .get(pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .input_priority
    );
    assert!(
        loaded
            .current()
            .grid
            .get(pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .standby
    );
}

#[test]
fn freight_orders_and_typed_hoppers_survive_a_save_roundtrip() {
    let mut campaign = campaign();
    let key = (3, 4);
    campaign.current_mut().export = Some(crate::state::ExportOrder {
        target: 1,
        cargo: ResourceType::Alloy,
        target_pad: Some(GridPos::new(8, 9)),
        schedule_seconds: 10.0,
        priority: 2,
        reserve_source: 100.0,
        surplus_only: true,
    });
    campaign.current_mut().input_hoppers.insert(
        key,
        std::collections::HashMap::from([
            (ResourceType::Minerals, 20.0),
            (ResourceType::Alloy, 40.0),
        ]),
    );
    campaign.current_mut().input_buffers.insert(key, 60.0);
    campaign.current_mut().pod_loads.insert(key, 12.0);
    campaign.current_mut().output_buffers.insert(key, 7.5);
    campaign
        .current_mut()
        .pad_cargo
        .insert((5, 6), ResourceType::Components);
    campaign.current_mut().output_buffers.insert((5, 6), 23.0);

    let json = save_to_json(&mut campaign).unwrap();
    let loaded = load_from_json(&json).unwrap();
    let order = loaded.current().export.unwrap();

    assert_eq!(order.cargo, ResourceType::Alloy);
    assert_eq!(order.target_pad, Some(GridPos::new(8, 9)));
    assert_eq!(order.schedule_seconds, 10.0);
    assert_eq!(order.priority, 2);
    assert!(order.surplus_only);
    assert_eq!(
        loaded
            .current()
            .input_hoppers
            .get(&key)
            .and_then(|hoppers| hoppers.get(&ResourceType::Minerals)),
        Some(&20.0)
    );
    assert_eq!(loaded.current().input_buffers.get(&key), Some(&60.0));
    assert_eq!(loaded.current().pod_loads.get(&key), Some(&12.0));
    assert_eq!(loaded.current().output_buffers.get(&key), Some(&7.5));
    assert_eq!(
        loaded.current().pad_cargo.get(&(5, 6)),
        Some(&ResourceType::Components)
    );
    assert_eq!(loaded.current().output_buffers.get(&(5, 6)), Some(&23.0));
}

#[test]
fn saves_from_before_processor_policies_load_as_standard_running_lines() {
    fn strip_policy_fields(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(object) => {
                object.remove("input_priority");
                object.remove("standby");
                for child in object.values_mut() {
                    strip_policy_fields(child);
                }
            }
            serde_json::Value::Array(values) => {
                for child in values {
                    strip_policy_fields(child);
                }
            }
            _ => {}
        }
    }

    let mut campaign = campaign();
    let core = campaign.current().grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    let planet = campaign.current_mut();
    planet.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
    assert!(planet.grid.place_building(pos, BuildingType::Smelter));
    let building = planet.grid.get_mut(pos).unwrap().building.as_mut().unwrap();
    building.input_priority = true;
    building.standby = true;

    let json = save_to_json(&mut campaign).unwrap();
    let mut legacy: serde_json::Value = serde_json::from_str(&json).unwrap();
    strip_policy_fields(&mut legacy);
    let loaded = load_from_json(&serde_json::to_string(&legacy).unwrap()).unwrap();
    let building = loaded
        .current()
        .grid
        .get(pos)
        .unwrap()
        .building
        .as_ref()
        .unwrap();
    assert!(!building.input_priority);
    assert!(!building.standby);
}

#[test]
fn json_roundtrip_preserves_every_colonized_world() {
    let mut campaign = campaign();
    campaign.colonize(0);
    campaign.travel_to(0);
    campaign.current_mut().resources.biomass = 42.0;
    campaign.travel_to(super::super::campaign::STARTING_PLANET);

    let json = save_to_json(&mut campaign).unwrap();
    let mut loaded = load_from_json(&json).unwrap();

    assert_eq!(loaded.colonized_flags(), [true, false, true, false, false]);
    assert_eq!(
        loaded.current_index(),
        super::super::campaign::STARTING_PLANET
    );
    assert!(loaded.travel_to(0));
    assert_eq!(loaded.current().resources.biomass, 42.0);
}

#[test]
fn a_save_stamps_every_colonized_planet_with_the_same_time() {
    let mut campaign = campaign();
    campaign.current_mut().last_saved_unix = 0;
    campaign.colonize(0);
    campaign.planet_mut(0).unwrap().last_saved_unix = 0;
    save_to_json(&mut campaign).unwrap();
    assert!(campaign.current().last_saved_unix > 0);
    assert_eq!(
        campaign.planet(0).unwrap().last_saved_unix,
        campaign.current().last_saved_unix
    );
}

#[test]
fn the_written_save_carries_the_current_schema_version() {
    let mut campaign = campaign();
    let json = save_to_json(&mut campaign).unwrap();
    let save: SaveGame = serde_json::from_str(&json).unwrap();
    assert_eq!(save.version, SAVE_VERSION);
}

#[test]
fn an_unversioned_single_planet_save_still_loads() {
    // Exactly what version 0 wrote: a bare PlanetState.
    let planet = PlanetState::new(2, 3, GameConfig::default());
    let json = serde_json::to_string(&planet).unwrap();

    let loaded = load_from_json(&json).unwrap();
    assert_eq!(loaded.current().name, "Mars");
    assert_eq!(
        loaded.current_index(),
        super::super::campaign::STARTING_PLANET
    );
    assert_eq!(loaded.colonized_flags(), [false, false, true, false, false]);
}

#[test]
fn every_supported_save_version_has_a_migration_path() {
    assert_eq!(
        super::super::migrations::SUPPORTED_SAVE_VERSIONS,
        &[0, 1, 2, 3]
    );
    let planet = PlanetState::new(2, 3, GameConfig::default());
    let legacy = serde_json::to_string(&planet).unwrap();
    assert!(load_from_json(&legacy).is_ok());

    let mut campaign = campaign();
    let mut value = serde_json::to_value(SaveGame {
        version: 1,
        campaign: campaign.clone(),
    })
    .unwrap();
    let object = value
        .get_mut("campaign")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    object.remove("slot_name");
    object.remove("directive_history");
    object.remove("toast_history");
    object.remove("shipments");
    assert!(load_from_json(&serde_json::to_string(&value).unwrap()).is_ok());

    let mut v2 = serde_json::to_value(SaveGame {
        version: 2,
        campaign: campaign.clone(),
    })
    .unwrap();
    v2.get_mut("campaign")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap()
        .remove("ending_seen");
    assert!(load_from_json(&serde_json::to_string(&v2).unwrap()).is_ok());

    campaign.current_mut().resources.biomass = 7.0;
    assert!(load_from_json(&save_to_json(&mut campaign).unwrap()).is_ok());
}

#[test]
fn completion_acknowledgement_survives_a_save_roundtrip() {
    let mut campaign = campaign();
    campaign.ending_seen = true;

    let json = save_to_json(&mut campaign).unwrap();
    let loaded = load_from_json(&json).unwrap();

    assert!(loaded.ending_seen);
}

#[test]
fn nonsense_json_is_an_error_not_a_fresh_campaign() {
    assert!(load_from_json("{\"not\": \"a save\"}").is_err());
}

#[test]
fn load_from_json_applies_offline_progress_for_past_save() {
    let mut campaign = campaign();
    campaign.current_mut().last_saved_unix = unix_seconds_now() - 120;
    let save = SaveGame {
        version: SAVE_VERSION,
        campaign: campaign.clone(),
    };
    let json = serde_json::to_string(&save).unwrap();

    let loaded = load_from_json(&json).unwrap();
    assert!(loaded.current().last_offline_seconds > 0.0);
}

#[test]
fn load_from_json_skips_offline_progress_for_fresh_save() {
    let mut campaign = campaign();
    let json = save_to_json(&mut campaign).unwrap();

    let loaded = load_from_json(&json).unwrap();
    assert_eq!(loaded.current().last_offline_seconds, 0.0);
}
