//! World conditions and resource ceilings used by placement and simulation.

use crate::engine::{BuildingType, GridPos, ResourceType, StatId};

use super::super::game_state::PlanetState;

pub(crate) fn hazard_field_strength(
    pos: GridPos,
    hazard: &str,
    broad: f32,
    fields: &[crate::data::HazardFieldDef],
    width: u32,
    height: u32,
) -> f32 {
    let width = width.max(1) as f32;
    let height = height.max(1) as f32;
    let x = pos.x as f32 / width;
    let y = pos.y as f32 / height;
    let field = fields
        .iter()
        .filter(|field| field.hazard == hazard && field.radius > 0.0)
        .map(|field| {
            let dx = (x - field.center[0]) * width;
            let dy = (y - field.center[1]) * height;
            let distance = (dx * dx + dy * dy).sqrt();
            (field.strength * (1.0 - distance / (field.radius * width.min(height)))).max(0.0)
        })
        .fold(0.0, f32::max);
    broad.max(field)
}

impl PlanetState {
    pub fn is_building_unlocked(&self, building_type: BuildingType) -> bool {
        self.is_building_researched(building_type)
            && !self.is_building_banned(building_type)
            && self.building_condition_met(building_type)
    }

    /// Research has opened this building up, whether or not this particular
    /// world will accept it.
    pub fn is_building_researched(&self, building_type: BuildingType) -> bool {
        matches!(building_type, BuildingType::Core)
            || self.unlocked_buildings.contains(&building_type)
    }

    pub fn active_planet_condition(&self) -> Option<&'static str> {
        if self.hazards.acid_rain > 0.0
            || self
                .data()
                .planet(self.planet_index)
                .hazard_fields
                .iter()
                .any(|field| field.hazard == "acid")
        {
            Some("acid")
        } else if self.hazards.freeze > 0.0
            || self
                .data()
                .planet(self.planet_index)
                .hazard_fields
                .iter()
                .any(|field| field.hazard == "cold")
        {
            Some("cold")
        } else {
            None
        }
    }

    /// The broad world condition declared by a building, if it has one.
    pub fn building_planet_condition(&self, building_type: BuildingType) -> Option<&str> {
        self.data()
            .building(building_type.id())
            .planet_condition
            .as_deref()
    }

    pub fn building_condition_met(&self, building_type: BuildingType) -> bool {
        self.building_planet_condition(building_type)
            .is_none_or(|required| self.active_planet_condition() == Some(required))
    }

    /// Strength of a named spatial field at a tile, including the world's
    /// broad hazard value. Radial falloff keeps field edges readable and lets
    /// one definition work on every supported map size.
    pub fn hazard_strength_at(&self, pos: GridPos, hazard: &str) -> f32 {
        let broad = match hazard {
            "acid" => self.acid_strength(),
            "cold" => self.freeze_strength(),
            _ => 0.0,
        };
        let def = self.data().planet(self.planet_index);
        hazard_field_strength(
            pos,
            hazard,
            broad,
            &def.hazard_fields,
            self.grid.width,
            self.grid.height,
        )
    }

    pub fn planet_constraint_status(&self) -> (bool, String) {
        let data = self.data();
        let constraints = &data.planet(self.planet_index).constraints;
        let missing: Vec<String> = constraints
            .required_buildings
            .iter()
            .filter(|id| {
                BuildingType::from_id(id)
                    .is_none_or(|kind| self.grid.find_buildings(kind).is_empty())
            })
            .map(|id| {
                BuildingType::from_id(id)
                    .map(|kind| kind.name(data).to_string())
                    .unwrap_or_else(|| id.clone())
            })
            .collect();
        let missing_research: Vec<String> = constraints
            .required_research
            .iter()
            .filter(|required| {
                !self
                    .research
                    .unlocked_techs
                    .iter()
                    .any(|known| known == *required)
            })
            .map(|id| {
                data.research
                    .nodes
                    .iter()
                    .find(|node| node.id == *id)
                    .map(|node| node.name.clone())
                    .unwrap_or_else(|| id.clone())
            })
            .collect();
        let generation_ok = self.power_generation() >= constraints.minimum_power_generation;
        let balance_ok = self.power_balance >= constraints.minimum_power_balance;
        let mut reasons = Vec::new();
        if !missing.is_empty() {
            reasons.push(format!("needs {}", missing.join(", ")));
        }
        if !missing_research.is_empty() {
            reasons.push(format!("needs research {}", missing_research.join(", ")));
        }
        if !generation_ok {
            reasons.push(format!(
                "needs {:.0} power generation",
                constraints.minimum_power_generation
            ));
        }
        if !balance_ok {
            reasons.push(format!(
                "needs {:.0} power surplus",
                constraints.minimum_power_balance
            ));
        }
        (reasons.is_empty(), reasons.join("; "))
    }

    /// Positive world rules gate the infrastructure that makes a campaign-wide
    /// route meaningful. Ordinary infrastructure must remain placeable while
    /// the player is satisfying that rule; only the gated Mass Driver checks
    /// the world's prerequisite buildings and power floor.
    pub fn constraints_allow_building(&self, building_type: BuildingType) -> bool {
        if !self.building_condition_met(building_type) {
            return false;
        }
        let constraints = &self.data().planet(self.planet_index).constraints;
        if !constraints.required_research.iter().all(|required| {
            self.research
                .unlocked_techs
                .iter()
                .any(|known| known == required)
        }) {
            return false;
        }
        if building_type == BuildingType::MassDriver {
            let required_buildings_met = constraints.required_buildings.iter().all(|required| {
                BuildingType::from_id(required).is_none_or(|kind| {
                    kind == building_type || !self.grid.find_buildings(kind).is_empty()
                })
            });
            let power_floor_met = self.power_generation() >= constraints.minimum_power_generation;
            if !required_buildings_met || !power_floor_met {
                return false;
            }
        }
        true
    }

    /// This world refuses the building outright, however much research the
    /// swarm has done.
    pub fn is_building_banned(&self, building_type: BuildingType) -> bool {
        self.banned_buildings.contains(&building_type)
    }

    pub fn unlock_building(&mut self, building_type: BuildingType) {
        if !self.unlocked_buildings.contains(&building_type) {
            self.unlocked_buildings.push(building_type);
        }
    }

    pub fn mineral_capacity(&self) -> f32 {
        self.stats
            .apply(StatId::MineralCapacity, self.built_mineral_capacity())
    }

    /// The live storage ceiling for a resource. Keeping this in one place is
    /// important because foreground ticks, offline progress, and the HUD all
    /// need to agree about what a full stockpile means.
    pub fn resource_capacity(&self, resource: ResourceType) -> f32 {
        match resource {
            ResourceType::Energy => self.config.resources.max_energy,
            ResourceType::Minerals => self.mineral_capacity(),
            ResourceType::Data
            | ResourceType::Biomass
            | ResourceType::Alloy
            | ResourceType::Components => 1_000.0,
        }
    }

    pub fn resource_is_full(&self, resource: ResourceType) -> bool {
        self.resources.get(resource) >= self.resource_capacity(resource) - 0.001
    }

    /// Storage before research touches it: the base plus whatever has been
    /// built for it. The stat sheet needs the two halves separately.
    pub fn built_mineral_capacity(&self) -> f32 {
        let storage_count = self.grid.find_buildings(BuildingType::Storage).len() as f32;
        self.config.resources.base_mineral_cap + storage_count * self.config.resources.storage_bonus
    }
}
