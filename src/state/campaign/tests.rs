use super::*;
use crate::engine::{BuildingType, GridPos};

mod directives;

fn campaign() -> Campaign {
    Campaign::new(GameConfig::default(), 42)
}

#[test]
fn a_new_campaign_starts_on_mars_with_nothing_else_colonized() {
    let campaign = campaign();
    assert_eq!(campaign.current_index(), STARTING_PLANET);
    assert_eq!(campaign.current().name, "Mars");
    assert_eq!(
        campaign.colonized_flags(),
        [false, false, true, false, false]
    );
}

#[test]
fn travelling_is_refused_for_a_world_the_swarm_has_not_reached() {
    let mut campaign = campaign();
    assert!(!campaign.travel_to(0));
    assert_eq!(campaign.current_index(), STARTING_PLANET);
}

#[test]
fn a_planet_left_behind_keeps_everything_it_had() {
    let mut campaign = campaign();
    let core = campaign.current().grid.find_core().unwrap();
    let drill = GridPos::new(core.x + 1, core.y);
    campaign.current_mut().grid.reveal_around(drill, 1);
    campaign.current_mut().select_building(BuildingType::Drill);
    assert!(campaign.current_mut().try_place_building(drill));
    campaign.current_mut().resources.minerals = 123.0;

    assert!(campaign.colonize(0));
    assert!(campaign.travel_to(0));
    assert_eq!(campaign.current().name, "Mercury");
    // The new world is untouched...
    assert!(campaign
        .current()
        .grid
        .find_buildings(BuildingType::Drill)
        .is_empty());

    assert!(campaign.travel_to(STARTING_PLANET));
    // ...and Mars is exactly as it was left.
    assert_eq!(campaign.current().resources.minerals, 123.0);
    assert!(campaign
        .current()
        .grid
        .get(drill)
        .unwrap()
        .building
        .is_some());
}

#[test]
fn colonizing_twice_does_not_wipe_the_world() {
    let mut campaign = campaign();
    assert!(campaign.colonize(4));
    campaign.travel_to(4);
    campaign.current_mut().resources.minerals = 77.0;
    campaign.travel_to(STARTING_PLANET);

    assert!(!campaign.colonize(4));
    campaign.travel_to(4);
    assert_eq!(campaign.current().resources.minerals, 77.0);
}

#[test]
fn every_world_is_built_from_its_own_entry_in_the_data() {
    let mut campaign = campaign();
    for index in 0..PLANET_COUNT {
        campaign.colonize(index);
    }
    for index in 0..PLANET_COUNT {
        campaign.travel_to(index);
        let def = crate::data::game_data().planet(index);
        let planet = campaign.current();
        assert_eq!(planet.name, def.name);
        assert_eq!(planet.grid.width, def.width);
        assert_eq!(planet.grid.height, def.height);
    }
}

#[test]
fn a_world_can_refuse_a_building_research_has_already_unlocked() {
    let mut campaign = campaign();
    campaign.colonize(1); // Venus: no wind, nothing to burn.
    campaign.travel_to(1);

    let planet = campaign.current_mut();
    planet.unlock_building(BuildingType::WindTurbine);
    planet.unlock_building(BuildingType::Drill);

    assert!(planet.is_building_banned(BuildingType::WindTurbine));
    assert!(!planet.is_building_unlocked(BuildingType::WindTurbine));
    // Everything else it has researched still works.
    assert!(planet.is_building_unlocked(BuildingType::Drill));

    // And it cannot be selected, let alone built.
    planet.select_building(BuildingType::WindTurbine);
    assert_ne!(planet.selected_building, Some(BuildingType::WindTurbine));
}

/// The world at `index`, generated and travelled to.
fn world(index: usize) -> PlanetState {
    let mut campaign = campaign();
    campaign.colonize(index);
    campaign.travel_to(index);
    campaign.current().clone()
}

#[test]
fn a_calm_world_reports_no_hazards() {
    let mars = world(STARTING_PLANET);
    assert_eq!(mars.acid_strength(), 0.0);
    assert_eq!(mars.freeze_strength(), 0.0);
    assert!(mars.hazard_label().is_empty());
}

