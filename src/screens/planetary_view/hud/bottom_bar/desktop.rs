//! Desktop entry point for the compact bottom edge rail.

use crate::data::UiTheme;
use crate::state::PlanetState;

use super::super::super::metrics::HudMetrics;
use super::super::PanelColors;
use super::ClockAction;

#[must_use]
pub(super) fn draw(
    state: &PlanetState,
    screen_w: f32,
    screen_h: f32,
    theme: &UiTheme,
    metrics: HudMetrics,
    colors: &PanelColors,
) -> ClockAction {
    super::focus::draw(state, screen_w, screen_h, theme, metrics, colors)
}
