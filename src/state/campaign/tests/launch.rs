//! Seed Ship construction, launch gating, and campaign-ending tests.

use super::*;
use crate::engine::{BuildingType, GridPos};

/// Finish the current world's Seed Ship the slow way, through the sim.
///
/// The later stages are gated on research, so the fixture has to have done
/// that research the same as a player would.
fn build_seed_ship(campaign: &mut Campaign) {
    for stage in &crate::data::test_data().data().seed_ship.stages {
        if let Some(tech) = stage.requires.as_deref() {
            if !campaign.research.unlocked_techs.iter().any(|id| id == tech) {
                campaign.research.unlocked_techs.push(tech.to_string());
            }
        }
    }
    campaign.sync_research();

    let planet = campaign.current_mut();
    planet.config.resources.base_mineral_cap = 1_000_000.0;
    planet.resources.minerals = 100_000.0;
    planet.resources.data = 100_000.0;
    planet.resources.biomass = 100_000.0;
    planet.resources.alloy = 100_000.0;
    planet.resources.components = 100_000.0;
    planet.toggle_seed_ship_commitment();
    for _ in 0..2_000 {
        planet.update_seed_ship(1.0);
    }
    assert!(planet.seed_ship.is_ready_to_launch());
}

/// The ship can leave only when a live Mass Driver is standing beside the
/// Core. The fixture uses a known adjacent tile so the test exercises the same
/// powered infrastructure a player would build, rather than editing the
/// building count behind the campaign's back.
fn place_launch_driver(campaign: &mut Campaign) {
    let planet = campaign.current_mut();
    let core = planet.grid.find_core().unwrap();
    let driver = GridPos::new(core.x + 1, core.y);
    let tile = planet.grid.get_mut(driver).unwrap();
    tile.terrain = crate::engine::TerrainType::Empty;
    tile.building = None;
    planet.grid.reveal_around(driver, 1);
    planet.select_building(BuildingType::MassDriver);
    assert!(planet.try_place_building(driver));
    planet.grid.update_power_grid();
    assert_eq!(planet.mass_drivers_online(), 1);
}

#[test]
fn an_untouched_world_cannot_be_reached_without_a_ship() {
    let mut campaign = super::campaign();
    assert!(!campaign.current().seed_ship.is_ready_to_launch());
    assert!(!campaign.launch_seed_ship(0));
    assert!(!campaign.is_colonized(0));
    assert_eq!(campaign.current_index(), STARTING_PLANET);
}

#[test]
fn launching_colonizes_the_target_and_carries_the_swarm_there() {
    let mut campaign = super::campaign();
    build_seed_ship(&mut campaign);
    place_launch_driver(&mut campaign);

    assert!(campaign.launch_seed_ship(1));
    assert!(campaign.is_colonized(1));
    assert_eq!(campaign.current_index(), 1);
    assert_eq!(campaign.current().name, "Venus");
}

#[test]
fn the_ship_is_spent_by_the_launch() {
    let mut campaign = super::campaign();
    build_seed_ship(&mut campaign);
    place_launch_driver(&mut campaign);
    campaign.launch_seed_ship(1);
    campaign.travel_to(STARTING_PLANET);

    let ship = &campaign.current().seed_ship;
    assert_eq!(ship.launches(), 1);
    assert!(!ship.is_ready_to_launch());
    assert_eq!(ship.stage_index(), 0);
    assert!(!ship.committed);
}

#[test]
fn one_ship_cannot_reach_two_worlds() {
    let mut campaign = super::campaign();
    build_seed_ship(&mut campaign);
    place_launch_driver(&mut campaign);
    assert!(campaign.launch_seed_ship(1));
    // Standing on Venus, whose yard is empty.
    assert!(!campaign.launch_seed_ship(3));
    assert!(!campaign.is_colonized(3));
}

#[test]
fn a_campaign_with_worlds_left_is_not_over() {
    let mut campaign = super::campaign();
    build_seed_ship(&mut campaign);
    // A finished ship, but four untouched worlds to send it to.
    assert!(campaign.current().seed_ship.is_ready_to_launch());
    assert!(!campaign.is_complete());
}

#[test]
fn every_world_taken_and_a_ship_with_nowhere_to_go_ends_it() {
    let mut campaign = super::campaign();
    for index in 0..PLANET_COUNT {
        campaign.colonize(index);
    }
    // Every world taken, but nothing built to leave on.
    assert!(!campaign.is_complete());

    build_seed_ship(&mut campaign);
    assert!(campaign.is_complete());
}

#[test]
fn the_ending_counts_the_whole_campaign_not_just_this_world() {
    let mut campaign = super::campaign();
    build_seed_ship(&mut campaign);
    place_launch_driver(&mut campaign);
    assert!(campaign.launch_seed_ship(1));

    assert_eq!(campaign.total_launches(), 1);
    assert!(campaign.total_structures() >= 2, "two Cores at least");
    assert!(campaign.total_time_played() >= 0.0);
}

#[test]
fn a_launch_at_an_already_colonized_world_is_refused() {
    let mut campaign = super::campaign();
    build_seed_ship(&mut campaign);
    place_launch_driver(&mut campaign);
    assert!(!campaign.launch_seed_ship(STARTING_PLANET));
    // The ship is still on the pad.
    assert!(campaign.current().seed_ship.is_ready_to_launch());
}

#[test]
fn a_finished_ship_cannot_leave_without_a_powered_mass_driver() {
    let mut campaign = super::campaign();
    build_seed_ship(&mut campaign);

    assert!(!campaign.launch_seed_ship(1));
    assert!(!campaign.is_colonized(1));
    assert_eq!(campaign.current_index(), STARTING_PLANET);

    place_launch_driver(&mut campaign);
    assert!(campaign.launch_seed_ship(1));
}
