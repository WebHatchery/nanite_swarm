use super::*;

#[test]
fn a_full_pod_is_thrown_and_lands_on_the_pad_waiting_for_it() {
    let mut campaign = two_world_campaign();
    let pad = pad_on(&mut campaign, VENUS);
    let driver = driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    // Hand-loaded: this is about what the driver does with cargo, not about
    // how the cargo got to it.
    campaign
        .current_mut()
        .input_buffers
        .insert((driver.x, driver.y), 100.0);
    let capacity = campaign.current().config.mass_driver.pod_capacity;
    let flight = transit_seconds(MARS, VENUS, &campaign.current().config.mass_driver);
    // Venus is running too, and its own drones start clearing the pad the
    // moment something lands on it, so the measure is what it took delivery of
    // rather than what is still standing there.
    let before = received_on(&campaign, VENUS, pad, ResourceType::Minerals);

    // 100 in the hopper at 3/s fills one 60-unit pod and starts a second.
    run(&mut campaign, 25.0);
    assert_eq!(campaign.shipments().len(), 1, "one pod should be up");
    assert_eq!(campaign.shipments()[0].amount, capacity);
    assert_eq!(campaign.shipments()[0].to, VENUS);
    assert_eq!(
        received_on(&campaign, VENUS, pad, ResourceType::Minerals),
        before,
        "nothing lands before the flight is over"
    );

    run(&mut campaign, flight);
    assert!(
        campaign.shipments().is_empty(),
        "the pod should have landed"
    );
    assert_eq!(
        received_on(&campaign, VENUS, pad, ResourceType::Minerals),
        before + capacity
    );
    assert_eq!(
        campaign
            .planet(VENUS)
            .unwrap()
            .pad_cargo
            .get(&(pad.x, pad.y)),
        Some(&ResourceType::Minerals)
    );
}

#[test]
fn a_pod_with_nowhere_to_land_holds_over_the_world_rather_than_vanishing() {
    let mut campaign = two_world_campaign();
    let driver = driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    campaign
        .current_mut()
        .input_buffers
        .insert((driver.x, driver.y), 100.0);
    let flight = transit_seconds(MARS, VENUS, &campaign.current().config.mass_driver);
    let banked_before = campaign.planet(VENUS).unwrap().resources.minerals;

    run(&mut campaign, 25.0 + flight + 10.0);

    assert_eq!(campaign.holding_shipments(), 1, "still up, still holding");
    assert_eq!(
        campaign.planet(VENUS).unwrap().resources.minerals,
        banked_before,
        "a world with no pad catches nothing"
    );

    // Build the pad and the cargo comes down without being thrown again.
    let pad = pad_on(&mut campaign, VENUS);
    run(&mut campaign, 1.0);
    assert_eq!(campaign.holding_shipments(), 0);
    assert!(held_on(&campaign, VENUS, pad) > 0.0);
}

#[test]
fn a_pad_holds_one_cargo_at_a_time() {
    let mut campaign = two_world_campaign();
    let pad = pad_on(&mut campaign, MARS);
    let planet = campaign.current_mut();

    assert!(planet.accept_pod(ResourceType::Minerals, 20.0, None));
    assert!(
        !planet.accept_pod(ResourceType::Alloy, 20.0, None),
        "a pad piled with ore is not a place to put alloy"
    );
    assert_eq!(
        planet.output_buffers.get(&(pad.x, pad.y)).copied(),
        Some(20.0)
    );

    // Once it has been carried off, the pad will take anything again.
    planet.output_buffers.insert((pad.x, pad.y), 0.0);
    assert!(planet.accept_pod(ResourceType::Alloy, 20.0, None));
    assert_eq!(
        planet.pad_cargo.get(&(pad.x, pad.y)),
        Some(&ResourceType::Alloy)
    );
}

