use super::*;
use crate::data::{GameConfig, MassDriverConfig};
use crate::engine::TerrainType;
use crate::state::TICK_SECONDS;

/// Campaign slots, by the order they sit in `planets.json`.
const MERCURY: usize = 0;
const VENUS: usize = 1;
const MARS: usize = 2;
const SATURN: usize = 4;

/// Mars with Venus colonized, both able to hold whatever lands on them, and
/// neither able to collapse: these tests are about cargo, not about power.
fn two_world_campaign() -> Campaign {
    let mut config = GameConfig::default();
    config.resources.base_mineral_cap = 100_000.0;
    config.collapse.negative_power_seconds = f32::MAX;
    let mut campaign = Campaign::new(config, 42);
    campaign.colonize(VENUS);
    campaign
}

/// Put a powered Mass Driver beside the current world's Core.
fn driver_on_current(campaign: &mut Campaign) -> GridPos {
    let planet = campaign.current_mut();
    planet.resources.minerals = 10_000.0;
    planet.resources.energy = 10_000.0;
    planet.config.resources.max_energy = 10_000.0;
    let core = planet.grid.find_core().unwrap();
    let pos = GridPos::new(core.x - 1, core.y);
    planet.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    planet.grid.reveal_around(core, 3);
    planet.unlock_building(BuildingType::MassDriver);
    planet.select_building(BuildingType::MassDriver);
    assert!(planet.try_place_building(pos), "driver beside the Core");
    planet.grid.update_power_grid();
    assert_eq!(planet.mass_drivers_online(), 1);
    pos
}

/// A drill on ordinary ground on the Core's other side.
fn drill_on_current(campaign: &mut Campaign) -> GridPos {
    let planet = campaign.current_mut();
    let core = planet.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    {
        let tile = planet.grid.get_mut(pos).unwrap();
        tile.terrain = TerrainType::Empty;
        tile.ore_richness = 1.0;
    }
    planet.select_building(BuildingType::Drill);
    assert!(planet.try_place_building(pos), "drill beside the Core");
    planet.grid.update_power_grid();
    pos
}

/// A powered Landing Pad beside a world's Core, so a pod thrown at it has
/// somewhere to come down.
fn pad_on(campaign: &mut Campaign, index: usize) -> GridPos {
    let planet = campaign.planet_mut(index).expect("colonized world");
    planet.resources.minerals = 10_000.0;
    planet.resources.energy = 10_000.0;
    planet.config.resources.max_energy = 10_000.0;
    let core = planet.grid.find_core().unwrap();
    let pos = GridPos::new(core.x, core.y + 1);
    planet.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    planet.grid.reveal_around(core, 3);
    planet.unlock_building(BuildingType::LandingPad);
    planet.select_building(BuildingType::LandingPad);
    assert!(planet.try_place_building(pos), "pad beside the Core");
    planet.grid.update_power_grid();
    assert_eq!(planet.landing_pads_online(), 1);
    pos
}

/// What a world's pad is holding.
fn held_on(campaign: &Campaign, index: usize, pad: GridPos) -> f32 {
    campaign
        .planet(index)
        .and_then(|planet| planet.output_buffers.get(&(pad.x, pad.y)).copied())
        .unwrap_or(0.0)
}

/// Everything a world has taken delivery of: what is still on its pad plus
/// whatever its own drones have already carried off it.
fn received_on(campaign: &Campaign, index: usize, pad: GridPos, cargo: ResourceType) -> f32 {
    held_on(campaign, index, pad)
        + campaign
            .planet(index)
            .map(|planet| planet.resources.get(cargo))
            .unwrap_or(0.0)
}

/// Run the campaign the way the game loop does: the world in front of the
/// player, the ones behind it, and everything in flight between them.
fn run(campaign: &mut Campaign, seconds: f32) {
    let ticks = (seconds / TICK_SECONDS) as u32;
    for _ in 0..ticks {
        campaign.current_mut().step(TICK_SECONDS, false);
        campaign.update_background(TICK_SECONDS);
        campaign.update_shipments(TICK_SECONDS);
    }
}

fn set_route(campaign: &mut Campaign, target: usize, cargo: ResourceType) {
    campaign.current_mut().export = Some(ExportOrder { target, cargo });
}

#[test]
fn a_farther_world_is_a_longer_throw() {
    let config = MassDriverConfig::default();
    let near = transit_seconds(MARS, VENUS, &config);
    let far = transit_seconds(MERCURY, SATURN, &config);
    assert!(near > 0.0);
    assert!(far > near, "{far} should beat {near}");
}

#[test]
fn no_two_worlds_are_closer_than_the_minimum_throw() {
    let config = MassDriverConfig {
        min_transit_seconds: 500.0,
        ..MassDriverConfig::default()
    };
    assert_eq!(transit_seconds(MARS, VENUS, &config), 500.0);
}

#[test]
fn a_driver_with_nowhere_to_throw_is_not_a_destination() {
    let mut campaign = two_world_campaign();
    let driver = driver_on_current(&mut campaign);
    drill_on_current(&mut campaign);
    campaign.current_mut().resources.minerals = 0.0;

    run(&mut campaign, 20.0);

    let planet = campaign.current();
    assert_eq!(
        planet
            .input_buffers
            .get(&(driver.x, driver.y))
            .copied()
            .unwrap_or(0.0),
        0.0,
        "nothing should be carried to a driver with no route"
    );
    assert!(planet.resources.minerals > 0.0, "the ore went to the Core");
}