#[test]
fn venus_corrodes_the_network_and_leaves_everything_else_alone() {
    let mut venus = world(1);
    assert!(venus.acid_strength() > 0.0);
    assert!(venus.hazard_label().contains("ACID"));

    let core = venus.grid.find_core().unwrap();
    let conduit = GridPos::new(core.x + 1, core.y);
    let drill = GridPos::new(core.x, core.y + 1);
    for pos in [conduit, drill] {
        venus.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        venus.grid.reveal_around(pos, 1);
    }
    venus.unlock_building(BuildingType::Conduit);
    venus.select_building(BuildingType::Conduit);
    assert!(venus.try_place_building(conduit));
    venus.select_building(BuildingType::Drill);
    assert!(venus.try_place_building(drill));

    for _ in 0..60 {
        venus.step(1.0, false);
    }

    let acid_at = |state: &PlanetState, pos: GridPos| {
        state
            .grid
            .get(pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .acid_wear
    };
    // The conduit carries the network, so the acid goes for it.
    assert!(acid_at(&venus, conduit) > acid_at(&venus, drill));
}

#[test]
fn a_shield_generator_protects_the_run_beside_it_and_not_the_far_one() {
    let mut venus = world(1);
    venus.resources.minerals = 10_000.0;
    venus.resources.energy = 10_000.0;
    venus.config.resources.max_energy = 10_000.0;
    venus.unlock_building(BuildingType::Conduit);
    venus.unlock_building(BuildingType::ShieldGenerator);

    let core = venus.grid.find_core().unwrap();
    venus.grid.reveal_around(core, 16);

    // One conduit beside the Core, another well outside a shield's reach.
    let sheltered = GridPos::new(core.x + 1, core.y);
    let exposed = GridPos::new(core.x, core.y + 8);
    let shield = GridPos::new(core.x + 1, core.y + 1);
    for pos in [sheltered, exposed, shield] {
        venus.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
    }
    venus.select_building(BuildingType::Conduit);
    assert!(venus.try_place_building(sheltered));
    assert!(venus.try_place_building(exposed));
    venus.select_building(BuildingType::ShieldGenerator);
    assert!(venus.try_place_building(shield));
    venus.grid.update_power_grid();

    for _ in 0..60 {
        venus.step(1.0, false);
    }

    let acid_at = |pos: GridPos| {
        venus
            .grid
            .get(pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .acid_wear
    };
    assert!(
        acid_at(sheltered) < acid_at(exposed),
        "sheltered conduit took {} , exposed took {}",
        acid_at(sheltered),
        acid_at(exposed)
    );
}

/// The map draws coverage from `coverage_radius`, and the simulation
/// applies it from its own constant. If those ever disagree the player is
/// being lied to, so pin them to each other at the boundary.
#[test]
fn the_drawn_coverage_radius_is_the_one_the_acid_respects() {
    let mut venus = world(1);
    venus.resources.minerals = 10_000.0;
    venus.resources.energy = 10_000.0;
    venus.config.resources.max_energy = 10_000.0;
    venus.unlock_building(BuildingType::Conduit);
    venus.unlock_building(BuildingType::ShieldGenerator);

    let radius = venus
        .coverage_radius(BuildingType::ShieldGenerator)
        .expect("shield generators cover an area");

    // Beside the Core, so the shield is actually powered: an unpowered one
    // protects nothing, which is a rule worth not tripping over silently.
    let core = venus.grid.find_core().unwrap();
    let shield = GridPos::new(core.x, core.y + 1);
    let inside = GridPos::new(shield.x + radius, shield.y);
    let outside = GridPos::new(shield.x + radius + 1, shield.y);
    venus.grid.reveal_around(shield, 16);
    for pos in [shield, inside, outside] {
        venus.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
    }
    venus.select_building(BuildingType::ShieldGenerator);
    assert!(venus.try_place_building(shield));
    venus.select_building(BuildingType::Conduit);
    assert!(venus.try_place_building(inside));
    assert!(venus.try_place_building(outside));
    venus.grid.update_power_grid();

    for _ in 0..60 {
        venus.step(1.0, false);
    }

    let acid_at = |pos: GridPos| {
        venus
            .grid
            .get(pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .acid_wear
    };
    assert!(
        acid_at(inside) < acid_at(outside),
        "the tile at exactly the drawn radius took {} and the one past it took {}",
        acid_at(inside),
        acid_at(outside)
    );
}

#[test]
fn ceramic_plating_holds_the_acid_off() {
    let bare = world(1);
    let mut plated = world(1);
    plated
        .research
        .unlocked_techs
        .push("ceramic_plating".to_string());
    plated.refresh_stats();

    assert!(plated.acid_strength() < bare.acid_strength());
    assert!(plated.acid_strength() > 0.0, "plating is not immunity");
}

/// A world with a short conduit run and a drill on the end of it, with an
/// optional Heater Node beside the run. Returns the speed of a drone once
/// it is actually walking.
fn moving_drone_speed(index: usize, heater: bool) -> f32 {
    let mut campaign = campaign();
    campaign.colonize(index);
    campaign.travel_to(index);
    let state = campaign.current_mut();
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    state.config.resources.max_energy = 10_000.0;
    state.unlock_building(BuildingType::Conduit);
    state.unlock_building(BuildingType::HeaterNode);

    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 12);
    for step in 1..=4 {
        let pos = GridPos::new(core.x + step, core.y);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.select_building(BuildingType::Conduit);
        assert!(state.try_place_building(pos));
    }
    if heater {
        let pos = GridPos::new(core.x + 2, core.y + 1);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.select_building(BuildingType::HeaterNode);
        assert!(state.try_place_building(pos));
    }
    let drill = GridPos::new(core.x + 5, core.y);
    state.grid.get_mut(drill).unwrap().terrain = crate::engine::TerrainType::Empty;
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));
    state.grid.update_power_grid();

    for _ in 0..400 {
        state.step(0.05, false);
        let drone = &state.drones.drones()[0];
        if drone.state == crate::engine::DroneState::MovingToCore {
            return drone.speed;
        }
    }
    panic!("no drone ever set out");
}

