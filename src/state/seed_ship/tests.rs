use super::*;
use crate::data::GameConfig;
use crate::engine::{BuildingType, GridPos};

fn state() -> PlanetState {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    state.config.resources.base_mineral_cap = 100_000.0;
    state.resources.minerals = 10_000.0;
    state.resources.data = 10_000.0;
    state.resources.biomass = 10_000.0;
    state.resources.alloy = 10_000.0;
    state.resources.components = 10_000.0;
    state
}

/// The same world, with the research the later stages are gated on.
fn researched_state() -> PlanetState {
    let mut state = state();
    for stage in &crate::data::game_data().seed_ship.stages {
        if let Some(tech) = stage.requires.as_deref() {
            state.research.unlocked_techs.push(tech.to_string());
        }
    }
    state.refresh_stats();
    state
}

fn intake() -> SeedShipCost {
    crate::data::game_data().seed_ship.intake_per_second
}

/// A drill beside the Core and a Smelter on its other side, both powered.
fn state_with_smelter() -> (PlanetState, GridPos, GridPos) {
    let mut state = state();
    state.resources.alloy = 0.0;
    let core = state.grid.find_core().unwrap();
    let drill = GridPos::new(core.x + 1, core.y);
    let smelter = GridPos::new(core.x, core.y + 1);
    for pos in [drill, smelter] {
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.grid.reveal_around(pos, 1);
    }
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(smelter));
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));
    state.grid.update_power_grid();
    (state, drill, smelter)
}

/// The whole chain in one go: ore is carried to a Smelter, refined into
/// alloy, and the alloy is carried on to the Core by the Smelter's own
/// drones. Nothing teleports at either end.
#[test]
fn ore_is_carried_in_refined_and_the_alloy_carried_out() {
    let (mut state, _drill, smelter) = state_with_smelter();

    // Long enough for a full load of alloy to be made and then delivered.
    for _ in 0..400 {
        state.step(0.1, false);
    }

    assert!(state.resources.alloy > 0.0, "no alloy reached the Core");
    assert!(state.alloy_rate() > 0.0);
    // Ore reached the hopper rather than the global pool...
    assert!(state.input_buffers.contains_key(&(smelter.x, smelter.y)));
    // ...and the Smelter has its own crew to carry the alloy out.
    assert!(!state.drones.drones_at(smelter).is_empty());
}

#[test]
fn alloy_waits_on_the_smelter_pad_until_a_drone_takes_it() {
    let (mut state, _drill, smelter) = state_with_smelter();

    // Short enough that a full load has not been collected yet.
    for _ in 0..100 {
        state.step(0.1, false);
    }

    let on_pad = state
        .output_buffers
        .get(&(smelter.x, smelter.y))
        .copied()
        .unwrap_or(0.0);
    assert!(on_pad > 0.0, "the Smelter refined nothing");
    assert_eq!(
        state.resources.alloy, 0.0,
        "alloy reached the pool without being carried"
    );
}

#[test]
fn a_smelter_nobody_delivers_to_stays_idle_however_full_the_pool_is() {
    let mut state = state();
    state.resources.alloy = 0.0;
    state.resources.minerals = 100_000.0;
    // Powered, but with no drill anywhere to send it anything.
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x, core.y + 1);
    state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(pos));
    state.grid.update_power_grid();

    for _ in 0..100 {
        state.step(0.1, false);
    }
    assert_eq!(state.resources.alloy, 0.0);
}

#[test]
fn ore_goes_to_the_smelter_before_it_goes_to_the_pool() {
    let (mut state, _drill, smelter) = state_with_smelter();
    state.resources.minerals = 0.0;
    state.config.resources.base_mineral_cap = 100_000.0;

    // One delivery's worth of time.
    for _ in 0..40 {
        state.step(0.1, false);
    }

    let refined = state
        .output_buffers
        .get(&(smelter.x, smelter.y))
        .copied()
        .unwrap_or(0.0);
    assert!(refined > 0.0, "nothing reached the smelter");
    assert_eq!(
        state.resources.minerals, 0.0,
        "ore went to the pool while the smelter had room"
    );
}

