//! Reusable working-world fixtures for deterministic UI captures.

use crate::{engine, state, Game};

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
