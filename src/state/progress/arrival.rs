use crate::engine::BuildingType;

use super::super::game_state::PlanetState;

impl PlanetState {
    /// The next visible control that moves this world's authored power or
    /// construction rule forward. This replaces the opening lesson on later
    /// worlds without inventing a second tutorial tree for every planet.
    pub fn arrival_guidance(&self) -> Option<String> {
        let data = self.data();
        let condition = self.active_planet_condition();
        let source = condition.and_then(|condition| {
            data.buildings.iter().find(|building| {
                building.generates_power
                    && building.planet_condition.as_deref() == Some(condition)
                    && building.show_in_build_menu
            })
        });

        if let Some(source) = source {
            if let Some(research_id) = source.unlocked_by.as_deref() {
                if let Some(next_id) = self.first_missing_research(research_id) {
                    let name = data
                        .research
                        .nodes
                        .iter()
                        .find(|node| node.id == next_id)
                        .map(|node| node.name.as_str())
                        .unwrap_or(next_id.as_str());
                    if self.research.current_research.as_deref() == Some(next_id.as_str()) {
                        return Some(format!("Next: let {name} finish."));
                    }
                    return Some(format!("Next: tap RESEARCH for {name}."));
                }
            }

            if let Some(kind) = BuildingType::from_id(&source.id) {
                if self.grid.find_buildings(kind).is_empty() {
                    return Some(format!("Next: tap BUILD, then place {}.", source.name));
                }
            }
        }

        let constraints = &data.planet(self.planet_index).constraints;
        if let Some(id) = constraints
            .required_research
            .iter()
            .find_map(|id| self.first_missing_research(id))
        {
            let name = data
                .research
                .nodes
                .iter()
                .find(|node| node.id == id)
                .map(|node| node.name.as_str())
                .unwrap_or(id.as_str());
            return Some(format!("Next: tap RESEARCH for {name}."));
        }
        if let Some(id) = constraints.required_buildings.iter().find(|id| {
            BuildingType::from_id(id).is_some_and(|kind| self.grid.find_buildings(kind).is_empty())
        }) {
            if let Some(kind) = BuildingType::from_id(id) {
                if let Some(research_id) = data.building(kind.id()).unlocked_by.as_deref() {
                    if let Some(next_id) = self.first_missing_research(research_id) {
                        let name = data
                            .research
                            .nodes
                            .iter()
                            .find(|node| node.id == next_id)
                            .map(|node| node.name.as_str())
                            .unwrap_or(next_id.as_str());
                        return Some(format!("Next: tap RESEARCH for {name}."));
                    }
                }
                return Some(format!("Next: tap BUILD, then place {}.", kind.name(data)));
            }
        }
        if self.power_generation() < constraints.minimum_power_generation {
            return Some(format!(
                "Next: tap BUILD, then add {:.0} power generation.",
                constraints.minimum_power_generation
            ));
        }
        None
    }

    fn first_missing_research(&self, id: &str) -> Option<String> {
        if self.research.unlocked_techs.iter().any(|known| known == id) {
            return None;
        }
        let node = self
            .data()
            .research
            .nodes
            .iter()
            .find(|node| node.id == id)?;
        node.prerequisites
            .iter()
            .find_map(|prerequisite| self.first_missing_research(prerequisite))
            .or_else(|| Some(id.to_string()))
    }
}
