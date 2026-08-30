//! Reusable working-world fixtures for deterministic UI captures.

use crate::{engine, state, Game};

/// Put the infrastructure needed to leave a captured world through its real
/// launch rules. Some worlds ask for a power floor or a heater before they
/// accept a Mass Driver, so the ending scene cannot just edit the building
/// count and call the ship finished.
pub(super) fn seed_capture_launch_rail(planet: &mut state::PlanetState) {
    use engine::{BuildingType, GridPos, TerrainType};

    let Some(core) = planet.grid.find_core() else {
        return;
    };
    let constraints = planet
        .data()
        .planet(planet.planet_index)
        .constraints
        .clone();
    let required_offsets = [(0, 1), (0, -1), (-1, 0), (1, 0)];

    for (required_index, id) in constraints.required_buildings.iter().enumerate() {
        let Some(building_type) = BuildingType::from_id(id) else {
            continue;
        };
        if building_type == BuildingType::MassDriver {
            continue;
        }
        let (x, y) = required_offsets
            .get(required_index)
            .copied()
            .unwrap_or((0, 1));
        let pos = GridPos::new(core.x + x, core.y + y);
        prepare_capture_tile(planet, pos, TerrainType::Empty);
        planet.unlock_building(building_type);
        planet.select_building(building_type);
        assert!(
            planet.try_place_building(pos),
            "capture fixture could not place required {:?}",
            building_type
        );
    }

    if planet.power_generation() < constraints.minimum_power_generation {
        if !planet.is_building_banned(BuildingType::WindTurbine) {
            let required = ((constraints.minimum_power_generation - planet.power_generation())
                / 5.0)
                .ceil()
                .max(1.0) as usize;
            for pos in [
                GridPos::new(core.x - 1, core.y),
                GridPos::new(core.x, core.y - 1),
                GridPos::new(core.x, core.y + 2),
            ]
            .into_iter()
            .take(required)
            {
                prepare_capture_tile(planet, pos, TerrainType::Empty);
                planet.unlock_building(BuildingType::WindTurbine);
                planet.select_building(BuildingType::WindTurbine);
                assert!(
                    planet.try_place_building(pos),
                    "capture fixture could not place a power source"
                );
            }
        } else if !planet.is_building_banned(BuildingType::BiomassHarvester) {
            let biomass_tile = GridPos::new(core.x - 1, core.y);
            prepare_capture_tile(planet, biomass_tile, TerrainType::Forest);
            if let Some(tile) = planet.grid.get_mut(biomass_tile) {
                tile.biomass_amount = planet.config.resources.forest_biomass;
            }
            planet.unlock_building(BuildingType::BiomassHarvester);
            planet.select_building(BuildingType::BiomassHarvester);
            assert!(
                planet.try_place_building(biomass_tile),
                "capture fixture could not place biomass power"
            );
            // Biomass power is calculated during upkeep, so give the harvester
            // one real tick before the Mass Driver checks the power floor.
            planet.grid.update_power_grid();
            planet.step(state::TICK_SECONDS, false);
        } else {
            let power_source = match planet.active_planet_condition() {
                Some("acid") => BuildingType::GeothermalTap,
                Some("cold") => BuildingType::FusionReactor,
                _ => return,
            };
            let source_tile = GridPos::new(core.x - 1, core.y);
            prepare_capture_tile(planet, source_tile, TerrainType::Empty);
            planet.unlock_building(power_source);
            planet.select_building(power_source);
            assert!(
                planet.try_place_building(source_tile),
                "capture fixture could not place {:?} power",
                power_source
            );
        }
    }

    let driver = GridPos::new(core.x + 1, core.y);
    prepare_capture_tile(planet, driver, TerrainType::Empty);
    planet.unlock_building(BuildingType::MassDriver);
    planet.select_building(BuildingType::MassDriver);
    assert!(
        planet.try_place_building(driver),
        "capture fixture could not place its Mass Driver"
    );
    planet.grid.update_power_grid();
}

fn prepare_capture_tile(
    planet: &mut state::PlanetState,
    pos: engine::GridPos,
    terrain: engine::TerrainType,
) {
    if let Some(tile) = planet.grid.get_mut(pos) {
        tile.terrain = terrain;
        tile.building = None;
        tile.revealed = true;
        tile.biomass_amount = 0.0;
        tile.forest_cleared = false;
    }
    planet.grid.reveal_around(pos, 1);
}

