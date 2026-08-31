use super::PlanetState;
use crate::data::GameConfig;
use crate::engine::{BuildingType, GridPos};

fn state() -> PlanetState {
    PlanetState::new(2, 7, GameConfig::default())
}

fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() <= eps
}

#[test]
fn offline_simulation_uses_hibernation_rate() {
    let mut state = PlanetState {
        battery_seconds: 4.0 * 60.0 * 60.0,
        ..Default::default()
    };

    let offline = 6.0 * 60.0 * 60.0;
    state.apply_offline_progress(offline);

    let expected_sim = (4.0 * 60.0 * 60.0) + (2.0 * 60.0 * 60.0) * 0.1;
    assert!(approx_eq(state.last_offline_simulated, expected_sim, 0.5));
    assert!(approx_eq(state.last_offline_seconds, offline, 0.5));
    assert!(state.battery_seconds <= 0.0);
    assert!(state.offline_notice_timer > 0.0);
}

#[test]
fn apply_offline_progress_is_noop_for_zero_or_negative_duration() {
    let mut state = PlanetState::default();
    state.apply_offline_progress(0.0);
    assert_eq!(state.last_offline_seconds, 0.0);
    assert_eq!(state.last_offline_simulated, 0.0);
}

#[test]
fn core_is_always_unlocked_others_require_explicit_unlock() {
    let mut state = PlanetState::default();
    assert!(state.is_building_unlocked(BuildingType::Core));
    assert!(!state.is_building_unlocked(BuildingType::Conduit));
    state.unlock_building(BuildingType::Conduit);
    assert!(state.is_building_unlocked(BuildingType::Conduit));
}

#[test]
fn hostile_arrival_guidance_walks_the_power_route_prerequisites() {
    let mut state = PlanetState::new(1, 7, GameConfig::default());
    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap RESEARCH for Power Grid.")
    );

    state.research.unlocked_techs.push("power_grid".to_string());
    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap RESEARCH for Geothermal Tapping.")
    );

    state
        .research
        .unlocked_techs
        .push("geothermal_tapping".to_string());
    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap BUILD, then place Geothermal Tap.")
    );
}

#[test]
fn acid_arrival_guidance_continues_to_the_corrosion_countermeasure() {
    let mut state = PlanetState::new(1, 7, GameConfig::default());
    for tech in ["power_grid", "geothermal_tapping"] {
        state.research.unlocked_techs.push(tech.to_string());
    }
    let core = state.grid.find_core().unwrap();
    let source = GridPos::new(core.x + 1, core.y);
    let tile = state.grid.get_mut(source).unwrap();
    tile.terrain = crate::engine::TerrainType::Empty;
    tile.revealed = true;
    assert!(state
        .grid
        .place_building(source, BuildingType::GeothermalTap));

    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap RESEARCH for Ceramic Plating.")
    );
}

#[test]
fn acid_arrival_guidance_names_the_shield_after_plating_is_researched() {
    let mut state = PlanetState::new(1, 7, GameConfig::default());
    for tech in ["power_grid", "geothermal_tapping", "ceramic_plating"] {
        state.research.unlocked_techs.push(tech.to_string());
    }
    let core = state.grid.find_core().unwrap();
    let geothermal = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(geothermal).unwrap().terrain = crate::engine::TerrainType::Empty;
    state
        .grid
        .place_building(geothermal, BuildingType::GeothermalTap);

    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap BUILD, then place Shield Generator.")
    );
}

#[test]
fn a_cold_world_keeps_heater_placement_visible_after_fusion_is_ready() {
    let mut state = PlanetState::new(3, 7, GameConfig::default());
    for tech in ["power_grid", "heater_nodes", "fusion_reactors"] {
        state.research.unlocked_techs.push(tech.to_string());
    }
    state.unlock_building(BuildingType::HeaterNode);
    state.unlock_building(BuildingType::FusionReactor);
    let core = state.grid.find_core().unwrap();
    let source = GridPos::new(core.x + 1, core.y);
    let tile = state.grid.get_mut(source).unwrap();
    tile.terrain = crate::engine::TerrainType::Empty;
    tile.revealed = true;
    assert!(state
        .grid
        .place_building(source, BuildingType::FusionReactor));

    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap BUILD, then place Heater Node.")
    );
}

