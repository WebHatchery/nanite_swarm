use super::*;

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
fn seed_ship_commitment_survives_a_save_roundtrip() {
    let mut campaign = campaign();
    campaign.current_mut().toggle_seed_ship_commitment();
    assert!(campaign.current().seed_ship.committed);

    let json = save_to_json(&mut campaign).unwrap();
    let loaded = load_from_json(&json).unwrap();

    assert!(
        loaded.current().seed_ship.committed,
        "the yard must remain committed after a save/load"
    );
}

#[test]
fn parked_research_progress_survives_a_save_roundtrip() {
    let mut campaign = campaign();
    campaign
        .research
        .parked_progress
        .insert("adaptive_clocking".to_string(), 42.5);

    let json = save_to_json(&mut campaign).unwrap();
    let loaded = load_from_json(&json).unwrap();

    assert_eq!(
        loaded.research.parked_progress.get("adaptive_clocking"),
        Some(&42.5)
    );
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
    campaign.colonize(1);
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
