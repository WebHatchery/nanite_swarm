//! Box selection, blueprint stamping, and bulk processor policy.

use crate::engine::GridPos;

use super::PlanetState;

impl PlanetState {
    pub fn save_blueprint(&mut self, anchor: GridPos, positions: &[GridPos]) -> bool {
        let entries: Vec<_> = positions
            .iter()
            .filter_map(|pos| {
                self.grid
                    .get(*pos)
                    .and_then(|tile| tile.building.as_ref())
                    .map(|building| super::super::game_state::BlueprintEntry {
                        offset: (pos.x - anchor.x, pos.y - anchor.y),
                        building_type: building.building_type,
                        overclocked: building.overclocked,
                        input_priority: building.input_priority,
                        standby: building.standby,
                    })
            })
            .collect();
        if entries.is_empty() {
            self.notifications
                .warning("Select at least one building before saving a blueprint");
            return false;
        }
        self.blueprint = entries;
        self.notifications.info("Blueprint saved");
        true
    }

    pub fn begin_box_select(&mut self) {
        self.selected_building = None;
        self.demolish_mode = false;
        self.relocation_source = None;
        self.box_select_start = None;
        self.bulk_purge_armed = false;
        self.purge_armed = None;
        self.drag_last_pos = None;
        self.smart_drag_source = None;
        self.box_select_mode = true;
        self.box_selected.clear();
        self.notifications
            .info("Drag across the grid, or tap two opposite corners");
    }

    pub fn finish_box_select(&mut self, end: GridPos) {
        let Some(start) = self.box_select_start.take() else {
            self.box_select_start = Some(end);
            self.notifications.info("Tap the opposite corner");
            return;
        };
        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);
        self.box_selected = self
            .grid
            .iter_tiles()
            .filter_map(|(pos, tile)| {
                (pos.x >= min_x
                    && pos.x <= max_x
                    && pos.y >= min_y
                    && pos.y <= max_y
                    && tile.building.is_some())
                .then_some(pos)
            })
            .collect();
        self.box_select_mode = false;
        self.bulk_purge_armed = false;
        self.selected_tile = self.box_selected.first().copied();
        self.notifications
            .info(format!("Selected {} buildings", self.box_selected.len()));
    }

    pub fn set_selected_overclock(&mut self, enabled: bool) -> usize {
        if !self
            .research
            .unlocked_techs
            .iter()
            .any(|tech| tech == "adaptive_clocking")
        {
            self.notifications
                .warning("Research Adaptive Clocking to boost processors");
            return 0;
        }
        let selected = self.box_selected.clone();
        self.auto_clocking = false;
        let mut changed = 0;
        let mut standby_count = 0;
        for pos in selected {
            let Some(building) = self
                .grid
                .get_mut(pos)
                .and_then(|tile| tile.building.as_mut())
            else {
                continue;
            };
            if !building.supports_overclock() {
                continue;
            }
            if building.standby {
                standby_count += 1;
                continue;
            }
            if building.overclocked != enabled {
                building.overclocked = enabled;
                changed += 1;
            }
        }
        self.power_balance = self.net_power();
        if changed > 0 {
            self.notifications.info(if enabled {
                format!("Boosted {} selected processors", changed)
            } else {
                format!("Normalized {} selected processors", changed)
            });
        } else if enabled && standby_count > 0 {
            self.notifications.warning(format!(
                "Resume {} paused processor{} before boosting",
                standby_count,
                if standby_count == 1 { "" } else { "s" }
            ));
        } else {
            self.notifications.warning("No selected processors changed");
        }
        changed
    }

    pub fn place_blueprint(&mut self, anchor: GridPos) -> usize {
        if self.blueprint.is_empty() {
            self.notifications
                .warning("Save a blueprint before stamping");
            return 0;
        }
        self.cancel_interaction_modes();
        let blueprint = self.blueprint.clone();
        let mut placed = 0;
        for entry in blueprint {
            let Some(x) = anchor.x.checked_add(entry.offset.0) else {
                continue;
            };
            let Some(y) = anchor.y.checked_add(entry.offset.1) else {
                continue;
            };
            let pos = GridPos::new(x, y);
            self.selected_building = Some(entry.building_type);
            if self.try_place_building(pos) {
                if let Some(building) = self
                    .grid
                    .get_mut(pos)
                    .and_then(|tile| tile.building.as_mut())
                {
                    let supports_policy = building.supports_overclock();
                    building.overclocked = entry.overclocked && supports_policy && !entry.standby;
                    building.input_priority = entry.input_priority && supports_policy;
                    building.standby = entry.standby && supports_policy;
                }
                placed += 1;
            }
        }
        self.cancel_interaction_modes();
        self.power_balance = self.net_power();
        if placed < self.blueprint.len() {
            self.notifications.warning(format!(
                "Blueprint placed {}/{}; invalid or unaffordable entries skipped",
                placed,
                self.blueprint.len()
            ));
        }
        placed
    }
}
