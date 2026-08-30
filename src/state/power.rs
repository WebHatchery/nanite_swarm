//! Planet-owned power refreshes.

use super::game_state::PlanetState;

impl PlanetState {
    /// Recompute connectivity with the dust thresholds supplied by the
    /// planet's research, rather than the engine's standalone defaults.
    pub(crate) fn refresh_power_grid(&mut self) {
        let dust_response = self.resolved_dust_response();
        self.grid.update_power_grid_with(&dust_response);
    }
}
