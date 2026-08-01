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
fn a_full_pod_is_thrown_and_lands_on_the_target_world() {
    let mut campaign = two_world_campaign();
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
    let landed_before = campaign.planet(VENUS).unwrap().resources.minerals;

    // 100 in the hopper at 3/s fills one 60-unit pod and starts a second.
    run(&mut campaign, 25.0);
    assert_eq!(campaign.shipments().len(), 1, "one pod should be up");
    assert_eq!(campaign.shipments()[0].amount, capacity);
    assert_eq!(campaign.shipments()[0].to, VENUS);
    assert_eq!(
        campaign.planet(VENUS).unwrap().resources.minerals,
        landed_before,
        "nothing lands before the flight is over"
    );

    run(&mut campaign, flight);
    assert!(
        campaign.shipments().is_empty(),
        "the pod should have landed"
    );
    assert_eq!(
        campaign.planet(VENUS).unwrap().resources.minerals,
        landed_before + capacity
    );
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

    assert!(campaign.travel_to(VENUS));
    let landed_before = campaign.current().resources.minerals;
    let flight = transit_seconds(MARS, VENUS, &campaign.current().config.mass_driver);

    run(&mut campaign, 25.0 + flight);

    assert!(
        campaign.current().resources.minerals > landed_before,
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

    run(&mut campaign, 25.0);
    assert_eq!(campaign.shipments().len(), 1);

    // The swarm follows its own cargo across.
    assert!(campaign.travel_to(VENUS));
    let landed_before = campaign.current().resources.minerals;
    run(&mut campaign, flight);

    assert!(campaign.shipments().is_empty());
    assert!(campaign.current().resources.minerals > landed_before);
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