#[test]
fn the_cold_slows_drones_and_a_heater_node_thaws_the_run() {
    let saturn = world(4);
    assert!(saturn.freeze_strength() > 0.0);
    assert!(saturn.hazard_label().contains("FREEZE"));

    let frozen = moving_drone_speed(4, false);
    let heated = moving_drone_speed(4, true);
    assert!(
        heated > frozen,
        "heated run moved at {heated}, frozen run at {frozen}"
    );
}

#[test]
fn drones_move_slower_on_a_frozen_world_than_a_temperate_one() {
    let temperate = moving_drone_speed(STARTING_PLANET, false);
    let frozen = moving_drone_speed(4, false);
    assert!(
        frozen < temperate,
        "frozen drones moved at {frozen}, temperate at {temperate}"
    );
}

/// The default autosave cadence, as the settings ship it.
const AUTOSAVE: f32 = 60.0;

#[test]
fn a_fresh_campaign_is_not_immediately_due_for_a_save() {
    let campaign = campaign();
    assert!(!campaign.due_for_autosave(AUTOSAVE));
}

#[test]
fn enough_world_time_makes_a_save_due() {
    let mut campaign = campaign();
    for _ in 0..61 {
        campaign.update_directive(1.0);
    }
    assert!(campaign.due_for_autosave(AUTOSAVE));

    campaign.mark_saved();
    assert!(!campaign.due_for_autosave(AUTOSAVE));
    assert!(campaign.current().save_notice_timer > 0.0);
}

#[test]
fn a_shorter_interval_from_settings_saves_sooner() {
    let mut campaign = campaign();
    for _ in 0..11 {
        campaign.update_directive(1.0);
    }
    assert!(
        !campaign.due_for_autosave(AUTOSAVE),
        "saved far too eagerly"
    );
    assert!(campaign.due_for_autosave(10.0), "the setting was ignored");
}

#[test]
fn a_nonsense_interval_does_not_turn_into_a_save_every_tick() {
    let mut campaign = campaign();
    campaign.update_directive(0.5);
    assert!(!campaign.due_for_autosave(0.0));
    assert!(!campaign.due_for_autosave(-100.0));
}

#[test]
fn a_paused_campaign_never_becomes_due() {
    let mut campaign = campaign();
    campaign.current_mut().toggle_pause();
    // `advance` returns no ticks while paused, so nothing calls the
    // directive tick and the save clock never moves.
    for _ in 0..600 {
        let ticks = campaign.current_mut().advance(0.1, false);
        campaign.update_directive(ticks as f32 * crate::state::TICK_SECONDS);
    }
    assert!(!campaign.due_for_autosave(AUTOSAVE));
}

#[test]
fn arriving_somewhere_says_what_the_world_is() {
    let mut campaign = campaign();
    assert_eq!(campaign.current().arrival_notice_timer, 0.0);

    campaign.colonize(0);
    assert!(campaign.travel_to(0));

    let planet = campaign.current();
    assert!(planet.arrival_notice_timer > 0.0);
    assert_eq!(
        planet.arrival_line(),
        crate::data::game_data().planet(0).arrival
    );
    assert!(!planet.arrival_line().is_empty());
}