impl Game {
    /// A Mass Driver under a standing order, with cargo on both ends.
    pub(super) fn seed_shipping_scene(&mut self, target: usize) {
        use engine::{BuildingType, GridPos, ResourceType};

        let planet = self.campaign.current_mut();
        let Some(core) = planet.grid.find_core() else {
            return;
        };
        planet.config.resources.max_energy = 1_000.0;
        planet.resources.energy = 1_000.0;
        planet.unlock_building(BuildingType::WindTurbine);
        planet.select_building(BuildingType::WindTurbine);
        for offset in [-1, 1] {
            let pos = GridPos::new(core.x, core.y + offset);
            if let Some(tile) = planet.grid.get_mut(pos) {
                tile.terrain = engine::TerrainType::Empty;
            }
            planet.grid.reveal_around(pos, 1);
            planet.try_place_building(pos);
        }

        let driver = GridPos::new(core.x - 1, core.y);
        if let Some(tile) = planet.grid.get_mut(driver) {
            tile.terrain = engine::TerrainType::Empty;
        }
        planet.grid.reveal_around(driver, 1);
        planet.unlock_building(BuildingType::MassDriver);
        planet.select_building(BuildingType::MassDriver);
        planet.try_place_building(driver);
        planet.grid.update_power_grid();

        planet.export = Some(state::ExportOrder {
            target,
            cargo: ResourceType::Minerals,
            ..state::ExportOrder::default()
        });
        let capacity = planet.config.mass_driver.pod_capacity;
        planet
            .pod_loads
            .insert((driver.x, driver.y), capacity * 0.45);
        let transit = state::transit_seconds(
            planet.data(),
            planet.planet_index,
            target,
            &planet.config.mass_driver.clone(),
        );
        for spent in [0.55, 0.2] {
            planet.launched_pods.push(state::Shipment {
                from: planet.planet_index,
                to: target,
                cargo: ResourceType::Minerals,
                amount: capacity,
                remaining: transit * (1.0 - spent),
                transit,
                target_pad: None,
                overflow: false,
                priority: 0,
            });
        }
        if let Some(receiver) = self.campaign.planet_mut(target) {
            if let Some(core) = receiver.grid.find_core() {
                let pad = GridPos::new(core.x, core.y + 1);
                if let Some(tile) = receiver.grid.get_mut(pad) {
                    tile.terrain = engine::TerrainType::Empty;
                }
                receiver.resources.energy = 500.0;
                receiver.config.resources.max_energy = 500.0;
                receiver.grid.reveal_around(pad, 1);
                receiver.unlock_building(BuildingType::LandingPad);
                receiver.select_building(BuildingType::LandingPad);
                receiver.try_place_building(pad);
                receiver.grid.update_power_grid();
            }
        }
        self.campaign.update_shipments(0.0);
    }

    /// A working conduit run with a drill on the end of it.
    pub(super) fn seed_logistics_scene(&mut self) {
        use engine::{BuildingType, GridPos};

        let state = self.campaign.current_mut();
        let Some(core) = state.grid.find_core() else {
            return;
        };
        state.grid.reveal_around(core, 12);
        state.resources.minerals = 500.0;
        state.resources.energy = 500.0;
        state.config.resources.max_energy = 500.0;
        state.unlock_building(BuildingType::Conduit);
        state.unlock_building(BuildingType::PowerNode);

        let mut run: Vec<GridPos> = (1..=5).map(|x| GridPos::new(core.x + x, core.y)).collect();
        run.extend((1..=4).map(|y| GridPos::new(core.x + 5, core.y - y)));
        for (index, pos) in run.iter().enumerate() {
            if let Some(tile) = state.grid.get_mut(*pos) {
                tile.terrain = engine::TerrainType::Empty;
                tile.building = None;
            }
            let piece = if index == 4 {
                BuildingType::PowerNode
            } else {
                BuildingType::Conduit
            };
            state.select_building(piece);
            state.try_place_building(*pos);
        }

        let drill = GridPos::new(core.x + 5, core.y - 5);
        if let Some(tile) = state.grid.get_mut(drill) {
            tile.terrain = engine::TerrainType::Empty;
            tile.building = None;
        }
        state.select_building(BuildingType::Drill);
        state.try_place_building(drill);
        state.grid.update_power_grid();
    }
}