#[test]
fn cold_arrival_guidance_starts_with_the_first_fusion_prerequisite() {
    let state = PlanetState::new(4, 7, GameConfig::default());
    assert_eq!(
        state.arrival_guidance().as_deref(),
        Some("Next: tap RESEARCH for Power Grid.")
    );
}

#[test]
fn calm_worlds_without_a_construction_rule_need_no_arrival_guidance() {
    let state = PlanetState::new(2, 7, GameConfig::default());
    assert_eq!(state.arrival_guidance(), None);
}

#[test]
fn unlock_building_does_not_duplicate_entries() {
    let mut state = PlanetState::default();
    state.unlock_building(BuildingType::Storage);
    state.unlock_building(BuildingType::Storage);
    assert_eq!(
        state
            .unlocked_buildings
            .iter()
            .filter(|b| **b == BuildingType::Storage)
            .count(),
        1
    );
}

#[test]
fn cancel_interaction_modes_disarms_every_pending_map_action() {
    let mut state = state();
    let core = state.grid.find_core().expect("generated Core");
    let tile = GridPos::new(core.x + 1, core.y);
    state.selected_building = Some(BuildingType::Drill);
    state.selected_tile = Some(tile);
    state.demolish_mode = true;
    state.relocation_source = Some(core);
    state.box_select_mode = true;
    state.box_select_start = Some(core);
    state.box_selected = vec![core, tile];
    state.purge_armed = Some(tile);
    state.bulk_purge_armed = true;
    state.drag_last_pos = Some(tile);
    state.smart_drag_source = Some(core);
    state.touch_camera_active = true;
    state.touch_route_active = true;
    state.touch_gesture_routed = true;
    state.blueprint.push(crate::state::BlueprintEntry {
        offset: (0, 0),
        building_type: BuildingType::Drill,
        overclocked: false,
        input_priority: false,
        standby: false,
    });
    let blueprint_len = state.blueprint.len();

    state.cancel_interaction_modes();

    assert_eq!(state.selected_building, None);
    assert_eq!(state.selected_tile, None);
    assert!(!state.demolish_mode);
    assert_eq!(state.relocation_source, None);
    assert!(!state.box_select_mode);
    assert_eq!(state.box_select_start, None);
    assert!(state.box_selected.is_empty());
    assert_eq!(state.purge_armed, None);
    assert!(!state.bulk_purge_armed);
    assert_eq!(state.drag_last_pos, None);
    assert_eq!(state.smart_drag_source, None);
    assert!(!state.touch_camera_active);
    assert!(!state.touch_route_active);
    assert!(!state.touch_gesture_routed);
    assert_eq!(state.blueprint.len(), blueprint_len);
}

#[test]
fn choosing_a_building_cancels_relocation_before_the_next_tap() {
    let mut state = state();
    let core = state.grid.find_core().expect("generated Core");
    state.relocation_source = Some(core);
    state.box_select_mode = true;
    state.box_select_start = Some(core);
    state.select_building(BuildingType::Drill);

    assert_eq!(state.selected_building, Some(BuildingType::Drill));
    assert_eq!(state.relocation_source, None);
    assert!(!state.box_select_mode);
    assert_eq!(state.box_select_start, None);
}

/// Every tech that claims a number in `research.json` has to move one here.
/// These are the five that shipped as no-ops.
#[test]
fn researching_storage_optimization_raises_the_mineral_cap() {
    let mut state = PlanetState::default();
    let before = state.mineral_capacity();
    state
        .research
        .unlocked_techs
        .push("storage_optimization".into());
    state.refresh_stats();
    assert_eq!(state.mineral_capacity(), before + 50.0);
}

#[test]
fn buffer_lattices_widen_every_processor_dispatch_pad() {
    let mut state = PlanetState::default();
    let before = state.processor_pad_capacity();
    state.research.unlocked_techs.push("buffer_lattices".into());
    state.refresh_stats();
    assert_eq!(state.processor_pad_capacity(), before * 1.5);
}