#[test]
fn an_unpowered_smelter_refines_nothing() {
    let mut state = state();
    state.resources.alloy = 0.0;
    // Far from the Core, so it never gets power.
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 6, core.y + 6);
    state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(pos));
    state.grid.update_power_grid();

    for _ in 0..100 {
        state.step(0.1, false);
    }
    assert_eq!(state.resources.alloy, 0.0);
    assert_eq!(state.alloy_rate(), 0.0);
}

#[test]
fn a_smelter_with_no_minerals_left_simply_slows_down() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(pos));
    state.grid.update_power_grid();

    state.resources.minerals = 0.0;
    state.resources.alloy = 0.0;
    for _ in 0..100 {
        state.step(0.1, false);
    }

    assert_eq!(state.resources.alloy, 0.0);
    assert!(state.resources.minerals >= 0.0, "minerals went negative");
}

#[test]
fn the_last_stages_of_the_ship_cannot_be_paid_without_alloy() {
    let stages = &crate::data::game_data().seed_ship.stages;
    let alloy_stages = stages.iter().filter(|s| s.cost.alloy > 0.0).count();
    assert!(alloy_stages >= 2, "the chain has no sink");

    let mut ship = SeedShip::default();
    let mut resources = Resources {
        minerals: 100_000.0,
        data: 100_000.0,
        biomass: 100_000.0,
        energy: 0.0,
        alloy: 0.0,
        components: 100_000.0,
    };
    for _ in 0..stages.len() {
        ship.absorb(&mut resources, intake(), 10_000.0);
    }
    // Everything else is paid, so the ship stalls on the first alloy stage.
    assert!(!ship.is_complete());
    assert!(ship.stage().unwrap().cost.alloy > 0.0);

    resources.alloy = 100_000.0;
    for _ in 0..stages.len() {
        ship.absorb(&mut resources, intake(), 10_000.0);
    }
    assert!(ship.is_complete());
}

#[test]
fn flight_stages_cannot_be_paid_without_factory_components() {
    let stages = &crate::data::game_data().seed_ship.stages;
    assert!(
        stages
            .iter()
            .filter(|stage| stage.cost.components > 0.0)
            .count()
            >= 2
    );

    let mut ship = SeedShip::default();
    let mut resources = Resources {
        minerals: 100_000.0,
        data: 100_000.0,
        biomass: 100_000.0,
        energy: 0.0,
        alloy: 100_000.0,
        components: 0.0,
    };
    for _ in 0..stages.len() {
        ship.absorb(&mut resources, intake(), 10_000.0);
    }
    assert!(!ship.is_complete());
    assert!(ship.stage().unwrap().cost.components > 0.0);

    resources.components = 100_000.0;
    for _ in 0..stages.len() {
        ship.absorb(&mut resources, intake(), 10_000.0);
    }
    assert!(ship.is_complete());
}

#[test]
fn the_skyline_grows_with_the_ship_and_not_before_it() {
    let mut state = researched_state();
    assert!(
        !state.seed_ship.has_broken_ground(),
        "a yard nobody has committed to is standing"
    );
    assert_eq!(state.seed_ship.built_fraction(), 0.0);

    state.toggle_seed_ship_commitment();
    assert!(state.seed_ship.has_broken_ground());

    let mut last = 0.0;
    for _ in 0..400 {
        state.update_seed_ship(1.0);
        let now = state.seed_ship.built_fraction();
        assert!(now >= last, "the ship shrank: {} then {}", last, now);
        assert!((0.0..=1.0).contains(&now), "out of range: {}", now);
        last = now;
    }
    assert!(state.seed_ship.is_complete());
    assert_eq!(state.seed_ship.built_fraction(), 1.0);
}

#[test]
fn a_stage_part_paid_counts_for_part_of_the_ship() {
    let mut state = researched_state();
    state.toggle_seed_ship_commitment();
    // Part-way into the first stage, and no further.
    state.update_seed_ship(1.0);
    let fraction = state.seed_ship.built_fraction();
    assert_eq!(state.seed_ship.stage_index(), 0);
    assert!(
        fraction > 0.0 && fraction < 1.0 / state.seed_ship.stage_count() as f32,
        "a part-paid first stage counted as {}",
        fraction
    );
}

