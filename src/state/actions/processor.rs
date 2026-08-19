//! Processor policy, standby, and destructive buffer-recovery actions.

use std::collections::HashSet;

use crate::engine::GridPos;

use super::PlanetState;

impl PlanetState {
    pub fn set_selected_input_priority(&mut self, enabled: bool) -> usize {
        let positions = self.box_selected.clone();
        let mut changed = 0;
        for pos in positions {
            let Some(building) = self
                .grid
                .get_mut(pos)
                .and_then(|tile| tile.building.as_mut())
            else {
                continue;
            };
            if building.supports_overclock() && building.input_priority != enabled {
                building.input_priority = enabled;
                changed += 1;
            }
        }
        if changed > 0 {
            self.notifications.info(format!(
                "{} processor{} set to {} input",
                changed,
                if changed == 1 { "" } else { "s" },
                if enabled { "priority" } else { "standard" }
            ));
        }
        changed
    }

    pub fn set_selected_standby(&mut self, standby: bool) -> usize {
        let positions = self.box_selected.clone();
        let mut changed = 0;
        for pos in positions {
            let Some(building) = self
                .grid
                .get_mut(pos)
                .and_then(|tile| tile.building.as_mut())
            else {
                continue;
            };
            if building.supports_overclock() && building.standby != standby {
                building.standby = standby;
                if standby {
                    building.overclocked = false;
                }
                changed += 1;
            }
        }
        if changed > 0 {
            self.grid.update_power_grid();
            self.power_balance = self.net_power();
            self.notifications.info(format!(
                "{} processor{} {}",
                changed,
                if changed == 1 { "" } else { "s" },
                if standby { "paused" } else { "resumed" }
            ));
        }
        changed
    }

    pub fn request_processor_pad_purge(&mut self, pos: GridPos) -> bool {
        self.bulk_purge_armed = false;
        let waiting = self
            .output_buffers
            .get(&(pos.x, pos.y))
            .copied()
            .unwrap_or(0.0);
        let is_processor = self
            .grid
            .get(pos)
            .and_then(|tile| tile.building.as_ref())
            .is_some_and(|building| building.supports_overclock());
        if !is_processor || waiting <= 0.001 {
            self.purge_armed = None;
            return false;
        }
        if self.purge_armed != Some(pos) {
            self.purge_armed = Some(pos);
            self.notifications.warning(format!(
                "Tap PURGE AGAIN to discard {:.0} staged output",
                waiting
            ));
            return false;
        }
        self.output_buffers.remove(&(pos.x, pos.y));
        self.purge_armed = None;
        self.notifications
            .warning(format!("Purged {:.0} staged output", waiting));
        true
    }

    pub fn request_selected_pad_purge(&mut self) -> usize {
        let blocked: HashSet<GridPos> = self.blocked_factories().into_iter().collect();
        let targets: Vec<GridPos> = self
            .box_selected
            .iter()
            .copied()
            .filter(|pos| blocked.contains(pos))
            .collect();
        if targets.is_empty() {
            self.bulk_purge_armed = false;
            return 0;
        }
        let total: f32 = targets
            .iter()
            .filter_map(|pos| self.output_buffers.get(&(pos.x, pos.y)))
            .sum();
        if !self.bulk_purge_armed {
            self.bulk_purge_armed = true;
            self.purge_armed = None;
            self.notifications.warning(format!(
                "Tap PURGE AGAIN to clear {} full pad{} ({:.0} cargo)",
                targets.len(),
                if targets.len() == 1 { "" } else { "s" },
                total
            ));
            return 0;
        }
        for pos in &targets {
            self.output_buffers.remove(&(pos.x, pos.y));
        }
        self.bulk_purge_armed = false;
        self.notifications.warning(format!(
            "Purged {} selected pad{} ({:.0} cargo)",
            targets.len(),
            if targets.len() == 1 { "" } else { "s" },
            total
        ));
        targets.len()
    }

    pub fn toggle_input_priority(&mut self, pos: GridPos) -> bool {
        let Some(building) = self
            .grid
            .get_mut(pos)
            .and_then(|tile| tile.building.as_mut())
        else {
            return false;
        };
        if !building.supports_overclock() {
            return false;
        }
        building.input_priority = !building.input_priority;
        let enabled = building.input_priority;
        let name = building.building_type.name();
        self.notifications.info(if enabled {
            format!("{} input priority: first claim on routed cargo", name)
        } else {
            format!("{} input priority returned to standard", name)
        });
        true
    }

    pub fn toggle_processor_standby(&mut self, pos: GridPos) -> bool {
        let Some(building) = self
            .grid
            .get_mut(pos)
            .and_then(|tile| tile.building.as_mut())
        else {
            return false;
        };
        if !building.supports_overclock() {
            return false;
        }
        building.standby = !building.standby;
        if building.standby {
            building.overclocked = false;
        }
        let standby = building.standby;
        let name = building.building_type.name();
        self.grid.update_power_grid();
        self.power_balance = self.net_power();
        self.notifications.info(if standby {
            format!("{} on standby; freight buffers preserved", name)
        } else {
            format!("{} resumed", name)
        });
        true
    }
}
