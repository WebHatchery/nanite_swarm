//! Portrait HUD drawers, keeping the map and its controls reachable on phones.

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::directives::Directive;
use crate::engine::GridPos;
use crate::state::{MobilePanel, PlanetState};

use super::super::metrics::HudMetrics;
use super::super::PlanetaryAction;
use super::RightStackLayout;
use super::{build_palette, directive_panel, inspector_panel, power_ops_panel, PanelColors};

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    state: &mut PlanetState,
    screen_w: f32,
    screen_h: f32,
    hovered_pos: Option<GridPos>,
    directive: &Directive,
    textures: &GameTextures,
    theme: &UiTheme,
    colors: &PanelColors,
    metrics: HudMetrics,
) -> PlanetaryAction {
    match state.mobile_panel {
        MobilePanel::Map => {
            if state.tutorial_current().is_some() && !state.tutorial_hidden {
                let right = RightStackLayout::mobile_tutorial(screen_w, metrics);
                directive_panel::draw(state, directive, theme, colors, &right);
            }
        }
        MobilePanel::Build => {
            let panel_metrics = metrics.mobile_build_panel(screen_w);
            build_palette::draw(
                state,
                theme,
                textures,
                panel_metrics,
                colors,
                screen_h,
                true,
            );
        }
        MobilePanel::Info => {
            let panel_metrics = metrics.mobile_info_panel(screen_w);
            let right = RightStackLayout::compute(screen_w, screen_h, panel_metrics);
            inspector_panel::draw(state, hovered_pos, textures, theme, colors, &right);
            let ops_action = power_ops_panel::draw(state, theme, colors, &right);
            if ops_action != PlanetaryAction::None {
                return ops_action;
            }
            directive_panel::draw(state, directive, theme, colors, &right);
        }
    }
    PlanetaryAction::None
}
