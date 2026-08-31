use super::*;

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
/// optional Heater Node beside the run. Returns the speed of a drone once it
/// is actually walking.
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