#[test]
fn ore_is_carried_to_a_driver_that_has_somewhere_to_throw_it() {
    let mut campaign = two_world_campaign();
    driver_on_current(&mut campaign);
    drill_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    campaign.current_mut().resources.minerals = 0.0;

    run(&mut campaign, 20.0);

    assert!(
        campaign.current().pod_fraction() > 0.0,
        "the drill's ore should be loading a pod"
    );
}

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

    assert!(planet.accept_pod(ResourceType::Minerals, 20.0));
    assert!(
        !planet.accept_pod(ResourceType::Alloy, 20.0),
        "a pad piled with ore is not a place to put alloy"
    );
    assert_eq!(
        planet.output_buffers.get(&(pad.x, pad.y)).copied(),
        Some(20.0)
    );

    // Once it has been carried off, the pad will take anything again.
    planet.output_buffers.insert((pad.x, pad.y), 0.0);
    assert!(planet.accept_pod(ResourceType::Alloy, 20.0));
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

    assert!(planet.accept_pod(ResourceType::Minerals, capacity * 0.5));
    assert!(planet.accept_pod(ResourceType::Minerals, capacity * 0.5));
    assert!(
        !planet.accept_pod(ResourceType::Minerals, capacity * 0.5),
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

#[test]
fn the_inspector_reads_a_driver_and_a_pad_out_in_words() {
    let mut campaign = two_world_campaign();
    let pad = pad_on(&mut campaign, MARS);
    let driver = driver_on_current(&mut campaign);

    assert!(
        campaign
            .current()
            .export_summary(driver)
            .contains("No route"),
        "a driver with no order should say so"
    );
    assert!(campaign.current().pad_summary(pad).contains("Empty"));

    set_route(&mut campaign, VENUS, ResourceType::Alloy);
    campaign
        .current_mut()
        .pod_loads
        .insert((driver.x, driver.y), 15.0);
    let summary = campaign.current().export_summary(driver);
    assert!(summary.contains("Alloy"), "{summary}");
    assert!(summary.contains("Venus"), "{summary}");

    campaign
        .current_mut()
        .accept_pod(ResourceType::Minerals, 30.0);
    let held = campaign.current().pad_summary(pad);
    assert!(held.contains("Minerals"), "{held}");
    assert!(held.contains("30"), "{held}");
}

#[test]
fn cycling_the_target_walks_the_colonized_worlds_and_then_holds() {
    let mut campaign = two_world_campaign();
    campaign.colonize(SATURN);

    campaign.cycle_export_target();
    assert_eq!(
        campaign.export_order().map(|order| order.target),
        Some(VENUS)
    );
    campaign.cycle_export_target();
    assert_eq!(
        campaign.export_order().map(|order| order.target),
        Some(SATURN)
    );
    campaign.cycle_export_target();
    assert!(
        campaign.export_order().is_none(),
        "past the last world is holding, not the first again"
    );
}

#[test]
fn a_world_is_never_offered_itself_as_a_target() {
    let mut campaign = two_world_campaign();
    for _ in 0..8 {
        campaign.cycle_export_target();
        if let Some(order) = campaign.export_order() {
            assert_ne!(order.target, campaign.current_index());
        }
    }
}

#[test]
fn a_lone_world_has_nowhere_to_ship_to() {
    let mut config = GameConfig::default();
    config.collapse.negative_power_seconds = f32::MAX;
    let mut campaign = Campaign::new(config, 42);

    campaign.cycle_export_target();

    assert!(campaign.export_order().is_none());
}

#[test]
fn cargo_cycles_through_what_the_drivers_accept() {
    let mut campaign = two_world_campaign();
    campaign.cycle_export_target();
    assert_eq!(
        campaign.export_order().map(|order| order.cargo),
        Some(ResourceType::Minerals)
    );

    campaign.cycle_export_cargo();
    assert_eq!(
        campaign.export_order().map(|order| order.cargo),
        Some(ResourceType::Alloy)
    );
    campaign.cycle_export_cargo();
    assert_eq!(
        campaign.export_order().map(|order| order.cargo),
        Some(ResourceType::Minerals)
    );
}

#[test]
fn a_cargo_with_no_destination_is_not_an_order() {
    let mut campaign = two_world_campaign();
    campaign.cycle_export_cargo();
    assert!(campaign.export_order().is_none());
}

#[test]
fn tearing_down_a_driver_takes_its_half_loaded_pod_with_it() {
    let mut campaign = two_world_campaign();
    let driver = driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    campaign
        .current_mut()
        .input_buffers
        .insert((driver.x, driver.y), 30.0);

    run(&mut campaign, 5.0);
    assert!(campaign.current().pod_fraction() > 0.0);

    assert!(campaign.current_mut().try_sell_building(driver));
    assert_eq!(campaign.current().pod_fraction(), 0.0);
    assert!(campaign.shipments().is_empty());
}
