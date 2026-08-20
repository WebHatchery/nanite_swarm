//! Processor recipes and their carried-input/output buffers.

use crate::engine::{BuildingType, ResourceType, StatId};
use crate::state::game_state::PlanetState;

impl PlanetState {
    /// Run every processing building's recipe.
    ///
    /// A recipe only runs as far as its inputs allow, so a smelter starved of
    /// minerals produces proportionally less rather than stopping dead - the
    /// same shape as the drill buffer, and it keeps the numbers continuous for
    /// the fixed timestep.
    pub(super) fn update_recipes(&mut self, delta_time: f32) {
        let dust_response = self.resolved_dust_response();
        for (pos, recipe) in self.recipe_buildings() {
            let Some(building) = self.grid.get(pos).and_then(|tile| tile.building.as_ref()) else {
                continue;
            };
            if !building.powered
                || building.standby
                || building.is_dust_stalled_with(&dust_response)
            {
                continue;
            }

            let hoppers = self.input_hoppers.get(&(pos.x, pos.y));
            let mut scale = building.dust_efficiency_with(&dust_response)
                * building.work_multiplier()
                * self.factory_focus_multiplier(building.building_type)
                * delta_time;
            let physical_output_rate: f32 = recipe
                .outputs
                .iter()
                .filter_map(|(id, rate)| {
                    ResourceType::from_id(id)
                        .is_some_and(ResourceType::is_physical)
                        .then_some(*rate)
                })
                .sum();
            if physical_output_rate > 0.0 {
                let waiting = self
                    .output_buffers
                    .get(&(pos.x, pos.y))
                    .copied()
                    .unwrap_or(0.0);
                let room = (self.processor_pad_capacity() - waiting).max(0.0);
                scale = scale.min(room / physical_output_rate);
            }
            for (id, rate) in &recipe.inputs {
                if *rate <= 0.0 {
                    continue;
                }
                let Some(resource) = ResourceType::from_id(id) else {
                    continue;
                };
                let available = if recipe.carried_ids().contains(&resource.id()) {
                    hoppers
                        .and_then(|bucket| bucket.get(&resource))
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
                scale = scale.min(available / rate);
            }
            if scale <= 0.0 {
                continue;
            }

            for (id, rate) in &recipe.inputs {
                let Some(resource) = ResourceType::from_id(id) else {
                    continue;
                };
                let taken = rate * scale;
                if recipe.carried_ids().contains(&resource.id()) {
                    let hoppers = self.input_hoppers.entry((pos.x, pos.y)).or_default();
                    if let Some(buffer) = hoppers.get_mut(&resource) {
                        *buffer = (*buffer - taken).max(0.0);
                    }
                    if let Some(buffer) = self.input_buffers.get_mut(&(pos.x, pos.y)) {
                        *buffer = (*buffer - taken).max(0.0);
                    }
                } else {
                    self.resources.add(resource, -taken);
                }
                match resource {
                    ResourceType::Minerals => {
                        self.factory_flow_since_sample.minerals_consumed += taken
                    }
                    ResourceType::Alloy => self.factory_flow_since_sample.alloy_consumed += taken,
                    ResourceType::Components => {
                        self.factory_flow_since_sample.components_consumed += taken
                    }
                    _ => {}
                }
            }

            for (id, rate) in &recipe.outputs {
                let Some(resource) = ResourceType::from_id(id) else {
                    continue;
                };
                let made = if resource == ResourceType::Data {
                    self.stats.apply(StatId::DataGeneration, rate * scale)
                } else {
                    rate * scale
                };
                if resource.is_physical() {
                    *self.output_buffers.entry((pos.x, pos.y)).or_insert(0.0) += made;
                } else {
                    self.resources.add(resource, made);
                }
                match resource {
                    ResourceType::Alloy => self.factory_flow_since_sample.alloy_produced += made,
                    ResourceType::Components => {
                        self.factory_flow_since_sample.components_produced += made
                    }
                    ResourceType::Data => self.factory_flow_since_sample.data_produced += made,
                    _ => {}
                }
            }
        }
    }

    /// Every placed building that has a recipe, with it.
    fn recipe_buildings(&self) -> Vec<(crate::engine::GridPos, &'static crate::data::RecipeDef)> {
        crate::data::game_data()
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
            .collect()
    }
}
