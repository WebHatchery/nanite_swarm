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
    assert_eq!(planet.mass_drivers_built(), 1);
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
    assert_eq!(planet.landing_pads_built(), 1);
    pos
}

/// A second powered pad at a different tile, for testing route recovery after
/// the first pad is no longer eligible.
fn second_pad_on(campaign: &mut Campaign, index: usize) -> GridPos {
    let planet = campaign.planet_mut(index).expect("colonized world");
    let core = planet.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    planet.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
    planet.grid.reveal_around(core, 3);
    planet.unlock_building(BuildingType::LandingPad);
    planet.select_building(BuildingType::LandingPad);
    assert!(planet.try_place_building(pos), "second pad beside the Core");
    planet.grid.update_power_grid();
    assert_eq!(planet.landing_pads_online(), 2);
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
    campaign.current_mut().export = Some(ExportOrder {
        target,
        cargo,
        ..ExportOrder::default()
    });
}

fn transit_seconds(from: usize, to: usize, config: &MassDriverConfig) -> f32 {
    let data = crate::data::test_data();
    super::transit_seconds(data.data(), from, to, config)
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
fn export_rate_explains_the_effect_of_a_shipping_cooldown() {
    let mut campaign = two_world_campaign();
    driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);

    assert!((campaign.current().export_rate_per_minute() - 180.0).abs() < 0.001);

    campaign
        .current_mut()
        .export
        .as_mut()
        .unwrap()
        .schedule_seconds = 10.0;
    assert!((campaign.current().export_rate_per_minute() - 120.0).abs() < 0.001);
}

#[test]
fn export_rate_is_zero_without_a_route_or_online_driver() {
    let mut campaign = two_world_campaign();
    assert_eq!(campaign.current().export_rate_per_minute(), 0.0);

    driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    let driver = campaign
        .current()
        .grid
        .find_buildings(BuildingType::MassDriver)
        .first()
        .copied()
        .unwrap();
    campaign
        .current_mut()
        .grid
        .get_mut(driver)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .standby = true;
    campaign.current_mut().grid.update_power_grid();

    assert_eq!(campaign.current().export_rate_per_minute(), 0.0);
}

#[test]
fn offline_production_loads_a_standing_order_into_a_full_pod() {
    let mut campaign = two_world_campaign();
    driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    let launched = {
        let planet = campaign.current_mut();
        planet.last_offline_simulated = 20.0;
        planet.last_offline_report.minerals_gained = 60.0;
        planet.resources.minerals = 60.0;

        planet.apply_offline_export_progress()
    };

    assert_eq!(launched, 1);
    assert_eq!(campaign.current().launched_pods.len(), 1);
    assert_eq!(campaign.current().launched_pods[0].amount, 60.0);
    assert_eq!(campaign.current().resources.minerals, 0.0);

    campaign.update_shipments(0.0);
    assert_eq!(campaign.shipments().len(), 1);
}

#[test]
fn offline_export_keeps_the_source_reserve_intact() {
    let mut campaign = two_world_campaign();
    driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    let planet = campaign.current_mut();
    planet.export.as_mut().unwrap().surplus_only = true;
    planet.export.as_mut().unwrap().reserve_source = 50.0;
    planet.last_offline_simulated = 20.0;
    planet.last_offline_report.minerals_gained = 60.0;
    planet.resources.minerals = 80.0;

    let launched = planet.apply_offline_export_progress();

    assert_eq!(launched, 0);
    assert!((planet.resources.minerals - 50.0).abs() < 0.001);
    assert!((planet.pod_fraction() - 0.5).abs() < 0.001);
}

#[test]
fn offline_export_applies_a_schedule_after_a_full_pod() {
    let mut campaign = two_world_campaign();
    let driver = driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    let planet = campaign.current_mut();
    planet.export.as_mut().unwrap().schedule_seconds = 10.0;
    planet.last_offline_simulated = 31.0;
    planet.last_offline_report.minerals_gained = 63.0;
    planet.resources.minerals = 63.0;

    let launched = planet.apply_offline_export_progress();

    assert_eq!(launched, 1);
    assert_eq!(planet.pod_loads.get(&(driver.x, driver.y)), Some(&3.0));
    assert!((planet.export_cooldown - 10.0).abs() < 0.001);
}

#[test]
fn changing_a_route_does_not_relabel_typed_cargo_in_the_hopper() {
    let mut campaign = two_world_campaign();
    let driver = driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    let key = (driver.x, driver.y);
    let mut hoppers = std::collections::HashMap::new();
    hoppers.insert(ResourceType::Minerals, 60.0);
    hoppers.insert(ResourceType::Alloy, 60.0);
    campaign.current_mut().input_buffers.insert(key, 120.0);
    campaign.current_mut().input_hoppers.insert(key, hoppers);

    // The first cycle advances the standing order from Minerals to Alloy.
    campaign.cycle_export_cargo();
    run(&mut campaign, 25.0);

    assert_eq!(campaign.shipments().len(), 1);
    assert_eq!(campaign.shipments()[0].cargo, ResourceType::Alloy);
    assert_eq!(
        campaign
            .current()
            .input_hoppers
            .get(&key)
            .and_then(|hoppers| hoppers.get(&ResourceType::Minerals)),
        Some(&60.0),
        "the old resource must remain available for a later order"
    );
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
        .accept_pod(ResourceType::Minerals, 30.0, None);
    let held = campaign.current().pad_summary(pad);
    assert!(held.contains("Minerals"), "{held}");
    assert!(held.contains("30"), "{held}");
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

mod arrivals;
mod orders;
