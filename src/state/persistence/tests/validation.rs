use super::*;

#[test]
fn a_save_with_negative_live_resources_is_rejected() {
    let mut campaign = campaign();
    campaign.current_mut().resources.biomass = -1.0;

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_an_invalid_world_configuration_is_rejected() {
    let mut campaign = campaign();
    campaign.current_mut().config.resources.research_rate = -1.0;

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_a_building_claiming_the_wrong_tile_is_rejected() {
    let mut campaign = campaign();
    let core = campaign.current().grid.find_core().unwrap();
    let position = GridPos::new(core.x + 1, core.y);
    let planet = campaign.current_mut();
    planet.grid.get_mut(position).unwrap().terrain = crate::engine::TerrainType::Empty;
    planet.grid.reveal_around(position, 1);
    assert!(planet.grid.place_building(position, BuildingType::Drill));
    planet
        .grid
        .get_mut(position)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .position = core;

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_an_out_of_bounds_buffer_is_rejected() {
    let mut campaign = campaign();
    campaign.current_mut().output_buffers.insert((-1, 0), 4.0);

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_unknown_parked_research_is_rejected() {
    let mut campaign = campaign();
    campaign
        .research
        .parked_progress
        .insert("research_that_does_not_exist".to_string(), 1.0);

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_a_seed_ship_past_its_plan_is_rejected() {
    let mut campaign = campaign();
    let json = save_to_json(&mut campaign).unwrap();
    let mut malformed: serde_json::Value = serde_json::from_str(&json).unwrap();
    malformed["campaign"]["planets"][2]["seed_ship"]["stage"] = serde_json::Value::from(usize::MAX);

    assert!(load_from_json(&serde_json::to_string(&malformed).unwrap()).is_err());
}

#[test]
fn loading_rebuilds_stale_building_unlocks_from_campaign_research() {
    let mut campaign = campaign();
    campaign
        .current_mut()
        .unlock_building(BuildingType::FusionReactor);
    assert!(campaign
        .current()
        .is_building_researched(BuildingType::FusionReactor));

    let json = save_to_json(&mut campaign).unwrap();
    let loaded = load_from_json(&json).unwrap();

    assert!(!loaded
        .current()
        .is_building_researched(BuildingType::FusionReactor));
}

#[test]
fn a_save_with_duplicate_drone_ids_is_rejected() {
    let mut campaign = campaign();
    let core = campaign.current().grid.find_core().unwrap();
    let planet = campaign.current_mut();
    planet.drones.spawn_drone(core);
    planet.drones.spawn_drone(core);
    let first_id = planet.drones.drones()[0].id;
    planet.drones.drones_mut()[1].id = first_id;

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_a_stale_drone_allocator_is_rejected() {
    let mut campaign = campaign();
    let core = campaign.current().grid.find_core().unwrap();
    campaign.current_mut().drones.spawn_drone(core);
    let json = save_to_json(&mut campaign).unwrap();
    let mut malformed: serde_json::Value = serde_json::from_str(&json).unwrap();
    malformed["campaign"]["planets"][2]["drones"]["next_id"] = serde_json::Value::from(1);

    assert!(load_from_json(&serde_json::to_string(&malformed).unwrap()).is_err());
}

#[test]
fn a_save_with_an_invalid_camera_zoom_is_rejected() {
    let mut campaign = campaign();
    campaign.current_mut().camera.zoom = 0.0;

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_non_physical_freight_cargo_is_rejected() {
    let mut campaign = campaign();
    campaign.colonize(1);
    campaign.current_mut().export = Some(crate::state::ExportOrder {
        target: 1,
        cargo: ResourceType::Data,
        ..crate::state::ExportOrder::default()
    });

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_an_out_of_range_freight_priority_is_rejected() {
    let mut campaign = campaign();
    campaign.colonize(1);
    campaign.current_mut().export = Some(crate::state::ExportOrder {
        target: 1,
        priority: 4,
        ..crate::state::ExportOrder::default()
    });

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_an_out_of_bounds_freight_landing_pad_is_rejected() {
    let mut campaign = campaign();
    campaign.colonize(1);
    campaign.current_mut().export = Some(crate::state::ExportOrder {
        target: 1,
        target_pad: Some(GridPos::new(-1, 0)),
        ..crate::state::ExportOrder::default()
    });

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}

#[test]
fn a_save_with_an_invalid_queued_freight_pod_is_rejected() {
    let mut campaign = campaign();
    campaign.colonize(1);
    campaign
        .current_mut()
        .launched_pods
        .push(crate::state::Shipment {
            from: 2,
            to: 1,
            cargo: ResourceType::Data,
            amount: 20.0,
            remaining: 10.0,
            transit: 20.0,
            target_pad: None,
            overflow: false,
            priority: 4,
        });

    let json = save_to_json(&mut campaign).unwrap();

    assert!(load_from_json(&json).is_err());
}
