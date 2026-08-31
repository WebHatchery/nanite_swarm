//! Planet-owned power refreshes.

use crate::engine::{BuildingType, GridPos, StatId};

use super::game_state::PlanetState;

impl PlanetState {
    /// The live power contribution of one standing building.
    ///
    /// The build palette intentionally shows a definition's nominal draw, but
    /// an inspector is looking at a building that may be dusty, overclocked,
    /// on standby, or affected by research. Keep this calculation in the
    /// planet layer so its answer follows the same modifiers as the global
    /// power balance.
    pub fn building_power_delta(&self, pos: GridPos) -> Option<f32> {
        let tile = self.grid.get(pos)?;
        let building = tile.building.as_ref()?;
        if !building.powered {
            return Some(0.0);
        }

        let dust_response = self.resolved_dust_response();
        let def = self.data().building(building.building_type.id());
        let mut generation = if def.generates_power {
            let mut output = def.power_generation;
            if def.uses_efficiency {
                output *= building.efficiency;
            }
            output * building.dust_power_generation_multiplier_with(&dust_response)
        } else {
            0.0
        };

        // Biomass harvesters are fuel generators rather than grid generators
        // in the building definitions. Their live output still belongs in the
        // selected-building readout, or the inspector would report 0/s while
        // the top-bar power balance is receiving their fuel bonus.
        if building.building_type == BuildingType::BiomassHarvester
            && tile.terrain == crate::engine::TerrainType::Forest
            && tile.biomass_amount > 0.0
            && !building.is_dust_stalled_with(&dust_response)
        {
            generation += self.config.resources.biomass_power_output
                * building.dust_power_generation_multiplier_with(&dust_response);
        }

        let raw_consumption = def.power_consumption
            * building.dust_power_consumption_multiplier_with(&dust_response)
            * building.power_demand_multiplier()
            + building.dust_power_leak_with(&dust_response);
        let consumption = self.stats.apply(StatId::PowerConsumption, raw_consumption);
        let focus_tax = if self.factory_focus.affects(building.building_type) {
            def.power_consumption
                * 0.15
                * building.dust_power_consumption_multiplier_with(&dust_response)
                * building.power_demand_multiplier()
        } else {
            0.0
        };

        Some(generation - consumption - focus_tax)
    }

    /// Recompute connectivity with the dust thresholds supplied by the
    /// planet's research, rather than the engine's standalone defaults.
    pub(crate) fn refresh_power_grid(&mut self) {
        let dust_response = self.resolved_dust_response();
        self.grid.update_power_grid_with(&dust_response);
        self.refresh_biomass_power_bonus();
    }
}