#[test]
fn worlds_differ_in_what_they_are_made_of() {
    let mut campaign = campaign();
    campaign.colonize(0);

    let count_forest = |campaign: &Campaign| {
        campaign
            .current()
            .grid
            .iter_tiles()
            .filter(|(_, tile)| tile.terrain == crate::engine::TerrainType::Forest)
            .count()
    };

    // Mars has forests in its weights; Mercury has none at all.
    let mars = count_forest(&campaign);
    campaign.travel_to(0);
    let mercury = count_forest(&campaign);

    assert!(mars > 0, "Mars generated no forest");
    assert_eq!(mercury, 0, "Mercury generated forest it has no weight for");
}

#[test]
fn each_world_generates_its_own_terrain() {
    let mut campaign = campaign();
    campaign.colonize(0);
    let mars: Vec<_> = campaign
        .current()
        .grid
        .iter_tiles()
        .map(|(_, tile)| tile.terrain)
        .collect();
    campaign.travel_to(0);
    let mercury: Vec<_> = campaign
        .current()
        .grid
        .iter_tiles()
        .map(|(_, tile)| tile.terrain)
        .collect();
    assert_ne!(mars, mercury);
}

#[test]
fn the_same_campaign_seed_lays_out_the_same_worlds() {
    let mut first = Campaign::new(GameConfig::default(), 7);
    let mut second = Campaign::new(GameConfig::default(), 7);
    first.colonize(3);
    second.colonize(3);
    first.travel_to(3);
    second.travel_to(3);

    let terrain = |campaign: &Campaign| -> Vec<_> {
        campaign
            .current()
            .grid
            .iter_tiles()
            .map(|(_, tile)| tile.terrain)
            .collect()
    };
    assert_eq!(terrain(&first), terrain(&second));
}

