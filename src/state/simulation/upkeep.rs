//! Dust, heat, biomass, and the building coverage queries used by the UI.

use crate::engine::{BuildingType, StatId, TerrainType};
use crate::state::game_state::PlanetState;

impl PlanetState {
    /// Update server bank data generation.
    pub(super) fn update_servers(&mut self, delta_time: f32) {
        let dust_response = self.resolved_dust_response();
        let rate = self.stats.apply(
            StatId::DataGeneration,
            self.config.resources.server_data_rate,
        );

        for server_pos in self.grid.find_buildings(BuildingType::ServerBank) {
            let Some(building) = self.grid.get(server_pos).and_then(|t| t.building.as_ref()) else {
                continue;
            };
            if !building.powered
                || building.standby
                || building.is_dust_stalled_with(&dust_response)
            {
                continue;
            }
            let heat_efficiency =
                if building.is_overheated(self.config.buildings.server_bank_heat_capacity) {
                    self.config.buildings.overheat_penalty
                } else {
                    1.0
                };
            self.resources.data += rate
                * building.dust_efficiency_with(&dust_response)
                * building.work_multiplier()
                * self.factory_focus_multiplier(BuildingType::ServerBank)
                * heat_efficiency
                * delta_time;
        }
    }

    pub(super) fn update_heat(&mut self, delta_time: f32) {
        let dust_response = self.resolved_dust_response();
        let cooling = self.config.buildings.water_cooling_rate.max(0.0);
        let capacity = self.config.buildings.server_bank_heat_capacity.max(1.0);
        let server_focus_multiplier = self.factory_focus_multiplier(BuildingType::ServerBank);
        let water_tiles: Vec<_> = self
            .grid
            .iter_tiles()
            .filter_map(|(pos, tile)| (tile.terrain == TerrainType::Water).then_some(pos))
            .collect();
        for (pos, tile) in self.grid.iter_tiles_mut() {
            let Some(building) = tile.building.as_mut() else {
                continue;
            };
            let def = crate::data::game_data().building(building.building_type.id());
            let generated = if building.building_type == BuildingType::ServerBank
                && building.powered
                && !building.is_dust_stalled_with(&dust_response)
            {
                self.config.buildings.server_bank_heat_per_second
                    * building.work_multiplier()
                    * server_focus_multiplier
            } else {
                0.0
            };
            let water_cooling = if def.water_cooling
                && water_tiles
                    .iter()
                    .any(|water| pos.distance(*water) as i32 <= 1)
            {
                cooling
            } else {
                0.0
            };
            building.heat =
                (building.heat + (generated - water_cooling) * delta_time).clamp(0.0, capacity);
        }
    }

    /// How far a building's upkeep effect reaches, if it has one.
    pub fn coverage_radius(&self, building_type: BuildingType) -> Option<i32> {
        let upkeep = &self.config.upkeep;
        match building_type {
            BuildingType::Sweeper => Some(upkeep.sweeper_radius),
            BuildingType::ShieldGenerator | BuildingType::HeaterNode => {
                Some(upkeep.hazard_counter_radius)
            }
            _ => None,
        }
    }

    /// Positions of powered, working buildings of a type.
    pub(crate) fn powered_positions(
        &self,
        building_type: BuildingType,
    ) -> Vec<crate::engine::GridPos> {
        let dust_response = self.resolved_dust_response();
        self.grid
            .find_buildings(building_type)
            .into_iter()
            .filter(|pos| {
                self.grid
                    .get(*pos)
                    .and_then(|tile| tile.building.as_ref())
                    .is_some_and(|building| {
                        building.powered
                            && !building.standby
                            && !building.is_dust_stalled_with(&dust_response)
                    })
            })
            .collect()
    }

