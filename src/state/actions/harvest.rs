//! Terrain harvesting and forest-filter conversion actions.

use super::*;

impl PlanetState {
    /// Try to harvest terrain at position.
    pub fn try_harvest_terrain(&mut self, pos: GridPos) -> bool {
        let data_context = self.data.clone();
        let data = data_context.data();
        let Some(tile) = self.grid.get(pos) else {
            return false;
        };
        let terrain = tile.terrain;
        let Some((minerals, biomass)) = self.harvest_rewards_at(pos) else {
            return false;
        };

        // Apply harvest.
        if let Some(tile) = self.grid.get_mut(pos) {
            tile.terrain = terrain.harvested(data);
            match terrain {
                TerrainType::Mountain => {
                    tile.mountain_harvested = true;
                }
                TerrainType::Forest => {
                    tile.forest_cleared = true;
                    tile.biomass_amount = 0.0;
                    self.forest_harvested_count += 1;
                }
                _ => {}
            }
        }

        self.resources.minerals += minerals;
        self.resources.biomass += biomass;
        self.spawn_harvest_burst(pos);
        self.emit_audio(crate::state::audio::AudioEvent::Harvest);

        self.notifications.info(format!(
            "Harvested {:.0} minerals and {:.0} biomass",
            minerals, biomass
        ));

        true
    }

    /// Resolve the reward shown by the terrain inspector and paid by harvest.
    ///
    /// Research and surveyed feature effects modify the terrain's ordinary
    /// reward without changing the tile's eventual harvested state.
    pub fn harvest_rewards_at(&self, pos: GridPos) -> Option<(f32, f32)> {
        let data = self.data();
        let tile = self.grid.get(pos)?;
        if !tile.revealed || !tile.terrain.is_harvestable(data) || tile.building.is_some() {
            return None;
        }
        let (minerals, biomass) = tile.terrain.harvest_rewards(data);
        let yield_multiplier = self.stats.apply(crate::engine::StatId::HarvestYield, 1.0)
            * self.feature_harvest_multiplier_at(pos).unwrap_or(1.0);
        Some((minerals * yield_multiplier, biomass * yield_multiplier))
    }

    /// Check if terrain at position can be harvested.
    pub fn can_harvest(&self, pos: GridPos) -> bool {
        if let Some(tile) = self.grid.get(pos) {
            tile.revealed && tile.terrain.is_harvestable(self.data()) && tile.building.is_none()
        } else {
            false
        }
    }

    pub fn try_convert_forest_to_filter(&mut self, pos: GridPos) -> bool {
        if let Some(tile) = self.grid.get(pos) {
            if !tile.revealed || tile.terrain != TerrainType::Forest || tile.building.is_some() {
                return false;
            }
        } else {
            return false;
        }

        if let Some(tile) = self.grid.get_mut(pos) {
            tile.terrain = TerrainType::Rough;
            tile.filter = true;
            tile.forest_cleared = true;
            tile.biomass_amount = 0.0;
            self.forest_harvested_count += 1;
            let reduction = (1.0 - self.config.upkeep.filter_multiplier.clamp(0.0, 1.0)) * 100.0;
            self.emit_audio(crate::state::audio::AudioEvent::UiConfirm);
            self.notifications.info(format!(
                "Forest converted: dust filter -{reduction:.0}% nearby"
            ));
            return true;
        }

        false
    }
}
