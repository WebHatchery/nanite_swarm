//! Top edge: world identity, compact resources, and navigation.

use crate::data::UiTheme;
use crate::state::PlanetState;

use super::super::metrics::HudMetrics;
use super::super::PlanetaryAction;
use super::PanelColors;

mod focus;
mod mobile;
mod notices;
mod readouts;

pub(super) use notices::{draw_mobile_offline_report, mobile_offline_work_label};
#[cfg(test)]
use readouts::{
    compact_hidden_resource_readout, compact_resource_amount, compact_resource_readout,
    resource_rate_label,
};
use readouts::{
    compact_resource_readout_for_screen, mobile_arrival_guidance_label, mobile_research_status,
    mobile_status_text,
};

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct TopBarLayout {
    compact: bool,
    abbreviated_labels: bool,
    brand_width: f32,
    actions_width: f32,
    card_gap: f32,
    card_width: f32,
    card_count: usize,
}

#[cfg(test)]
fn top_bar_layout(screen_width: f32, top_bar_height: f32, panel_gap: f32) -> TopBarLayout {
    let compact = screen_width < 1180.0 || top_bar_height < 80.0;
    let narrow = screen_width < 720.0;
    let brand_width = if narrow {
        150.0
    } else {
        (screen_width * 0.18).clamp(188.0, 318.0)
    };
    let actions_width = if narrow {
        236.0
    } else {
        (screen_width * 0.25).clamp(304.0, 448.0)
    };
    let card_gap = if compact { 6.0 } else { 10.0 };
    let card_count = if compact { 4 } else { 6 };
    let cards_x = brand_width + panel_gap;
    let action_x = screen_width - actions_width - 8.0;
    let card_area_width = (action_x - cards_x - panel_gap).max(0.0);
    let card_width = ((card_area_width - card_gap * (card_count - 1) as f32) / card_count as f32)
        .max(if narrow { 42.0 } else { 52.0 });
    TopBarLayout {
        compact,
        abbreviated_labels: narrow,
        brand_width,
        actions_width,
        card_gap,
        card_width,
        card_count,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    state: &mut PlanetState,
    screen_w: f32,
    metrics: HudMetrics,
    theme: &UiTheme,
    colors: &PanelColors,
) -> PlanetaryAction {
    if metrics.mobile {
        mobile::draw(state, screen_w, metrics, theme, colors)
    } else {
        focus::draw(state, screen_w, metrics, theme, colors)
    }
}

#[cfg(test)]
mod tests;