#[test]
fn a_full_pad_turns_a_pod_away() {
    let mut campaign = two_world_campaign();
    let pad = pad_on(&mut campaign, MARS);
    let capacity = campaign.current().config.mass_driver.pad_capacity;
    let planet = campaign.current_mut();

    assert!(planet.accept_pod(ResourceType::Minerals, capacity * 0.5, None));
    assert!(planet.accept_pod(ResourceType::Minerals, capacity * 0.5, None));
    assert!(
        !planet.accept_pod(ResourceType::Minerals, capacity * 0.5, None),
        "the pad took what it could hold and no more"
    );
    assert_eq!(
        planet.output_buffers.get(&(pad.x, pad.y)).copied(),
        Some(capacity)
    );
}

#[test]
fn a_pod_that_finds_the_pad_full_stays_up_until_it_is_cleared() {
    let mut campaign = two_world_campaign();
    let pad = pad_on(&mut campaign, VENUS);
    let capacity = campaign.current().config.mass_driver.pad_capacity;
    let flight = transit_seconds(MARS, VENUS, &campaign.current().config.mass_driver);
    // Venus's pad is full and its drones are not moving, so the pod arriving
    // has nowhere to come down.
    let venus = campaign.planet_mut(VENUS).unwrap();
    venus.output_buffers.insert((pad.x, pad.y), capacity);
    venus
        .pad_cargo
        .insert((pad.x, pad.y), ResourceType::Minerals);
    venus.drones.drone_speed = 0.0;

    campaign.current_mut().launched_pods.push(Shipment {
        from: MARS,
        to: VENUS,
        cargo: ResourceType::Minerals,
        amount: 20.0,
        remaining: flight,
        transit: flight,
        target_pad: None,
        overflow: false,
        priority: 0,
    });

    run(&mut campaign, flight + 2.0);
    assert_eq!(campaign.holding_shipments(), 1);

    // Clear the pad and it comes down without being thrown again.
    campaign
        .planet_mut(VENUS)
        .unwrap()
        .output_buffers
        .insert((pad.x, pad.y), 0.0);
    run(&mut campaign, 1.0);
    assert_eq!(campaign.holding_shipments(), 0);
}

#[test]
fn cycling_a_recovered_pad_releases_an_in_flight_pod_from_the_old_coordinate() {
    let mut campaign = two_world_campaign();
    let old_pad = pad_on(&mut campaign, VENUS);
    let driver = driver_on_current(&mut campaign);
    campaign.current_mut().export = Some(ExportOrder {
        target: VENUS,
        target_pad: Some(old_pad),
        ..ExportOrder::default()
    });
    campaign
        .current_mut()
        .input_buffers
        .insert((driver.x, driver.y), 100.0);

    let flight = transit_seconds(MARS, VENUS, &campaign.current().config.mass_driver);
    run(&mut campaign, 25.0);
    assert_eq!(campaign.shipments().len(), 1);

    let new_pad = second_pad_on(&mut campaign, VENUS);
    campaign
        .planet_mut(VENUS)
        .unwrap()
        .grid
        .get_mut(old_pad)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .standby = true;
    campaign.planet_mut(VENUS).unwrap().grid.update_power_grid();

    run(&mut campaign, flight);
    assert_eq!(campaign.holding_shipments(), 1);

    campaign.cycle_export_pad_for(MARS);
    assert_eq!(campaign.export_order().unwrap().target_pad, None);
    run(&mut campaign, 1.0);

    assert_eq!(campaign.holding_shipments(), 0);
    assert!(held_on(&campaign, VENUS, new_pad) > 0.0);
}