#[test]
fn researching_power_efficiency_lowers_consumption() {
    let mut state = PlanetState::default();
    let core = state.grid.find_core().unwrap();
    let pos = crate::engine::GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    let before = state.power_consumption();
    assert!(before > 0.0);
    state
        .research
        .unlocked_techs
        .push("power_efficiency".into());
    state.refresh_stats();
    assert!(approx_eq(state.power_consumption(), before * 0.75, 1e-4));
}

#[test]
fn live_resource_rates_follow_active_buildings_and_dust_research() {
    let mut plain = state();
    let plain_core = plain.grid.find_core().unwrap();
    plain
        .grid
        .get_mut(plain_core)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 26.0;
    let plain_data = plain.data_generation_rate();

    let mut researched = state();
    let researched_core = researched.grid.find_core().unwrap();
    researched
        .grid
        .get_mut(researched_core)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 26.0;
    researched
        .research
        .unlocked_techs
        .push("self_cleaning_servos".to_string());
    researched.refresh_stats();
    assert!(researched.data_generation_rate() > plain_data);

    let mut server_state = state();
    let server_core = server_state.grid.find_core().unwrap();
    let server = GridPos::new(server_core.x + 1, server_core.y);
    server_state.grid.reveal_around(server, 1);
    server_state.grid.get_mut(server).unwrap().terrain = crate::engine::TerrainType::Empty;
    server_state.unlock_building(BuildingType::ServerBank);
    server_state.select_building(BuildingType::ServerBank);
    assert!(server_state.try_place_building(server));
    assert!(server_state.data_generation_rate() > state().data_generation_rate());

    let drill = GridPos::new(researched_core.x + 1, researched_core.y);
    researched.grid.reveal_around(drill, 1);
    researched.grid.get_mut(drill).unwrap().terrain = crate::engine::TerrainType::Empty;
    researched.select_building(BuildingType::Drill);
    assert!(researched.try_place_building(drill));
    researched
        .grid
        .get_mut(drill)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 26.0;
    assert_eq!(researched.mineral_output_rate(), 5.0);

    let biomass = GridPos::new(researched_core.x - 1, researched_core.y);
    researched.grid.reveal_around(biomass, 1);
    let biomass_tile = researched.grid.get_mut(biomass).unwrap();
    biomass_tile.terrain = crate::engine::TerrainType::Forest;
    biomass_tile.biomass_amount = researched.config.resources.forest_biomass;
    researched.unlock_building(BuildingType::BiomassHarvester);
    researched.select_building(BuildingType::BiomassHarvester);
    assert!(researched.try_place_building(biomass));
    assert_eq!(researched.biomass_harvest_rate(), 1.0);
}

#[test]
fn selected_building_power_rate_follows_live_state() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let drill = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(drill, 1);
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));

    assert!(approx_eq(
        state.building_power_delta(drill).unwrap(),
        -2.0,
        1e-4
    ));

    state
        .grid
        .get_mut(drill)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 75.0;
    assert!(approx_eq(
        state.building_power_delta(drill).unwrap(),
        -2.4,
        1e-4
    ));

    state.factory_focus = super::super::FactoryFocus::Extraction;
    assert!(approx_eq(
        state.building_power_delta(drill).unwrap(),
        -2.76,
        1e-4
    ));

    state
        .research
        .unlocked_techs
        .push("power_efficiency".to_string());
    state.refresh_stats();
    // Research lowers the normal draw; the focus tax remains the declared
    // operating surcharge, matching the global power balance.
    assert!(approx_eq(
        state.building_power_delta(drill).unwrap(),
        -2.16,
        1e-4
    ));

    state
        .grid
        .get_mut(drill)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .standby = true;
    assert!(approx_eq(
        state.building_power_delta(drill).unwrap(),
        0.0,
        1e-4
    ));
}

#[test]
fn selected_processor_power_rate_includes_overclock_and_focus_tax() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let smelter = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(smelter, 1);
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(smelter));

    state
        .grid
        .get_mut(smelter)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .overclocked = true;
    assert!(approx_eq(
        state.building_power_delta(smelter).unwrap(),
        -7.0,
        1e-4
    ));

    state.factory_focus = super::super::FactoryFocus::Refining;
    assert!(approx_eq(
        state.building_power_delta(smelter).unwrap(),
        -8.05,
        1e-4
    ));
}

