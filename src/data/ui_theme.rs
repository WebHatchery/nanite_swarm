//! UI theme values loaded from JSON.

use serde::{Deserialize, Serialize};

/// Themes written before alloy existed still need a colour for it.
fn default_alloy_color() -> [f32; 4] {
    [0.85, 0.58, 0.30, 1.0]
}

fn default_components_color() -> [f32; 4] {
    [0.78, 0.55, 0.95, 1.0]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTheme {
    pub colors: UiColors,
    pub layout: UiLayout,
    pub typography: UiTypography,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    pub background: [f32; 4],
    pub panel: [f32; 4],
    pub panel_deep: [f32; 4],
    pub panel_inner: [f32; 4],
    pub border: [f32; 4],
    pub border_bright: [f32; 4],
    pub text: [f32; 4],
    pub text_dim: [f32; 4],
    pub primary: [f32; 4],
    pub primary_soft: [f32; 4],
    pub success: [f32; 4],
    pub warning: [f32; 4],
    pub error: [f32; 4],
    pub energy: [f32; 4],
    pub minerals: [f32; 4],
    pub data: [f32; 4],
    pub biomass: [f32; 4],
    #[serde(default = "default_alloy_color")]
    pub alloy: [f32; 4],
    #[serde(default = "default_components_color")]
    pub components: [f32; 4],
    pub shadow: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayout {
    pub top_bar_height: f32,
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub bottom_bar_height: f32,
    pub panel_gap: f32,
    pub panel_padding: f32,
    pub tile_size: f32,
    pub build_row_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTypography {
    pub title: f32,
    pub section: f32,
    pub body: f32,
    pub small: f32,
    pub value: f32,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            colors: UiColors {
                background: [0.015, 0.027, 0.035, 1.0],
                panel: [0.018, 0.075, 0.095, 0.92],
                panel_deep: [0.006, 0.025, 0.035, 0.96],
                panel_inner: [0.025, 0.11, 0.14, 0.78],
                border: [0.0, 0.56, 0.78, 0.55],
                border_bright: [0.0, 0.78, 1.0, 0.92],
                text: [0.82, 0.9, 0.96, 1.0],
                text_dim: [0.48, 0.64, 0.72, 1.0],
                primary: [0.0, 0.78, 1.0, 1.0],
                primary_soft: [0.13, 0.62, 0.82, 1.0],
                success: [0.31, 0.88, 0.5, 1.0],
                warning: [1.0, 0.62, 0.18, 1.0],
                error: [1.0, 0.24, 0.18, 1.0],
                energy: [1.0, 0.67, 0.19, 1.0],
                minerals: [0.14, 0.7, 1.0, 1.0],
                data: [0.18, 0.78, 1.0, 1.0],
                biomass: [0.47, 0.92, 0.32, 1.0],
                alloy: default_alloy_color(),
                components: default_components_color(),
                shadow: [0.0, 0.0, 0.0, 0.45],
            },
            layout: UiLayout {
                top_bar_height: 86.0,
                left_panel_width: 286.0,
                right_panel_width: 316.0,
                bottom_bar_height: 74.0,
                panel_gap: 10.0,
                panel_padding: 12.0,
                tile_size: 28.0,
                build_row_height: 76.0,
            },
            typography: UiTypography {
                title: 20.0,
                section: 14.0,
                body: 12.0,
                small: 10.0,
                value: 22.0,
            },
        }
    }
}
