use std::collections::HashSet;

use super::GameData;

fn duplicate_id<'a>(source: &str, mut ids: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut seen = HashSet::new();
    ids.find_map(|id| {
        (!seen.insert(id)).then(|| format!("{source} contains duplicate identifier \"{id}\""))
    })
}

impl GameData {
    /// Validate cross-file references once, with the source asset and
    /// identifier in every failure. Runtime lookups remain compact because a
    /// malformed package cannot reach them.
    pub fn validate(&self) -> Result<(), String> {
        use crate::engine::{BuildingType, ResourceType};

        if let Some(error) = duplicate_id(
            "assets/buildings.json",
            self.buildings.iter().map(|def| def.id.as_str()),
        ) {
            return Err(error);
        }
        if let Some(error) = duplicate_id(
            "assets/terrain.json",
            self.terrain.iter().map(|def| def.id.as_str()),
        ) {
            return Err(error);
        }
        if let Some(error) = duplicate_id(
            "assets/research.json",
            self.research.nodes.iter().map(|node| node.id.as_str()),
        ) {
            return Err(error);
        }
        if let Some(error) = duplicate_id(
            "assets/planets.json",
            self.planets.iter().map(|planet| planet.id.as_str()),
        ) {
            return Err(error);
        }
        if let Some(error) = duplicate_id(
            "assets/directives.json",
            self.directives.directives.iter().map(|def| def.id.as_str()),
        ) {
            return Err(error);
        }
        if let Some(error) = duplicate_id(
            "assets/achievements.json",
            self.achievements.iter().map(|def| def.id.as_str()),
        ) {
            return Err(error);
        }
        if let Some(error) = duplicate_id(
            "assets/core_stages.json",
            self.core_stages.iter().map(|stage| stage.id.as_str()),
        ) {
            return Err(error);
        }
        if let Some(error) = duplicate_id(
            "assets/seed_ship.json",
            self.seed_ship.stages.iter().map(|stage| stage.id.as_str()),
        ) {
            return Err(error);
        }

        let building_ids: HashSet<_> = self.buildings.iter().map(|def| def.id.as_str()).collect();
        let research_ids: HashSet<_> = self
            .research
            .nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect();
        let terrain_ids: HashSet<_> = self.terrain.iter().map(|def| def.id.as_str()).collect();

        for def in &self.buildings {
            if BuildingType::from_id(&def.id).is_none() {
                return Err(format!(
                    "assets/buildings.json building \"{}\" has no engine type",
                    def.id
                ));
            }
            for (resource, value) in def.recipe.inputs.iter().chain(def.recipe.outputs.iter()) {
                if ResourceType::from_id(resource).is_none() {
                    return Err(format!(
                        "assets/buildings.json building \"{}\" recipe uses unknown resource \"{}\"",
                        def.id, resource
                    ));
                }
                if !value.is_finite() || *value < 0.0 {
                    return Err(format!(
                        "assets/buildings.json building \"{}\" recipe has invalid rate for \"{}\"",
                        def.id, resource
                    ));
                }
            }
            for carried in def.recipe.carried_ids() {
                if !def.recipe.inputs.contains_key(carried) {
                    return Err(format!(
                        "assets/buildings.json building \"{}\" carries missing input \"{}\"",
                        def.id, carried
                    ));
                }
                if ResourceType::from_id(carried).is_none_or(|resource| !resource.is_physical()) {
                    return Err(format!(
                        "assets/buildings.json building \"{}\" carries non-physical input \"{}\"",
                        def.id, carried
                    ));
                }
            }
            if let Some(required) = &def.unlocked_by {
                if !research_ids.contains(required.as_str()) {
                    return Err(format!(
                        "assets/buildings.json building \"{}\" references missing unlock \"{}\"",
                        def.id, required
                    ));
                }
            }
        }

        for node in &self.research.nodes {
            for prerequisite in &node.prerequisites {
                if !research_ids.contains(prerequisite.as_str()) {
                    return Err(format!(
                        "assets/research.json node \"{}\" references missing prerequisite \"{}\"",
                        node.id, prerequisite
                    ));
                }
            }
            if let Some(condition) = &node.planet_condition {
                if !["acid", "cold", "water", "mountain"].contains(&condition.as_str()) {
                    return Err(format!(
                        "assets/research.json node \"{}\" has unknown planet condition \"{}\"",
                        node.id, condition
                    ));
                }
            }
        }
        for unlocked in &self.research.starting_unlocked {
            if !research_ids.contains(unlocked.as_str()) {
                return Err(format!(
                    "assets/research.json starting unlock references missing node \"{}\"",
                    unlocked
                ));
            }
        }

        for terrain in &self.terrain {
            if !terrain_ids.contains(terrain.harvested_to.as_str()) {
                return Err(format!(
                    "assets/terrain.json terrain \"{}\" references missing harvested_to \"{}\"",
                    terrain.id, terrain.harvested_to
                ));
            }
        }

        for planet in &self.planets {
            if planet.width == 0 || planet.height == 0 {
                return Err(format!(
                    "assets/planets.json planet \"{}\" has an empty map",
                    planet.id
                ));
            }
            let terrain_sum = planet.terrain.mountain
                + planet.terrain.forest
                + planet.terrain.water
                + planet.terrain.void;
            if !terrain_sum.is_finite() || !(0.0..=1.0).contains(&terrain_sum) {
                return Err(format!(
                    "assets/planets.json planet \"{}\" has invalid terrain weights",
                    planet.id
                ));
            }
            for id in planet
                .banned_buildings
                .iter()
                .chain(planet.constraints.required_buildings.iter())
            {
                if !building_ids.contains(id.as_str()) {
                    return Err(format!(
                        "assets/planets.json planet \"{}\" references missing building \"{}\"",
                        planet.id, id
                    ));
                }
            }
            for id in &planet.constraints.required_research {
                if !research_ids.contains(id.as_str()) {
                    return Err(format!(
                        "assets/planets.json planet \"{}\" references missing research \"{}\"",
                        planet.id, id
                    ));
                }
            }
            if planet.pending_pod_cap == 0 {
                return Err(format!(
                    "assets/planets.json planet \"{}\" has non-positive pending pod cap",
                    planet.id
                ));
            }
            if !planet.constraints.minimum_power_generation.is_finite()
                || !planet.constraints.minimum_power_balance.is_finite()
                || planet.constraints.minimum_power_generation < 0.0
                || planet.constraints.minimum_power_balance < 0.0
            {
                return Err(format!(
                    "assets/planets.json planet \"{}\" has invalid power constraints",
                    planet.id
                ));
            }
            for feature in &planet.features {
                if feature.width <= 0.0
                    || feature.height <= 0.0
                    || feature.width > 1.0
                    || feature.height > 1.0
                    || feature.width + feature.height <= 0.0
                {
                    return Err(format!(
                        "assets/planets.json planet \"{}\" feature \"{}\" has invalid bounds",
                        planet.id, feature.id
                    ));
                }
            }
            if let Some(error) = duplicate_id(
                &format!("assets/planets.json planet \"{}\" features", planet.id),
                planet.features.iter().map(|feature| feature.id.as_str()),
            ) {
                return Err(error);
            }
            for field in &planet.hazard_fields {
                if !["acid", "cold"].contains(&field.hazard.as_str())
                    || !field.radius.is_finite()
                    || field.radius <= 0.0
                    || !field.strength.is_finite()
                    || field.strength < 0.0
                    || field.center.iter().any(|value| !value.is_finite())
                {
                    return Err(format!(
                        "assets/planets.json planet \"{}\" field \"{}\" has invalid hazard definition",
                        planet.id, field.id
                    ));
                }
            }
        }

        for def in &self.directives.directives {
            if crate::directives::DirectiveKind::from_id(&def.kind).is_none() {
                return Err(format!(
                    "assets/directives.json directive \"{}\" has unknown kind \"{}\"",
                    def.id, def.kind
                ));
            }
            if !def.base_target.is_finite() || !def.base_reward.is_finite() {
                return Err(format!(
                    "assets/directives.json directive \"{}\" has invalid target or reward",
                    def.id
                ));
            }
        }
        for def in &self.achievements {
            if crate::state::Milestone::from_id(&def.condition.kind).is_none() {
                return Err(format!(
                    "assets/achievements.json achievement \"{}\" has unknown condition \"{}\"",
                    def.id, def.condition.kind
                ));
            }
            if !def.condition.target.is_finite() || def.condition.target < 0.0 {
                return Err(format!(
                    "assets/achievements.json achievement \"{}\" has invalid target",
                    def.id
                ));
            }
        }
        let valid_sprites = [
            "core_stage_1a",
            "core_stage_1b",
            "core_stage_1c",
            "core_stage_2a",
            "core_stage_2b",
            "core_stage_3a",
            "core_stage_3b",
            "core_stage_4a",
            "core_stage_4b",
        ];
        for stage in &self.core_stages {
            for requirement in &stage.requires {
                if crate::state::Milestone::from_id(&requirement.kind).is_none() {
                    return Err(format!(
                        "assets/core_stages.json stage \"{}\" has unknown requirement \"{}\"",
                        stage.id, requirement.kind
                    ));
                }
            }
            for modifier in &stage.modifiers {
                crate::engine::parse_modifier(modifier).map_err(|problem| {
                    format!("assets/core_stages.json stage \"{}\": {problem}", stage.id)
                })?;
            }
            if stage
                .sprite
                .as_deref()
                .is_some_and(|sprite| !valid_sprites.contains(&sprite))
            {
                return Err(format!(
                    "assets/core_stages.json stage \"{}\" references missing sprite",
                    stage.id
                ));
            }
        }
        for stage in &self.seed_ship.stages {
            if let Some(required) = &stage.requires {
                if !research_ids.contains(required.as_str()) {
                    return Err(format!(
                        "assets/seed_ship.json stage \"{}\" references missing research \"{}\"",
                        stage.id, required
                    ));
                }
            }
            for modifier in &stage.modifiers {
                crate::engine::parse_modifier(modifier).map_err(|problem| {
                    format!("assets/seed_ship.json stage \"{}\": {problem}", stage.id)
                })?;
            }
        }
        for step in &self.tutorial {
            if step.goal.kind == "research" && !research_ids.contains(step.goal.target.as_str()) {
                return Err(format!(
                    "assets/tutorial.json step \"{}\" references missing research \"{}\"",
                    step.id, step.goal.target
                ));
            }
        }
        Ok(())
    }
}