#[test]
fn retargeting_a_route_releases_an_in_flight_pod_from_its_old_pad() {
    let mut campaign = two_world_campaign();
    campaign.colonize(SATURN);
    let old_pad = pad_on(&mut campaign, VENUS);
    let new_pad = second_pad_on(&mut campaign, VENUS);
    let driver = driver_on_current(&mut campaign);
    campaign.current_mut().export = Some(ExportOrder {
        target: VENUS,
        target_pad: Some(old_pad),
        ..ExportOrder::default()
    });
    campaign
        .current_mut()
        .input_buffers
        .insert((driver.x, driver.y), 100.0);

    let flight = transit_seconds(MARS, VENUS, &campaign.current().config.mass_driver);
    run(&mut campaign, 25.0);
    assert_eq!(campaign.shipments().len(), 1);

    // The pod still belongs to Venus, but the standing order now points at
    // Saturn. The old pad is taken offline before the pod arrives.
    campaign.current_mut().export = Some(ExportOrder {
        target: SATURN,
        ..campaign.current().export.unwrap()
    });
    campaign
        .planet_mut(VENUS)
        .unwrap()
        .grid
        .get_mut(old_pad)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .standby = true;
    campaign.planet_mut(VENUS).unwrap().grid.update_power_grid();

    run(&mut campaign, flight);

    assert_eq!(campaign.holding_shipments(), 0);
    assert!(held_on(&campaign, VENUS, new_pad) > 0.0);
}

#[test]
fn what_lands_on_a_pad_is_carried_off_it_like_anything_else() {
    let mut campaign = two_world_campaign();
    let pad = pad_on(&mut campaign, MARS);
    let planet = campaign.current_mut();
    planet.resources.minerals = 0.0;
    planet.output_buffers.insert((pad.x, pad.y), 40.0);
    planet
        .pad_cargo
        .insert((pad.x, pad.y), ResourceType::Minerals);

    run(&mut campaign, 15.0);

    assert!(
        campaign.current().resources.minerals > 0.0,
        "a drone should have walked it to the Core"
    );
    assert!(held_on(&campaign, MARS, pad) < 40.0);
}

#[test]
fn tearing_down_a_pad_takes_what_was_standing_on_it() {
    let mut campaign = two_world_campaign();
    let pad = pad_on(&mut campaign, MARS);
    let planet = campaign.current_mut();
    planet.output_buffers.insert((pad.x, pad.y), 40.0);
    planet.pad_cargo.insert((pad.x, pad.y), ResourceType::Alloy);

    assert!(campaign.current_mut().try_sell_building(pad));

    assert_eq!(held_on(&campaign, MARS, pad), 0.0);
    assert!(!campaign.current().pad_cargo.contains_key(&(pad.x, pad.y)));
}

#[test]
fn a_world_left_behind_keeps_feeding_the_one_the_swarm_moved_to() {
    let mut campaign = two_world_campaign();
    let driver = driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    campaign
        .current_mut()
        .input_buffers
        .insert((driver.x, driver.y), 200.0);

    let pad = pad_on(&mut campaign, VENUS);
    assert!(campaign.travel_to(VENUS));
    let flight = transit_seconds(MARS, VENUS, &campaign.current().config.mass_driver);
    let before = received_on(&campaign, VENUS, pad, ResourceType::Minerals);

    run(&mut campaign, 25.0 + flight);

    assert!(
        received_on(&campaign, VENUS, pad, ResourceType::Minerals) > before,
        "Mars should still be throwing at the world the swarm left for"
    );
}

#[test]
fn a_pod_that_is_up_when_the_swarm_travels_still_lands() {
    let mut campaign = two_world_campaign();
    let driver = driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    campaign
        .current_mut()
        .input_buffers
        .insert((driver.x, driver.y), 100.0);
    let flight = transit_seconds(MARS, VENUS, &campaign.current().config.mass_driver);

    let pad = pad_on(&mut campaign, VENUS);
    let before = received_on(&campaign, VENUS, pad, ResourceType::Minerals);

    run(&mut campaign, 25.0);
    assert_eq!(campaign.shipments().len(), 1);

    // The swarm follows its own cargo across.
    assert!(campaign.travel_to(VENUS));
    run(&mut campaign, flight);

    assert!(campaign.shipments().is_empty());
    assert!(received_on(&campaign, VENUS, pad, ResourceType::Minerals) > before);
}
