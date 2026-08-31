use super::*;
use crate::engine::{BuildingType, GridPos};

mod directives;
mod launch;
mod research;
mod telemetry;
mod tutorial;
mod worlds;

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
