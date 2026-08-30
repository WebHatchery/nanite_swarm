use super::*;
use crate::data::GameConfig;
use crate::state::ExportOrder;

/// One full drone load at the shipped drill rate: 10 minerals at 5/s.
const FILL_SECONDS: f32 = 2.0;

/// A Core with a conduit run east and a drill at the far end of it.
fn state_with_run(length: i32) -> (PlanetState, GridPos, GridPos) {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 24);
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    state.config.resources.max_energy = 10_000.0;
    state.unlock_building(BuildingType::Conduit);
    state.unlock_building(BuildingType::PowerNode);

    for step in 1..=length {
        let pos = GridPos::new(core.x + step, core.y);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        // A repeater every five tiles keeps a long run powered; power nodes
        // carry traffic too, so the route length is unaffected.
        let piece = if step % 5 == 0 {
            BuildingType::PowerNode
        } else {
            BuildingType::Conduit
        };
        state.select_building(piece);
        assert!(state.try_place_building(pos), "run piece at step {step}");
    }

    let drill = GridPos::new(core.x + length + 1, core.y);
    {
        // Ordinary ground under the drill. These tests are about routes
        // and dispatch; letting them start on whatever deposit the world
        // generator happened to lay there makes them measure geology.
        let tile = state.grid.get_mut(drill).unwrap();
        tile.terrain = crate::engine::TerrainType::Empty;
        tile.ore_richness = 1.0;
    }
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));
    state.grid.update_power_grid();

    (state, core, drill)
}

/// Loads delivered by a single drill over `seconds` of fixed-step
/// simulation, with the storage cap lifted out of the way.
fn loads_delivered(state: &mut PlanetState, seconds: f32) -> u32 {
    state.resources.minerals = 0.0;
    state.config.resources.base_mineral_cap = 100_000.0;

    let ticks = (seconds / crate::state::TICK_SECONDS) as u32;
    for _ in 0..ticks {
        state.step(crate::state::TICK_SECONDS, false);
    }

    (state.resources.minerals / state.drones.drone_capacity).round() as u32
}

/// Ore banked, rather than whole loads. Rounding to loads is too coarse to
/// see a difference that is a few seconds of drone speed.
fn ore_delivered(state: &mut PlanetState, seconds: f32) -> u32 {
    state.resources.minerals = 0.0;
    state.config.resources.base_mineral_cap = 100_000.0;

    let ticks = (seconds / crate::state::TICK_SECONDS) as u32;
    for _ in 0..ticks {
        state.step(crate::state::TICK_SECONDS, false);
    }
    state.resources.minerals.round() as u32
}

/// Snapshot of the whole harvest loop at the fixed timestep: drill cycle,
/// dispatch, travel out over the network, delivery, and the walk home. If
/// the tick length, drone speed, drill cycle or route cost changes, this
/// number moves.
#[test]
fn alloy_is_carried_to_whatever_eats_it_rather_than_always_to_the_core() {
    let (mut state, core, drill) = state_with_run(4);
    // A Smelter on the run, and a Server Bank beside the Core that wants
    // what the Smelter makes.
    let smelter = GridPos::new(core.x + 2, core.y - 1);
    let bank = GridPos::new(core.x, core.y - 1);
    for (pos, kind) in [
        (smelter, BuildingType::Smelter),
        (bank, BuildingType::ServerBank),
    ] {
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.unlock_building(kind);
        state.select_building(kind);
        assert!(
            state.try_place_building(pos),
            "could not place at {:?}",
            pos
        );
    }
    state.grid.update_power_grid();

    let delivery = state
        .delivery_for(smelter, core, ResourceType::Alloy)
        .expect("somewhere for the alloy to go");
    assert_eq!(
        delivery.0, bank,
        "alloy went to {:?} rather than to the thing that eats it",
        delivery.0
    );
}

#[test]
fn a_resource_nothing_wants_still_goes_home_to_the_core() {
    let (state, core, drill) = state_with_run(4);
    // Biomass has no consumer building at all.
    let delivery = state
        .delivery_for(drill, core, ResourceType::Biomass)
        .expect("a route home");
    assert_eq!(delivery.0, core);
}

#[test]
fn a_source_reserve_keeps_refined_cargo_on_the_core_route_until_ready() {
    let (mut state, core, drill) = state_with_run(4);
    let driver = GridPos::new(core.x - 1, core.y);
    state.grid.get_mut(driver).unwrap().terrain = crate::engine::TerrainType::Empty;
    state.unlock_building(BuildingType::MassDriver);
    assert!(state.grid.place_building(driver, BuildingType::MassDriver));
    state.grid.update_power_grid();
    state.export = Some(ExportOrder {
        cargo: ResourceType::Alloy,
        target: 1,
        reserve_source: 100.0,
        surplus_only: true,
        ..ExportOrder::default()
    });

    assert!(
        !state.consumers_of(ResourceType::Alloy).contains(&driver),
        "a held reserve must stop refined cargo bypassing the Core"
    );
    assert_eq!(
        state
            .delivery_for(drill, core, ResourceType::Alloy)
            .unwrap()
            .0,
        core,
        "refined cargo should fall back to the Core while the reserve is held"
    );

    state.resources.alloy = 101.0;
    assert!(
        state.consumers_of(ResourceType::Alloy).contains(&driver),
        "a surplus refined stockpile may feed the Mass Driver"
    );
    assert_eq!(
        state
            .delivery_for(drill, core, ResourceType::Alloy)
            .unwrap()
            .0,
        driver,
        "refined cargo may resume its freight route above the reserve"
    );
}

fn two_smelter_run() -> (PlanetState, GridPos, GridPos, GridPos, GridPos) {
    let (mut state, core, drill) = state_with_run(6);
    let near = GridPos::new(core.x + 2, core.y - 1);
    let far = GridPos::new(core.x + 4, core.y - 1);
    for pos in [near, far] {
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        assert!(state.grid.place_building(pos, BuildingType::Smelter));
    }
    state.grid.update_power_grid();
    (state, core, drill, near, far)
}

mod delivery;
mod network;