#[test]
fn biomass_harvester_power_rate_reads_live_fuel_output() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let forest = GridPos::new(core.x - 1, core.y);
    state.grid.reveal_around(forest, 1);
    let tile = state.grid.get_mut(forest).unwrap();
    tile.terrain = crate::engine::TerrainType::Forest;
    tile.biomass_amount = state.config.resources.forest_biomass;
    state.unlock_building(BuildingType::BiomassHarvester);
    state.select_building(BuildingType::BiomassHarvester);
    assert!(state.try_place_building(forest));

    assert!(approx_eq(
        state.building_power_delta(forest).unwrap(),
        state.config.resources.biomass_power_output,
        1e-4
    ));
    state
        .grid
        .get_mut(forest)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 75.0;
    assert!(approx_eq(
        state.building_power_delta(forest).unwrap(),
        state.config.resources.biomass_power_output * 0.7,
        1e-4
    ));
}

#[test]
fn researching_drone_capacity_grows_existing_drones_too() {
    let mut state = PlanetState::default();
    let core = state.grid.find_core().unwrap();
    let pos = crate::engine::GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);
    let before = state.drones.drone_capacity;

    state.research.unlocked_techs.push("drone_capacity".into());
    state.refresh_stats();

    assert_eq!(state.drones.drone_capacity, before * 2.0);
    assert_eq!(state.drones.drones()[0].capacity, before * 2.0);
}

#[test]
fn researching_efficient_drills_and_advanced_research_speeds_production() {
    let mut plain = PlanetState::default();
    let mut upgraded = PlanetState::default();
    upgraded
        .research
        .unlocked_techs
        .extend(["efficient_drills".into(), "advanced_research".into()]);
    upgraded.refresh_stats();

    for state in [&mut plain, &mut upgraded] {
        let core = state.grid.find_core().unwrap();
        let pos = crate::engine::GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);
        state.resources.minerals = 0.0;
        // Both runs would otherwise clamp at the storage cap.
        state.config.resources.base_mineral_cap = 100_000.0;
        for _ in 0..300 {
            state.step(0.1, false);
        }
    }

    assert!(upgraded.resources.minerals > plain.resources.minerals);
    assert!(upgraded.resources.data > plain.resources.data);
}

#[test]
fn mineral_capacity_grows_with_storage_buildings() {
    let mut state = PlanetState::default();
    let base = state.mineral_capacity();
    let core = state.grid.find_core().unwrap();
    let pos = crate::engine::GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::Storage);
    state.select_building(BuildingType::Storage);
    state.try_place_building(pos);
    assert!(state.mineral_capacity() > base);
}

#[test]
fn every_resource_reports_the_same_cap_used_by_simulation() {
    let state = state();
    assert_eq!(
        state.resource_capacity(crate::engine::ResourceType::Energy),
        50.0
    );
    assert_eq!(
        state.resource_capacity(crate::engine::ResourceType::Minerals),
        state.mineral_capacity()
    );
    for resource in [
        crate::engine::ResourceType::Data,
        crate::engine::ResourceType::Biomass,
        crate::engine::ResourceType::Alloy,
        crate::engine::ResourceType::Components,
    ] {
        assert_eq!(state.resource_capacity(resource), 1_000.0);
    }
}

#[test]
fn offline_progress_respects_live_storage_caps() {
    let mut state = state();
    state.config.offline.max_resource_gain = 10_000.0;
    state.config.resources.max_energy = 10_000.0;
    state.config.resources.base_mineral_cap = 100_000.0;
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    let core = state.grid.find_core().unwrap();
    for (building, pos) in [
        (BuildingType::Smelter, GridPos::new(core.x + 1, core.y)),
        (BuildingType::Assembler, GridPos::new(core.x, core.y + 1)),
    ] {
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.grid.reveal_around(pos, 1);
        state.unlock_building(building);
        state.select_building(building);
        assert!(state.try_place_building(pos));
    }
    state.grid.update_power_grid();
    state.resources.data = 999.0;
    state.resources.biomass = 999.0;
    state.resources.alloy = 999.0;
    state.resources.components = 999.0;
    state.apply_offline_progress(10_000.0);

    for resource in [
        crate::engine::ResourceType::Data,
        crate::engine::ResourceType::Biomass,
        crate::engine::ResourceType::Alloy,
        crate::engine::ResourceType::Components,
    ] {
        assert!(
            state.resources.get(resource) <= state.resource_capacity(resource),
            "offline progress exceeded the {:?} cap",
            resource
        );
    }
    assert_eq!(state.resources.data, 1_000.0);
    assert_eq!(state.resources.alloy, 1_000.0);
    assert_eq!(state.resources.components, 1_000.0);
}