    pub(super) fn update_dust(&mut self, delta_time: f32) {
        let upkeep = self.config.upkeep.clone();
        let dust_rate = self.stats.apply(StatId::DustAccumulation, upkeep.dust_rate);
        let acid_rate = upkeep.dust_rate * upkeep.acid_multiplier;
        let acid_strength = self.acid_strength();
        let shields = self.powered_positions(BuildingType::ShieldGenerator);
        let powered_sweepers: Vec<_> = self
            .grid
            .find_buildings(BuildingType::Sweeper)
            .into_iter()
            .filter(|pos| {
                self.grid
                    .get(*pos)
                    .and_then(|t| t.building.as_ref())
                    .is_some_and(|building| building.powered && !building.is_dust_stalled())
            })
            .collect();
        let filter_positions: Vec<_> = self
            .grid
            .iter_tiles()
            .filter_map(|(pos, tile)| tile.filter.then_some(pos))
            .collect();
        let cleared_forest_positions: Vec<_> = self
            .grid
            .iter_tiles()
            .filter_map(|(pos, tile)| tile.forest_cleared.then_some(pos))
            .collect();
        let acid_fields = crate::data::game_data()
            .planet(self.planet_index)
            .hazard_fields
            .clone();
        let grid_width = self.grid.width;
        let grid_height = self.grid.height;

        for (pos, tile) in self.grid.iter_tiles_mut() {
            let Some(building) = tile.building.as_mut() else {
                continue;
            };
            let mut rate = dust_rate;
            let mut acid = 0.0;
            if acid_rate > 0.0 && building.transmits_power() {
                let sheltered = shields
                    .iter()
                    .any(|shield| pos.distance(*shield) as i32 <= upkeep.hazard_counter_radius);
                acid = acid_rate
                    * crate::state::progress::hazard_field_strength(
                        pos,
                        "acid",
                        acid_strength,
                        &acid_fields,
                        grid_width,
                        grid_height,
                    );
                if sheltered {
                    acid *= 1.0 - upkeep.hazard_counter_strength;
                }
            }

            if filter_positions
                .iter()
                .any(|filter_pos| pos.distance(*filter_pos) as i32 <= upkeep.filter_radius)
            {
                rate *= upkeep.filter_multiplier;
            }
            if cleared_forest_positions
                .iter()
                .any(|cleared_pos| pos.distance(*cleared_pos) as i32 <= upkeep.pollution_radius)
            {
                rate *= upkeep.pollution_multiplier;
            }
            rate *= building.dust_accumulation_multiplier();

            let clean_rate = if powered_sweepers
                .iter()
                .any(|sweeper_pos| pos.distance(*sweeper_pos) as i32 <= upkeep.sweeper_radius)
            {
                upkeep.sweeper_rate
            } else {
                0.0
            };
            building.dust =
                (building.dust + rate * delta_time - clean_rate * delta_time).clamp(0.0, 100.0);
            building.acid_wear = (building.acid_wear + acid * delta_time).clamp(0.0, 100.0);
        }
    }

    pub(super) fn update_biomass_harvesters(&mut self, delta_time: f32) {
        let output = self.config.resources.biomass_power_output;
        let rate = self.config.resources.biomass_consumption_rate;
        let mut power_bonus = 0.0;

        for (_, tile) in self.grid.iter_tiles_mut() {
            let Some(building) = tile.building.as_mut() else {
                continue;
            };
            if building.building_type != BuildingType::BiomassHarvester
                || tile.terrain != TerrainType::Forest
                || tile.biomass_amount <= 0.0
                || !building.powered
                || building.is_dust_stalled()
            {
                continue;
            }

            let available = tile.biomass_amount;
            if rate <= 0.0 {
                continue;
            }
            let consume = (rate * delta_time).min(available);
            tile.biomass_amount = (available - consume).max(0.0);
            self.resources.biomass += consume;
            let fraction = if rate * delta_time > 0.0 {
                consume / (rate * delta_time)
            } else {
                0.0
            };
            power_bonus += output * fraction * building.dust_power_generation_multiplier();

            if tile.biomass_amount <= 0.0 {
                tile.terrain = TerrainType::Empty;
                tile.forest_cleared = true;
                tile.filter = false;
            }
        }

        self.biomass_power_bonus = power_bonus;
    }
}