#[test]
fn a_launched_yard_is_bare_ground_again() {
    let mut state = researched_state();
    state.toggle_seed_ship_commitment();
    for _ in 0..400 {
        state.update_seed_ship(1.0);
    }
    assert_eq!(state.seed_ship.built_fraction(), 1.0);
    state.seed_ship.mark_launched();
    assert_eq!(state.seed_ship.built_fraction(), 0.0);
    assert!(!state.seed_ship.has_broken_ground());
}

#[test]
fn the_ship_cannot_be_finished_on_minerals_alone() {
    let mut state = state();
    // Every resource in the world, and no research past the start.
    state.toggle_seed_ship_commitment();
    for _ in 0..2_000 {
        state.update_seed_ship(1.0);
    }

    assert!(
        !state.seed_ship.is_complete(),
        "the tech tree can be skipped entirely"
    );
    assert!(state.seed_ship_blocked_by().is_some());
}

#[test]
fn a_blocked_yard_takes_nothing_rather_than_banking_it() {
    let mut state = state();
    state.toggle_seed_ship_commitment();
    // Clear the first stage, which needs no research.
    while state.seed_ship.stage_index() == 0 {
        state.update_seed_ship(1.0);
    }
    assert!(state.seed_ship_blocked_by().is_some(), "stage two is gated");

    let minerals = state.resources.minerals;
    for _ in 0..100 {
        state.update_seed_ship(1.0);
    }
    assert_eq!(
        state.resources.minerals, minerals,
        "the yard ate resources it could not use"
    );
    assert_eq!(state.seed_ship.stage_fraction(), 0.0);
}

#[test]
fn the_research_that_unblocks_a_stage_gets_it_moving() {
    let mut state = state();
    state.toggle_seed_ship_commitment();
    while state.seed_ship.stage_index() == 0 {
        state.update_seed_ship(1.0);
    }

    let required = state
        .seed_ship
        .blocked_by(&state.research.unlocked_techs)
        .expect("stage two is gated")
        .to_string();
    state.research.unlocked_techs.push(required);
    state.refresh_stats();

    assert!(state.seed_ship_blocked_by().is_none());
    state.update_seed_ship(1.0);
    assert!(state.seed_ship.stage_fraction() > 0.0);
}

#[test]
fn the_shipped_stages_declare_only_modifiers_the_game_can_read() {
    let stages = &crate::data::game_data().seed_ship.stages;
    let with_boons = stages.iter().filter(|s| !s.modifiers.is_empty()).count();
    assert!(with_boons >= 3, "the ship is still all cost and no payoff");
    for stage in stages {
        for modifier in &stage.modifiers {
            assert!(
                crate::engine::parse_modifier(modifier).is_ok(),
                "stage {} declares an unreadable modifier",
                stage.id
            );
        }
        if !stage.modifiers.is_empty() {
            assert!(!stage.boon.is_empty(), "stage {} does not say", stage.id);
        }
    }
}

#[test]
fn a_standing_stage_works_for_the_world_it_stands_on() {
    let mut state = state();
    let before = state.stats.multiplier(crate::engine::StatId::DrillOutput);

    // Finish the first stage, which pays the drills back.
    state.toggle_seed_ship_commitment();
    while state.seed_ship.stage_index() == 0 {
        state.update_seed_ship(1.0);
    }

    assert_eq!(state.seed_ship.standing_stages().len(), 1);
    assert!(
        state.stats.multiplier(crate::engine::StatId::DrillOutput) > before,
        "the finished stage did nothing"
    );
}

#[test]
fn the_yards_advantages_leave_with_the_ship() {
    let mut state = researched_state();
    state.toggle_seed_ship_commitment();
    for _ in 0..2_000 {
        state.update_seed_ship(1.0);
    }
    assert!(state.seed_ship.is_complete());
    let boosted = state.stats.multiplier(crate::engine::StatId::DrillOutput);
    assert!(boosted > 1.0);

    state.seed_ship.mark_launched();
    state.refresh_stats();

    assert!(state.seed_ship.standing_stages().is_empty());
    assert!(
        state.stats.multiplier(crate::engine::StatId::DrillOutput) < boosted,
        "the ship left but its yard did not"
    );
}

