//! Battery accounting and offline planetary simulation.

use super::{OfflineReport, PlanetState};
use crate::engine::ResourceType;

impl PlanetState {
    pub fn battery_time_left(&self) -> (i32, i32) {
        let total = self.battery_seconds.max(0.0) as i32;
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        (hours, minutes)
    }

    pub fn apply_offline_progress(&mut self, offline_seconds: f32) {
        if offline_seconds <= 0.0 {
            self.last_offline_seconds = 0.0;
            self.last_offline_simulated = 0.0;
            return;
        }

        let capped = offline_seconds.min(self.config.offline.max_elapsed_seconds.max(0.0));
        let active_seconds = capped.min(self.battery_seconds.max(0.0));
        let hibernation_seconds = (capped - active_seconds).max(0.0) * 0.1;
        let simulated = active_seconds + hibernation_seconds;
        self.refresh_biomass_power_bonus();
        let biomass_before = self.resources.biomass;
        self.update_biomass_harvesters(simulated);
        let biomass = (self.resources.biomass - biomass_before).max(0.0);
        let mineral_rate = self.mineral_output_rate();
        let alloy_rate = self.alloy_rate();
        let components_rate = self.components_rate();
        let data_rate = self.data_generation_rate();
        let power_rate = self.net_power();
        let cap = self.config.offline.max_resource_gain.max(0.0);
        let minerals = (mineral_rate * simulated).clamp(-cap, cap);
        let alloy = (alloy_rate * simulated).clamp(0.0, cap);
        let components = (components_rate * simulated).clamp(0.0, cap);
        let data = (data_rate * simulated).clamp(0.0, cap);
        let power = (power_rate * simulated).clamp(-cap, cap);
        self.resources.consumed.record(
            ResourceType::Energy,
            self.power_consumption().max(0.0) * simulated,
        );
        self.resources.minerals = (self.resources.minerals + minerals)
            .min(self.resource_capacity(ResourceType::Minerals));
        self.resources.alloy =
            (self.resources.alloy + alloy).min(self.resource_capacity(ResourceType::Alloy));
        self.resources.components = (self.resources.components + components)
            .min(self.resource_capacity(ResourceType::Components));
        self.resources.data =
            (self.resources.data + data).min(self.resource_capacity(ResourceType::Data));
        self.resources.energy = (self.resources.energy + power)
            .clamp(0.0, self.resource_capacity(ResourceType::Energy));
        self.refresh_biomass_power_bonus();
        self.power_balance = self.net_power();
        self.time_played += simulated as f64;
        self.battery_seconds = (self.battery_seconds - capped).max(0.0);
        self.last_offline_report = OfflineReport {
            elapsed_seconds: offline_seconds,
            capped_seconds: capped,
            tamper_guarded: false,
            minerals_gained: minerals,
            biomass_gained: biomass,
            alloy_gained: alloy,
            components_gained: components,
            data_gained: data,
            power_gained: power,
            ..OfflineReport::default()
        };
        self.last_offline_seconds = offline_seconds;
        self.last_offline_simulated = simulated;
        self.offline_notice_timer = 8.0;
    }
}
