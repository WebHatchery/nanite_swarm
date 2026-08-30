//! Actionable locations for the planetary alert counter.
//!
//! The HUD already knows when logistics are stalled, starved, blocked, or
//! congested. This module turns that summary into a deterministic sequence of
//! tiles the player can inspect without searching the whole map. It also
//! surfaces upkeep before dust or corrosion becomes a silent production loss.

use crate::engine::{DroneState, GridPos};

use super::PlanetState;

/// Dust begins reducing work at the configured efficiency threshold. Acid has
/// no intermediate production penalty, so late corrosion uses the same
/// warning point as the network's dust leak response.
const CORROSION_ALERT_AT: f32 = 75.0;

impl PlanetState {
    /// Return buildings whose upkeep has started to affect the colony.
    ///
    /// The map already paints lower wear as a warning, but a player needs a
    /// single actionable threshold for the alert card and remote telemetry.
    /// Sort by map position so alert cycling remains stable across frames and
    /// save/load boundaries.
    pub fn upkeep_alert_positions(&self) -> Vec<GridPos> {
        let response = self.resolved_dust_response();
        let dust_threshold = response.efficiency_threshold.clamp(0.0, 100.0);
        let mut positions: Vec<_> = self
            .grid
            .iter_tiles()
            .filter_map(|(position, tile)| {
                let building = tile.building.as_ref()?;
                (building.dust >= dust_threshold || building.acid_wear >= CORROSION_ALERT_AT)
                    .then_some(position)
            })
            .collect();
        positions.sort_by_key(|position| (position.y, position.x));
        positions
    }

    /// Return the locations that can explain a live logistics alert.
    ///
    /// The order is intentional: a stranded crew is the most urgent thing to
    /// recover, followed by machines waiting for input or a free output pad,
    /// then the network tiles carrying too much traffic. Duplicates are
    /// removed so one tile is never visited twice in an alert cycle.
    pub fn alert_positions(&self) -> Vec<GridPos> {
        let mut positions = Vec::new();
        let mut add = |position: GridPos| {
            if !positions.contains(&position) {
                positions.push(position);
            }
        };

        for drone in self
            .drones
            .drones()
            .iter()
            .filter(|drone| drone.state == DroneState::Error)
        {
            add(drone.home);
        }
        for position in self.starved_factories() {
            add(position);
        }
        for position in self.blocked_factories() {
            add(position);
        }
        for position in self.upkeep_alert_positions() {
            add(position);
        }

        let capacity = self.config.buildings.conduit_capacity.max(1.0);
        let mut congested: Vec<_> = self
            .traffic
            .iter()
            .filter_map(|(&(x, y), load)| (*load as f32 > capacity).then_some(GridPos::new(x, y)))
            .collect();
        congested.sort_by_key(|position| (position.y, position.x));
        for position in congested {
            add(position);
        }

        positions
    }

    /// Select and return the next actionable alert location.
    ///
    /// Tapping the alert control again after the last item wraps around to the
    /// first one. Selecting an alert also drops build/demolition/relocation
    /// modes so the next tap on the map is safe inspection rather than an
    /// accidental construction command.
    pub fn focus_next_alert(&mut self) -> Option<GridPos> {
        let positions = self.alert_positions();
        let next = match self.selected_tile {
            Some(selected) => positions
                .iter()
                .position(|position| *position == selected)
                .map(|index| positions[(index + 1) % positions.len()])
                .or_else(|| positions.first().copied()),
            None => positions.first().copied(),
        }?;

        self.selected_tile = Some(next);
        self.selected_building = None;
        self.demolish_mode = false;
        self.relocation_source = None;
        Some(next)
    }
}

#[cfg(test)]
mod tests;
