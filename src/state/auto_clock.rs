//! Adaptive processor policy: spend spare power on fed, healthy machines.

use std::collections::HashSet;

use crate::engine::GridPos;

use super::PlanetState;

const POLICY_INTERVAL: f32 = 1.0;
const ENABLE_BELOW_DUST: f32 = 55.0;
const DISABLE_AT_DUST: f32 = 75.0;
const POWER_RESERVE: f32 = 2.0;

impl PlanetState {
    pub fn toggle_auto_clocking(&mut self) -> bool {
        if !self
            .research
            .unlocked_techs
            .iter()
            .any(|tech| tech == "adaptive_clocking")
        {
            self.notifications
                .warning("Research Adaptive Clocking to automate processors");
            return false;
        }
        self.auto_clocking = !self.auto_clocking;
        self.auto_clock_timer = POLICY_INTERVAL;
        self.notifications.info(if self.auto_clocking {
            "Auto Clock ON: fed processors use spare power"
        } else {
            "Auto Clock OFF: processor modes stay manual"
        });
        true
    }

    pub(super) fn update_auto_clocking(&mut self, delta_time: f32) {
        if !self.auto_clocking {
            self.auto_clock_timer = 0.0;
            return;
        }
        self.auto_clock_timer += delta_time;
        if self.auto_clock_timer < POLICY_INTERVAL {
            return;
        }
        self.auto_clock_timer %= POLICY_INTERVAL;
        self.rebalance_auto_clocking();
    }

    fn rebalance_auto_clocking(&mut self) -> (usize, usize) {
        let starved: HashSet<GridPos> = self.starved_factories().into_iter().collect();
        let blocked: HashSet<GridPos> = self.blocked_factories().into_iter().collect();
        let mut processors: Vec<GridPos> = self
            .grid
            .iter_tiles()
            .filter_map(|(pos, tile)| {
                tile.building
                    .as_ref()
                    .is_some_and(|building| building.supports_overclock())
                    .then_some(pos)
            })
            .collect();
        processors.sort_by_key(|pos| (pos.y, pos.x));
        let mut headroom = self.net_power();
        let mut normalized = 0;

        // Recover safety first. Hysteresis between the dust thresholds keeps a
        // machine from flipping every policy pulse at one boundary.
        for pos in &processors {
            let Some(building) = self
                .grid
                .get_mut(*pos)
                .and_then(|tile| tile.building.as_mut())
            else {
                continue;
            };
            let unsafe_to_boost = !building.powered
                || starved.contains(pos)
                || blocked.contains(pos)
                || building.dust >= DISABLE_AT_DUST
                || headroom < 0.0;
            if building.overclocked && unsafe_to_boost {
                building.overclocked = false;
                headroom += extra_power_draw(building);
                normalized += 1;
            }
        }

        let mut boosted = 0;
        for pos in processors {
            let Some(building) = self
                .grid
                .get_mut(pos)
                .and_then(|tile| tile.building.as_mut())
            else {
                continue;
            };
            let extra = extra_power_draw(building);
            if !building.overclocked
                && building.powered
                && !starved.contains(&pos)
                && !blocked.contains(&pos)
                && building.dust <= ENABLE_BELOW_DUST
                && headroom >= extra + POWER_RESERVE
            {
                building.overclocked = true;
                headroom -= extra;
                boosted += 1;
            }
        }
        if boosted > 0 || normalized > 0 {
            self.grid.update_power_grid();
            self.power_balance = self.net_power();
        }
        (boosted, normalized)
    }
}

fn extra_power_draw(building: &crate::engine::Building) -> f32 {
    building.building_type.power_delta().abs() * 0.75
}

#[cfg(test)]
mod tests;
