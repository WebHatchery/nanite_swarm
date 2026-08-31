use super::*;

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
fn a_save_with_an_impossible_current_world_is_recovered_from_the_backup() {
    let mut store = MapStore::default();
    let mut good = campaign();
    good.current_mut().resources.biomass = 33.0;
    save_campaign(&mut store, "save", &mut good).unwrap();

    let mut newer = campaign();
    newer.current_mut().resources.biomass = 44.0;
    save_campaign(&mut store, "save", &mut newer).unwrap();

    let mut malformed: serde_json::Value =
        serde_json::from_str(&store.read("save").unwrap()).unwrap();
    malformed["campaign"]["current"] = serde_json::Value::from(99);
    store
        .write("save", &serde_json::to_string(&malformed).unwrap())
        .unwrap();

    let (recovered, source) = load_campaign(&store, "save").unwrap();
    assert_eq!(source, LoadSource::Backup);
    assert_eq!(recovered.current().resources.biomass, 33.0);
}

#[test]
fn a_save_with_a_truncated_grid_is_recovered_from_the_backup() {
    let mut store = MapStore::default();
    let mut good = campaign();
    good.current_mut().resources.biomass = 33.0;
    save_campaign(&mut store, "save", &mut good).unwrap();

    let mut newer = campaign();
    newer.current_mut().resources.biomass = 44.0;
    save_campaign(&mut store, "save", &mut newer).unwrap();

    let mut malformed: serde_json::Value =
        serde_json::from_str(&store.read("save").unwrap()).unwrap();
    malformed["campaign"]["planets"][2]["grid"]["width"] = serde_json::Value::from(1);
    store
        .write("save", &serde_json::to_string(&malformed).unwrap())
        .unwrap();

    let (recovered, source) = load_campaign(&store, "save").unwrap();
    assert_eq!(source, LoadSource::Backup);
    assert_eq!(recovered.current().resources.biomass, 33.0);
}

#[test]
fn a_save_with_a_drone_past_its_route_is_recovered_from_the_backup() {
    let mut store = MapStore::default();
    let mut good = campaign();
    good.current_mut().resources.biomass = 33.0;
    save_campaign(&mut store, "save", &mut good).unwrap();

    let mut newer = campaign();
    let core = newer.current().grid.find_core().unwrap();
    newer.current_mut().drones.spawn_drone(core);
    newer.current_mut().resources.biomass = 44.0;
    save_campaign(&mut store, "save", &mut newer).unwrap();

    let mut malformed: serde_json::Value =
        serde_json::from_str(&store.read("save").unwrap()).unwrap();
    malformed["campaign"]["planets"][2]["drones"]["drones"][0]["path_index"] =
        serde_json::Value::from(1);
    store
        .write("save", &serde_json::to_string(&malformed).unwrap())
        .unwrap();

    let (recovered, source) = load_campaign(&store, "save").unwrap();
    assert_eq!(source, LoadSource::Backup);
    assert_eq!(recovered.current().resources.biomass, 33.0);
}

#[test]
fn a_save_with_a_freight_order_to_an_untouched_world_is_rejected() {
    let mut campaign = campaign();
    campaign.current_mut().export = Some(crate::state::ExportOrder {
        target: 1,
        ..crate::state::ExportOrder::default()
    });

    let json = save_to_json(&mut campaign).unwrap();
    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_a_queued_pod_from_an_impossible_world_is_rejected() {
    let mut campaign = campaign();
    campaign.current_mut().launched_pods.push(Shipment {
        from: 99,
        to: crate::state::campaign::STARTING_PLANET,
        cargo: ResourceType::Minerals,
        amount: 10.0,
        remaining: 1.0,
        transit: 1.0,
        target_pad: None,
        overflow: false,
        priority: 0,
    });

    let json = save_to_json(&mut campaign).unwrap();
    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_a_world_in_the_wrong_campaign_slot_is_rejected() {
    let mut campaign = campaign();
    campaign.current_mut().planet_index = 4;

    let json = save_to_json(&mut campaign).unwrap();
    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_no_core_is_rejected_before_simulation() {
    let mut campaign = campaign();
    let core = campaign.current().grid.find_core().unwrap();
    campaign.current_mut().grid.remove_building(core);

    let json = save_to_json(&mut campaign).unwrap();
    assert!(load_from_json(&json).is_err());
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