/// Finish the current world's Seed Ship the slow way, through the sim.
///
/// The later stages are gated on research, so the fixture has to have done
/// that research the same as a player would.
fn build_seed_ship(campaign: &mut Campaign) {
    for stage in &crate::data::game_data().seed_ship.stages {
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

#[test]
fn an_untouched_world_cannot_be_reached_without_a_ship() {
    let mut campaign = campaign();
    assert!(!campaign.current().seed_ship.is_ready_to_launch());
    assert!(!campaign.launch_seed_ship(0));
    assert!(!campaign.is_colonized(0));
    assert_eq!(campaign.current_index(), STARTING_PLANET);
}

#[test]
fn launching_colonizes_the_target_and_carries_the_swarm_there() {
    let mut campaign = campaign();
    build_seed_ship(&mut campaign);

    assert!(campaign.launch_seed_ship(1));
    assert!(campaign.is_colonized(1));
    assert_eq!(campaign.current_index(), 1);
    assert_eq!(campaign.current().name, "Venus");
}

#[test]
fn the_ship_is_spent_by_the_launch() {
    let mut campaign = campaign();
    build_seed_ship(&mut campaign);
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
    let mut campaign = campaign();
    build_seed_ship(&mut campaign);
    assert!(campaign.launch_seed_ship(1));
    // Standing on Venus, whose yard is empty.
    assert!(!campaign.launch_seed_ship(3));
    assert!(!campaign.is_colonized(3));
}

/// A world with a powered drill beside its Core, ready to produce.
fn put_a_drill_on(campaign: &mut Campaign, index: usize) {
    let here = campaign.current_index();
    campaign.travel_to(index);
    let planet = campaign.current_mut();
    planet.resources.minerals = 0.0;
    planet.config.resources.base_mineral_cap = 100_000.0;
    let core = planet.grid.find_core().unwrap();
    let drill = GridPos::new(core.x + 1, core.y);
    planet.grid.get_mut(drill).unwrap().terrain = crate::engine::TerrainType::Empty;
    planet.grid.reveal_around(drill, 1);
    planet.resources.minerals = 1_000.0;
    planet.select_building(BuildingType::Drill);
    assert!(planet.try_place_building(drill));
    planet.grid.update_power_grid();
    planet.resources.minerals = 0.0;
    campaign.travel_to(here);
}

#[test]
fn a_newly_colonized_world_already_knows_what_the_swarm_knows() {
    let mut campaign = campaign();
    campaign
        .research
        .unlocked_techs
        .push("efficient_drills".to_string());
    campaign.sync_research();

    assert!(campaign.colonize(0));
    campaign.travel_to(0);
    assert!(
        campaign
            .current()
            .stats
            .multiplier(crate::engine::StatId::DrillOutput)
            > 1.0,
        "the new world landed on starting-tier research"
    );
}

#[test]
fn research_reaches_every_world_not_just_the_one_underfoot() {
    let mut campaign = campaign();
    campaign.colonize(0);
    campaign.colonize(3);

    campaign
        .research
        .unlocked_techs
        .push("efficient_drills".to_string());
    campaign.sync_research();

    for index in [STARTING_PLANET, 0, 3] {
        campaign.travel_to(index);
        let planet = campaign.current();
        assert!(
            planet.stats.multiplier(crate::engine::StatId::DrillOutput) > 1.0,
            "world {index} is still running on stale research"
        );
    }
}

#[test]
fn a_building_unlocked_by_research_is_unlocked_everywhere() {
    let mut campaign = campaign();
    campaign.colonize(0);
    campaign
        .research
        .unlocked_techs
        .push("wind_power".to_string());
    campaign.sync_research();

    campaign.travel_to(0);
    assert!(campaign
        .current()
        .is_building_researched(BuildingType::WindTurbine));
}

#[test]
fn a_save_that_kept_research_on_the_planet_is_adopted_once() {
    let mut campaign = campaign();
    // What an older save looks like: the planet knows, the campaign does not.
    campaign.research.unlocked_techs.clear();
    campaign
        .current_mut()
        .research
        .unlocked_techs
        .push("efficient_drills".to_string());

    campaign.adopt_planet_research();

    assert!(campaign
        .research
        .unlocked_techs
        .iter()
        .any(|id| id == "efficient_drills"));
    assert!(
        campaign
            .current()
            .stats
            .multiplier(crate::engine::StatId::DrillOutput)
            > 1.0
    );

    // And it does not clobber research the campaign already has.
    let mut newer = campaign;
    newer.research.unlocked_techs = vec!["wind_power".to_string()];
    newer.adopt_planet_research();
    assert_eq!(
        newer.research.unlocked_techs,
        vec!["wind_power".to_string()]
    );
}

#[test]
fn a_world_left_behind_keeps_working() {
    let mut campaign = campaign();
    campaign.colonize(0);
    put_a_drill_on(&mut campaign, 0);
    assert_eq!(campaign.current_index(), STARTING_PLANET);
    assert_eq!(campaign.stockpile(0), Some(0.0));

    for _ in 0..600 {
        campaign.update_background(0.1);
    }

    let stocked = campaign.stockpile(0).unwrap();
    assert!(stocked > 0.0, "the world froze the moment it was left");
}

#[test]
fn the_world_in_front_of_the_player_is_not_run_twice() {
    let mut campaign = campaign();
    put_a_drill_on(&mut campaign, STARTING_PLANET);
    let before = campaign.current().time_played;

    for _ in 0..600 {
        campaign.update_background(0.1);
    }

    assert_eq!(
        campaign.current().time_played,
        before,
        "the foreground world was stepped by the background pass"
    );
}

#[test]
fn background_time_does_not_pile_up_into_a_stutter() {
    let mut campaign = campaign();
    campaign.colonize(0);
    put_a_drill_on(&mut campaign, 0);

    // One enormous frame: the backlog is dropped, not replayed.
    campaign.update_background(10_000.0);
    let after_stall = campaign.stockpile(0).unwrap();

    campaign.update_background(1.0);
    let after_normal = campaign.stockpile(0).unwrap();
    assert!(
        after_normal - after_stall < after_stall.max(1.0) * 10.0,
        "a stalled frame turned into an unbounded catch-up"
    );
}

#[test]
fn a_campaign_with_worlds_left_is_not_over() {
    let mut campaign = campaign();
    build_seed_ship(&mut campaign);
    // A finished ship, but four untouched worlds to send it to.
    assert!(campaign.current().seed_ship.is_ready_to_launch());
    assert!(!campaign.is_complete());
}

#[test]
fn every_world_taken_and_a_ship_with_nowhere_to_go_ends_it() {
    let mut campaign = campaign();
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
    let mut campaign = campaign();
    build_seed_ship(&mut campaign);
    assert!(campaign.launch_seed_ship(1));

    assert_eq!(campaign.total_launches(), 1);
    assert!(campaign.total_structures() >= 2, "two Cores at least");
    assert!(campaign.total_time_played() >= 0.0);
}

#[test]
fn a_launch_at_an_already_colonized_world_is_refused() {
    let mut campaign = campaign();
    build_seed_ship(&mut campaign);
    assert!(!campaign.launch_seed_ship(STARTING_PLANET));
    // The ship is still on the pad.
    assert!(campaign.current().seed_ship.is_ready_to_launch());
}
