use super::*;
use crate::engine::{BuildingType, GridPos};

mod directives;
mod launch;
mod research;

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
fn offline_research_spends_foreground_data_and_keeps_campaign_state_synced() {
    let mut campaign = campaign();
    campaign.research.current_research = Some("power_grid".to_string());
    campaign.current_mut().resources.data = 50.0;
    campaign.sync_research();

    let (spent, completed) = campaign.apply_offline_research(1.0);

    assert_eq!(spent, 5.0);
    assert!(!completed);
    assert_eq!(campaign.research.research_progress, 5.0);
    assert_eq!(campaign.current().resources.data, 45.0);
    assert_eq!(
        campaign.current().research.research_progress,
        campaign.research.research_progress
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
fn the_solar_map_reads_live_telemetry_for_colonized_worlds() {
    let mut campaign = campaign();
    assert!(campaign.world_telemetry(0).is_none());

    assert!(campaign.colonize(0));
    let fresh = campaign.world_telemetry(0).unwrap();
    assert_eq!(fresh.minerals, 50.0);
    assert_eq!(fresh.data, 0.0);
    assert_eq!(fresh.biomass, 0.0);
    assert_eq!(fresh.alloy, 0.0);
    assert_eq!(fresh.components, 0.0);
    assert_eq!(fresh.data_rate, 0.25);
    assert_eq!(fresh.biomass_rate, 0.0);
    assert_eq!(fresh.structures, 1);
    assert_eq!(fresh.drones, 0);
    assert_eq!(fresh.alerts, 0);
    assert_eq!(fresh.alert_kind, WorldAlert::Nominal);

    let mercury = campaign.planet_mut(0).unwrap();
    mercury.resources.minerals = 123.0;
    mercury.resources.data = 23.0;
    mercury.resources.biomass = 7.0;
    mercury.resources.alloy = 41.0;
    mercury.resources.components = 13.0;
    mercury.power_balance = -1.0;
    let live = campaign.world_telemetry(0).unwrap();
    assert_eq!(live.minerals, 123.0);
    assert_eq!(live.data, 23.0);
    assert_eq!(live.biomass, 7.0);
    assert_eq!(live.alloy, 41.0);
    assert_eq!(live.components, 13.0);
    assert_eq!(live.power_balance, -1.0);
    assert_eq!(live.alerts, 1);
    assert_eq!(live.alert_kind, WorldAlert::Power);
}

#[test]
fn the_solar_map_prioritizes_collapse_over_secondary_world_alerts() {
    let mut campaign = campaign();
    let world = campaign.current_mut();
    world.power_balance = -1.0;
    world.battery_seconds = 0.0;
    world.power_collapse_shutdown = 5.0;

    let telemetry = campaign.world_telemetry(STARTING_PLANET).unwrap();
    assert_eq!(telemetry.alerts, 3);
    assert_eq!(telemetry.alert_kind, WorldAlert::Collapse);
}

#[test]
fn the_solar_map_reports_upkeep_degradation_on_a_remote_world() {
    let mut campaign = campaign();
    assert!(campaign.colonize(0));
    let world = campaign.planet_mut(0).expect("colonized Mercury");
    let core = world.grid.find_core().expect("generated Core");
    world
        .grid
        .get_mut(core)
        .expect("core tile")
        .building
        .as_mut()
        .expect("core building")
        .dust = world.resolved_dust_response().efficiency_threshold;

    let telemetry = campaign.world_telemetry(0).unwrap();
    assert_eq!(telemetry.alerts, 1);
    assert_eq!(telemetry.alert_kind, WorldAlert::Maintenance);
}

#[test]
fn every_world_is_built_from_its_own_entry_in_the_data() {
    let mut campaign = campaign();
    for index in 0..PLANET_COUNT {
        campaign.colonize(index);
    }
    for index in 0..PLANET_COUNT {
        campaign.travel_to(index);
        let def = campaign.data().data().planet(index);
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

#[test]
fn hostile_worlds_offer_their_own_power_route() {
    let mut campaign = campaign();
    campaign.colonize(1);
    campaign.travel_to(1);
    let venus = campaign.current_mut();
    venus.unlock_building(BuildingType::GeothermalTap);
    assert!(venus.building_condition_met(BuildingType::GeothermalTap));
    assert!(!venus.building_condition_met(BuildingType::FusionReactor));

    campaign.colonize(4);
    campaign.travel_to(4);
    let saturn = campaign.current_mut();
    saturn.unlock_building(BuildingType::FusionReactor);
    assert!(saturn.building_condition_met(BuildingType::FusionReactor));
    assert!(saturn.is_building_banned(BuildingType::BiomassHarvester));
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
fn a_successful_save_clears_the_backup_recovery_warning() {
    let mut campaign = campaign();
    campaign.current_mut().restored_from_backup = true;

    campaign.mark_saved();

    assert!(!campaign.current().restored_from_backup);
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
        crate::data::test_data().data().planet(0).arrival
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
