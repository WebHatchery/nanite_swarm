//! Durable lifetime accounting for resources spent by a world.

use crate::engine::ResourceType;
use serde::{Deserialize, Serialize};

/// Resources that have been irreversibly spent by construction, operations,
/// research, processing, power draw, or the Seed Ship.
///
/// This is deliberately separate from the live resource pools. A stockpile is
/// capped and can be refunded or moved; this ledger only moves forward, so an
/// ending report can tell the truth about the work a world performed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ConsumptionTotals {
    #[serde(default)]
    pub minerals: f32,
    #[serde(default)]
    pub energy: f32,
    #[serde(default)]
    pub data: f32,
    #[serde(default)]
    pub biomass: f32,
    #[serde(default)]
    pub alloy: f32,
    #[serde(default)]
    pub components: f32,
}

impl ConsumptionTotals {
    /// Add one positive sink to the lifetime ledger.
    pub fn record(&mut self, resource: ResourceType, amount: f32) {
        if !amount.is_finite() || amount <= 0.0 {
            return;
        }
        match resource {
            ResourceType::Minerals => self.minerals += amount,
            ResourceType::Energy => self.energy += amount,
            ResourceType::Data => self.data += amount,
            ResourceType::Biomass => self.biomass += amount,
            ResourceType::Alloy => self.alloy += amount,
            ResourceType::Components => self.components += amount,
        }
    }

    /// Combine totals from another world without allowing invalid values to
    /// contaminate the campaign report.
    pub fn add(&mut self, other: Self) {
        for (resource, amount) in [
            (ResourceType::Minerals, other.minerals),
            (ResourceType::Energy, other.energy),
            (ResourceType::Data, other.data),
            (ResourceType::Biomass, other.biomass),
            (ResourceType::Alloy, other.alloy),
            (ResourceType::Components, other.components),
        ] {
            self.record(resource, amount);
        }
    }

    /// Read one total by the same identifier used by recipes and telemetry.
    pub fn get(self, resource: ResourceType) -> f32 {
        match resource {
            ResourceType::Minerals => self.minerals,
            ResourceType::Energy => self.energy,
            ResourceType::Data => self.data,
            ResourceType::Biomass => self.biomass,
            ResourceType::Alloy => self.alloy,
            ResourceType::Components => self.components,
        }
    }
}

/// Live resource pools plus their durable lifetime sinks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resources {
    pub energy: f32,
    pub minerals: f32,
    pub data: f32,
    pub biomass: f32,
    /// Refined from minerals by processing buildings. The first product the
    /// swarm cannot dig straight out of the ground.
    #[serde(default)]
    pub alloy: f32,
    /// Precision assemblies produced from more than one routed factory input.
    #[serde(default)]
    pub components: f32,
    /// Lifetime resource sinks; unlike the pools above this never goes down.
    #[serde(default)]
    pub consumed: ConsumptionTotals,
}

impl Resources {
    /// Read a pool by resource id, so a recipe declared in JSON can ask about
    /// something without the engine having a branch per resource.
    pub fn get(&self, resource: ResourceType) -> f32 {
        match resource {
            ResourceType::Energy => self.energy,
            ResourceType::Minerals => self.minerals,
            ResourceType::Data => self.data,
            ResourceType::Biomass => self.biomass,
            ResourceType::Alloy => self.alloy,
            ResourceType::Components => self.components,
        }
    }

    pub fn add(&mut self, resource: ResourceType, amount: f32) {
        match resource {
            ResourceType::Energy => self.energy += amount,
            ResourceType::Minerals => self.minerals += amount,
            ResourceType::Data => self.data += amount,
            ResourceType::Biomass => self.biomass += amount,
            ResourceType::Alloy => self.alloy += amount,
            ResourceType::Components => self.components += amount,
        }
    }

    /// Remove a resource from its live pool and record the irreversible sink.
    pub fn consume(&mut self, resource: ResourceType, amount: f32) -> f32 {
        let moved = amount.max(0.0).min(self.get(resource));
        if moved > 0.0 && moved.is_finite() {
            self.add(resource, -moved);
            self.consumed.record(resource, moved);
            moved
        } else {
            0.0
        }
    }
}

impl Resources {
    /// Check if player can afford a cost
    pub fn can_afford(&self, minerals: f32, energy: f32) -> bool {
        self.minerals >= minerals && self.energy >= energy
    }

    /// Deduct cost from resources
    pub fn spend(&mut self, minerals: f32, energy: f32) -> bool {
        if self.can_afford(minerals, energy) {
            self.minerals -= minerals;
            self.energy -= energy;
            self.consumed.record(ResourceType::Minerals, minerals);
            self.consumed.record(ResourceType::Energy, energy);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests;
