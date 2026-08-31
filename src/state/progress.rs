use crate::data::DustResponseConfig;
use crate::engine::{BuildingType, GridPos, ResourceType, StatId};

use super::game_state::PlanetState;

mod arrival;
mod clock;
mod constraints;
mod interaction;
mod offline;

pub(crate) use constraints::hazard_field_strength;

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct OfflineReport {
    pub elapsed_seconds: f32,
    pub capped_seconds: f32,
    pub tamper_guarded: bool,
    pub minerals_gained: f32,
    pub alloy_gained: f32,
    pub components_gained: f32,
    pub data_gained: f32,
    pub power_gained: f32,
    /// Data spent by the active research job while the player was away.
    #[serde(default)]
    pub research_data_spent: f32,
    /// Whether that offline research job crossed its completion line.
    #[serde(default)]
    pub research_completed: bool,
    /// Seed Ship stages paid for while the player was away.
    #[serde(default)]
    pub seed_ship_stages_completed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NextEvent {
    pub label: &'static str,
    pub seconds: f32,
}

impl PlanetState {
    /// Research can move the response points without changing the data file
    /// that describes ordinary upkeep. Keeping this resolver on the state
    /// makes every simulation consumer read the same values.
    pub fn resolved_dust_response(&self) -> DustResponseConfig {
        let base = &self.config.upkeep.dust_response;
        DustResponseConfig {
            efficiency_threshold: self
                .stats
                .apply(StatId::DustEfficiencyThreshold, base.efficiency_threshold)
                .max(0.0),
            efficiency: base.efficiency,
            speed_threshold: self
                .stats
                .apply(StatId::DustSpeedThreshold, base.speed_threshold)
                .max(0.0),
            speed_multiplier: base.speed_multiplier,
            leak_threshold: self
                .stats
                .apply(StatId::DustLeakThreshold, base.leak_threshold)
                .max(0.0),
            leak: base.leak,
            stall_threshold: self
                .stats
                .apply(StatId::DustStallThreshold, base.stall_threshold)
                .max(0.0),
        }
    }
    /// Power produced this tick, including whatever the biomass harvesters
    /// burned. Generation and consumption are asked for separately because the
    /// HUD shows both halves.
    pub fn power_generation(&self) -> f32 {
        let dust_response = self.resolved_dust_response();
        self.grid.total_power_generation_with(&dust_response) + self.biomass_power_bonus
    }

    /// Power drawn this tick, after research efficiencies.
    pub fn power_consumption(&self) -> f32 {
        let dust_response = self.resolved_dust_response();
        self.stats.apply(
            StatId::PowerConsumption,
            self.grid.total_power_consumption_with(&dust_response),
        ) + self.factory_focus_power_tax()
    }

    pub fn net_power(&self) -> f32 {
        self.power_generation() - self.power_consumption()
    }

    /// Acid still reaching the network after research counters it.
    pub fn acid_strength(&self) -> f32 {
        (self.hazards.acid_rain * self.stats.multiplier(StatId::AcidResistance)).max(0.0)
    }

    /// Share of drone speed the cold still takes, after research. Capped so a
    /// world can slow the swarm to a crawl but never stop it dead.
    pub fn freeze_strength(&self) -> f32 {
        (self.hazards.freeze * self.stats.multiplier(StatId::FreezeResistance)).clamp(0.0, 0.9)
    }

    /// Alloy the working smelters would produce per second, for the readout.
    pub fn alloy_rate(&self) -> f32 {
        self.physical_output_rate("alloy")
    }

    /// Components the working assemblers would produce per second.
    pub fn components_rate(&self) -> f32 {
        self.physical_output_rate("components")
    }

    /// What the factory actually produced in its latest complete observation
    /// window, rather than its theoretical powered capacity.
    pub fn observed_alloy_rate(&self) -> f32 {
        self.graph_samples
            .last()
            .map(|sample| sample.alloy_produced)
            .unwrap_or(0.0)
    }

    pub fn observed_components_rate(&self) -> f32 {
        self.graph_samples
            .last()
            .map(|sample| sample.components_produced)
            .unwrap_or(0.0)
    }

    /// The finite dispatch apron shared by every physical-output processor.
    pub fn processor_pad_capacity(&self) -> f32 {
        let base = self
            .config
            .logistics
            .pad_depth
            .max(self.drones.drone_capacity * 3.0);
        self.stats.apply(StatId::ProcessorPadCapacity, base)
    }

    /// Powered processors that cannot work because their dispatch pad is full.
    pub fn blocked_factories(&self) -> Vec<GridPos> {
        let capacity = self.processor_pad_capacity();
        let dust_response = self.resolved_dust_response();
        self.data()
            .buildings
            .iter()
            .filter(|def| {
                def.recipe.outputs.iter().any(|(id, rate)| {
                    *rate > 0.0 && ResourceType::from_id(id).is_some_and(ResourceType::is_physical)
                })
            })
            .filter_map(|def| BuildingType::from_id(&def.id))
            .flat_map(|kind| self.grid.find_buildings(kind))
            .filter(|pos| {
                self.grid
                    .get(*pos)
                    .and_then(|tile| tile.building.as_ref())
                    .is_some_and(|building| {
                        building.powered
                            && !building.standby
                            && !building.is_dust_stalled_with(&dust_response)
                    })
                    && self
                        .output_buffers
                        .get(&(pos.x, pos.y))
                        .copied()
                        .unwrap_or(0.0)
                        >= capacity - 0.001
            })
            .collect()
    }

    /// Powered processors with at least one empty required input. Returning
    /// positions lets both the HUD count the bottleneck and the map mark it.
    pub fn starved_factories(&self) -> Vec<GridPos> {
        let dust_response = self.resolved_dust_response();
        self.data()
            .buildings
            .iter()
            .filter(|def| !def.recipe.is_empty())
            .filter_map(|def| BuildingType::from_id(&def.id).map(|kind| (kind, &def.recipe)))
            .flat_map(|(kind, recipe)| {
                self.grid
                    .find_buildings(kind)
                    .into_iter()
                    .map(move |pos| (pos, recipe))
            })
            .filter(|(pos, recipe)| {
                let Some(building) = self.grid.get(*pos).and_then(|tile| tile.building.as_ref())
                else {
                    return false;
                };
                if !building.powered
                    || building.standby
                    || building.is_dust_stalled_with(&dust_response)
                {
                    return false;
                }
                let carried = recipe.carried_ids();
                recipe.inputs.iter().any(|(id, rate)| {
                    if *rate <= 0.0 {
                        return false;
                    }
                    let Some(resource) = ResourceType::from_id(id) else {
                        return false;
                    };
                    let available = if carried.contains(&resource.id()) {
                        self.input_hoppers
                            .get(&(pos.x, pos.y))
                            .and_then(|hopper| hopper.get(&resource))
                            .copied()
                            .unwrap_or_else(|| {
                                if recipe.carried.as_deref() == Some(resource.id()) {
                                    self.input_buffers
                                        .get(&(pos.x, pos.y))
                                        .copied()
                                        .unwrap_or(0.0)
                                } else {
                                    0.0
                                }
                            })
                    } else {
                        self.resources.get(resource)
                    };
                    available <= 0.001
                })
            })
            .map(|(pos, _)| pos)
            .collect()
    }

    fn physical_output_rate(&self, resource_id: &str) -> f32 {
        let dust_response = self.resolved_dust_response();
        self.data()
            .buildings
            .iter()
            .filter_map(|def| {
                let out = def.recipe.outputs.get(resource_id).copied().unwrap_or(0.0);
                (out > 0.0).then_some((def, out))
            })
            .filter_map(|(def, out)| BuildingType::from_id(&def.id).map(|kind| (kind, out)))
            .map(|(kind, out)| {
                self.grid
                    .find_buildings(kind)
                    .into_iter()
                    .filter_map(|pos| self.grid.get(pos).and_then(|tile| tile.building.as_ref()))
                    .filter(|building| {
                        building.powered && !building.is_dust_stalled_with(&dust_response)
                    })
                    .map(|building| {
                        out * building.dust_efficiency_with(&dust_response)
                            * building.work_multiplier()
                            * self.factory_focus_multiplier(building.building_type)
                    })
                    .sum::<f32>()
            })
            .sum()
    }

    /// A short label for whatever this world is doing to the machinery, for
    /// the HUD. Empty when the world is merely somewhere to build.
    pub fn hazard_label(&self) -> String {
        let mut parts = Vec::new();
        if self.acid_strength() > 0.0 {
            parts.push("ACID RAIN");
        }
        if self.freeze_strength() > 0.0 {
            parts.push("DEEP FREEZE");
        }
        parts.join(" / ")
    }

    /// What this world has to say for itself, shown on arrival.
    pub fn arrival_line(&self) -> &str {
        self.data().planet(self.planet_index).arrival.as_str()
    }

    pub fn achievements_progress(&self) -> (usize, usize) {
        self.achievements.progress()
    }

    pub fn placement_scale(&self, pos: GridPos) -> f32 {
        if macroquad_toolkit::settings::reduced_motion_enabled() {
            return 1.0;
        }
        let Some(anim) = self
            .placement_anims
            .iter()
            .find(|anim| anim.position == pos)
        else {
            return 1.0;
        };
        let progress = (anim.timer / 0.3).clamp(0.0, 1.0);
        let bounce_phase = (1.0 - progress) * std::f32::consts::PI * 2.0;
        1.0 + (bounce_phase.sin() * 0.12)
    }

    /// Unlock an achievement, and say so if it had not already fired.
    ///
    /// `unlock` returns whether this was the moment it happened, which is the
    /// only reason the toast is not repeated every tick.
    pub fn announce_achievement(&mut self, id: &str) {
        if !self.achievements.unlock(id) {
            return;
        }
        let name = self
            .achievements
            .get(id)
            .map(|achievement| achievement.name.clone())
            .unwrap_or_else(|| id.to_string());
        self.emit_audio(super::audio::AudioEvent::Achievement);
        self.notifications.success(format!("Achievement: {}", name));
    }
}

#[cfg(test)]
mod tests;