#[test]
fn a_new_ship_starts_on_the_first_stage_and_is_not_complete() {
    let ship = SeedShip::default();
    assert_eq!(ship.stage_index(), 0);
    assert!(ship.stage_count() >= 4);
    assert!(!ship.is_complete());
    assert_eq!(ship.stage_fraction(), 0.0);
    assert!(!ship.committed);
}

#[test]
fn nothing_is_taken_until_the_swarm_commits() {
    let mut state = state();
    let before = state.resources.minerals;
    state.update_seed_ship(10.0);
    assert_eq!(state.resources.minerals, before);
    assert_eq!(state.seed_ship.stage_fraction(), 0.0);
}

#[test]
fn intake_is_capped_per_second_rather_than_taken_all_at_once() {
    let mut ship = SeedShip::default();
    let mut resources = Resources {
        minerals: 10_000.0,
        ..Default::default()
    };

    ship.absorb(&mut resources, intake(), 1.0);

    assert_eq!(resources.minerals, 10_000.0 - intake().minerals);
    assert_eq!(ship.progress().minerals, intake().minerals);
    assert_eq!(ship.stage_index(), 0);
}

#[test]
fn a_stage_completes_only_once_every_resource_is_paid() {
    let mut ship = SeedShip::default();
    let cost = ship.stage().unwrap().cost;
    let mut resources = Resources {
        minerals: cost.minerals,
        data: 0.0,
        biomass: 0.0,
        energy: 0.0,
        alloy: 0.0,
        components: 0.0,
    };

    // One very long step: intake is capped by what the stage still needs.
    let finished = ship.absorb(&mut resources, intake(), 10_000.0);

    // Stage one asks only for minerals, so this pays it off exactly.
    assert!(finished);
    assert_eq!(ship.stage_index(), 1);
    assert_eq!(ship.progress(), StageProgress::default());
    assert!(resources.minerals < 1.0);
}

#[test]
fn a_stage_that_needs_data_waits_for_it() {
    let mut ship = SeedShip::default();
    let mut resources = Resources {
        minerals: 100_000.0,
        ..Default::default()
    };
    // Clear stage one, which is minerals only.
    assert!(ship.absorb(&mut resources, intake(), 10_000.0));
    assert_eq!(ship.stage_index(), 1);
    assert!(ship.stage().unwrap().cost.data > 0.0);

    // Minerals alone cannot finish stage two.
    assert!(!ship.absorb(&mut resources, intake(), 10_000.0));
    assert_eq!(ship.stage_index(), 1);
    assert!(ship.stage_fraction() > 0.0 && ship.stage_fraction() < 1.0);

    resources.data = 10_000.0;
    assert!(ship.absorb(&mut resources, intake(), 10_000.0));
    assert_eq!(ship.stage_index(), 2);
}

#[test]
fn a_committed_swarm_eventually_finishes_the_whole_ship() {
    let mut state = researched_state();
    state.toggle_seed_ship_commitment();
    assert!(state.seed_ship.committed);

    for _ in 0..2_000 {
        state.update_seed_ship(1.0);
    }

    assert!(state.seed_ship.is_complete());
    assert!(state.seed_ship.stage().is_none());
    assert_eq!(state.seed_ship.stage_fraction(), 1.0);
    // A finished ship stops drawing on the pool.
    assert!(!state.seed_ship.committed);
}

#[test]
fn finishing_the_ship_unlocks_the_achievement() {
    let mut state = researched_state();
    state.toggle_seed_ship_commitment();
    for _ in 0..2_000 {
        state.update_seed_ship(1.0);
    }
    assert!(state.achievements.is_unlocked("seed_ship"));
}

#[test]
fn a_finished_ship_cannot_be_re_committed() {
    let mut state = state();
    state.toggle_seed_ship_commitment();
    for _ in 0..2_000 {
        state.update_seed_ship(1.0);
    }
    state.toggle_seed_ship_commitment();
    assert!(!state.seed_ship.committed);
}