#[test]
fn battery_time_left_converts_seconds_to_hours_and_minutes() {
    let state = PlanetState {
        battery_seconds: 3661.0, // 1h 1m 1s
        ..Default::default()
    };
    assert_eq!(state.battery_time_left(), (1, 1));
}

#[test]
fn next_event_names_the_timer_that_will_be_reached_first() {
    let mut state = state();
    state.power_collapse_shutdown = 12.0;
    state.research_lock_timer = 8.0;
    state.export_cooldown = 20.0;

    let event = state.next_interesting_event().expect("an event is pending");
    assert_eq!(event.label, "Research unlock");
    assert_eq!(event.seconds, 8.0);
}

#[test]
fn fast_forward_to_next_event_stops_at_the_selected_timer() {
    let mut state = state();
    state.export_cooldown = 2.5;
    let before = state.time_played;

    assert!(state.fast_forward_to_next_event());
    assert!((state.time_played - before - 2.5).abs() < 0.001);
    assert!(state.export_cooldown < 0.001);
    assert!(state.next_interesting_event().is_none());
}

#[test]
fn research_modifies_the_declared_dust_response_thresholds() {
    let mut bare = state();
    let mut researched = state();
    researched
        .research
        .unlocked_techs
        .push("self_cleaning_servos".to_string());
    researched.refresh_stats();

    let base = bare.resolved_dust_response();
    let improved = researched.resolved_dust_response();
    assert!(improved.efficiency_threshold > base.efficiency_threshold);
    assert!(improved.speed_threshold > base.speed_threshold);
    assert!(researched
        .stat_sources(crate::engine::StatId::DustSpeedThreshold)
        .iter()
        .any(|source| source == "Self-Cleaning Servos"));
    bare.config.upkeep.dust_response.efficiency_threshold = 80.0;
    assert!(bare.resolved_dust_response().efficiency_threshold > 0.0);
}

#[test]
fn an_achievement_announces_itself_once_and_not_again() {
    let mut state = PlanetState::default();
    assert!(state.notifications.is_empty());

    state.announce_achievement("first_drill");
    assert_eq!(state.notifications.count(), 1);
    let message = state.notifications.get_notifications()[0].message.clone();
    assert!(message.contains("First Drill"), "{message}");

    // Ticking on does not re-announce something already unlocked.
    state.announce_achievement("first_drill");
    assert_eq!(state.notifications.count(), 1);
}

#[test]
fn toasts_fade_in_real_time_even_while_the_world_is_paused() {
    let mut state = PlanetState::default();
    state.announce_achievement("first_drill");
    // `opacity` only moves in the toast's last second, so the timer itself
    // is the thing to watch.
    let before = state.notifications.get_notifications()[0].progress();

    state.toggle_pause();
    for _ in 0..20 {
        assert_eq!(state.advance(0.1, false), 0, "the world moved while paused");
    }

    let after = state.notifications.get_notifications()[0].progress();
    assert!(
        after > before,
        "the toast froze with the world: {before} then {after}"
    );
}

#[test]
fn achievements_progress_unlocks_first_drill_and_power_surplus() {
    let mut state = PlanetState::default();
    let (unlocked_before, total) = state.achievements_progress();
    assert_eq!(unlocked_before, 0);

    let core = state.grid.find_core().unwrap();
    let pos = crate::engine::GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    let (unlocked_after, total_after) = state.achievements_progress();
    assert!(unlocked_after > unlocked_before);
    assert_eq!(total_after, total);
}
