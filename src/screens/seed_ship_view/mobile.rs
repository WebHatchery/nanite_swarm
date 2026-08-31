//! Compact Seed Ship yard for portrait and short desktop screens.

use super::*;
use macroquad_toolkit::ui::{draw_ui_text, is_fully_visible, truncate_text_to_width, ScrollArea};

mod stage;

const MOBILE_STAGE_HEIGHT: f32 = 88.0;
const MOBILE_STAGE_GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct CompactYardGeometry {
    pub panel_x: f32,
    pub panel_w: f32,
    pub view: Rect,
    pub stages_h: f32,
    pub content_h: f32,
    pub footer_y: Option<f32>,
}

pub(super) fn draw(
    state: &PlanetState,
    scroll: &mut ScrollArea,
    pulse: f32,
    screen_h: f32,
    action: SeedShipAction,
) -> SeedShipAction {
    draw_compact(state, scroll, pulse, screen_h, action, false)
}

pub(super) fn draw_compact_desktop(
    state: &PlanetState,
    scroll: &mut ScrollArea,
    pulse: f32,
    screen_h: f32,
    action: SeedShipAction,
) -> SeedShipAction {
    draw_compact(state, scroll, pulse, screen_h, action, true)
}

fn draw_compact(
    state: &PlanetState,
    scroll: &mut ScrollArea,
    pulse: f32,
    screen_h: f32,
    mut action: SeedShipAction,
    fixed_footer: bool,
) -> SeedShipAction {
    let geometry = compact_yard_geometry(
        screen_width(),
        screen_h,
        state.data().seed_ship.stages.len(),
        fixed_footer,
    );
    scroll.update(geometry.view, geometry.content_h);
    let offset_y = geometry.view.y - scroll.offset();
    let interactive = !scroll.absorbs_press();

    for (index, stage_def) in state.data().seed_ship.stages.iter().enumerate() {
        let y = offset_y + index as f32 * (MOBILE_STAGE_HEIGHT + MOBILE_STAGE_GAP);
        if is_fully_visible(
            Rect::new(geometry.panel_x, y, geometry.panel_w, MOBILE_STAGE_HEIGHT),
            geometry.view,
        ) {
            stage::draw(
                state,
                index,
                geometry.panel_x,
                y,
                geometry.panel_w,
                pulse,
                stage_def,
            );
        }
    }

    let footer_y = offset_y + geometry.stages_h + 12.0;
    if state.seed_ship.is_complete() {
        if !fixed_footer && footer_y + 44.0 >= geometry.view.y && footer_y <= geometry.view.bottom()
        {
            draw_ui_text(
                &truncate_text_to_width(
                    "The ship is whole. This world is spent.",
                    geometry.panel_w,
                    12.0,
                ),
                geometry.panel_x,
                footer_y + 18.0,
                12.0,
                Colors::SUCCESS,
            );
        }
    } else if !fixed_footer {
        let blocked = state.seed_ship_blocked_by().is_some();
        let label = if state.seed_ship.committed {
            "HALT DIVERSION"
        } else {
            "DIVERT PRODUCTION TO THE YARD"
        };
        if blocked {
            let gap = 8.0;
            let button_w = (geometry.panel_w - gap) * 0.5;
            let research_button = Rect::new(
                geometry.panel_x,
                footer_y,
                button_w,
                Dimensions::BUTTON_HEIGHT,
            );
            if is_fully_visible(research_button, geometry.view)
                && draw_button_sized(
                    research_button.x,
                    research_button.y,
                    research_button.w,
                    research_button.h,
                    "RESEARCH",
                )
                && interactive
            {
                action = SeedShipAction::OpenResearch;
            }
            let commitment_button = Rect::new(
                geometry.panel_x + button_w + gap,
                footer_y,
                button_w,
                Dimensions::BUTTON_HEIGHT,
            );
            if is_fully_visible(commitment_button, geometry.view)
                && draw_button_sized(
                    commitment_button.x,
                    commitment_button.y,
                    commitment_button.w,
                    commitment_button.h,
                    if state.seed_ship.committed {
                        "HALT"
                    } else {
                        "DIVERT"
                    },
                )
                && interactive
            {
                action = SeedShipAction::ToggleCommitment;
            }
        } else {
            let button = Rect::new(
                geometry.panel_x,
                footer_y,
                geometry.panel_w,
                Dimensions::BUTTON_HEIGHT,
            );
            if is_fully_visible(button, geometry.view)
                && draw_button_sized(button.x, button.y, button.w, button.h, label)
                && interactive
            {
                action = SeedShipAction::ToggleCommitment;
            }
        }
        let note = if blocked {
            "Tap RESEARCH to unlock the next stage."
        } else if state.seed_ship.committed {
            "Harvest is being poured into the hull."
        } else {
            "The yard is idle; harvested resources stay in the pool."
        };
        if footer_y + 64.0 >= geometry.view.y && footer_y + 44.0 <= geometry.view.bottom() {
            draw_ui_text(
                &truncate_text_to_width(note, geometry.panel_w, 11.0),
                geometry.panel_x,
                footer_y + Dimensions::BUTTON_HEIGHT + 18.0,
                11.0,
                Colors::TEXT_DIM,
            );
        }
    }
    if fixed_footer {
        action = draw_fixed_footer(
            state,
            geometry.panel_x,
            geometry.panel_w,
            geometry.footer_y.expect("compact desktop footer"),
            interactive,
            action,
        );
    }
    scroll.draw_scrollbar(geometry.view, geometry.content_h);

    if is_key_pressed(KeyCode::Escape) {
        action = SeedShipAction::Close;
    }
    if geometry.content_h > geometry.view.h && !fixed_footer {
        draw_ui_text(
            "Drag the yard to scroll",
            18.0,
            screen_h - 34.0,
            10.0,
            Colors::TEXT_DIM,
        );
    }
    draw_ui_text(
        "Tap Back to return",
        18.0,
        screen_h - 18.0,
        12.0,
        Colors::TEXT_DIM,
    );
    action
}

pub(super) fn compact_yard_geometry(
    screen_w: f32,
    screen_h: f32,
    stage_count: usize,
    fixed_footer: bool,
) -> CompactYardGeometry {
    let panel_x = 12.0;
    let panel_w = (screen_w - 24.0).max(80.0);
    let stages_h = stage_count.saturating_sub(1) as f32 * (MOBILE_STAGE_HEIGHT + MOBILE_STAGE_GAP)
        + MOBILE_STAGE_HEIGHT;
    let view_h = if fixed_footer {
        (screen_h - HEADER_HEIGHT - 116.0).max(120.0)
    } else {
        (screen_h - HEADER_HEIGHT - 70.0).max(1.0)
    };
    CompactYardGeometry {
        panel_x,
        panel_w,
        view: Rect::new(panel_x, HEADER_HEIGHT + 12.0, panel_w, view_h),
        stages_h,
        content_h: if fixed_footer {
            stages_h
        } else {
            stages_h + 92.0
        },
        footer_y: fixed_footer.then_some(screen_h - 84.0),
    }
}

fn draw_fixed_footer(
    state: &PlanetState,
    panel_x: f32,
    panel_w: f32,
    footer_y: f32,
    interactive: bool,
    mut action: SeedShipAction,
) -> SeedShipAction {
    if state.seed_ship.is_complete() {
        draw_ui_text(
            "The ship is whole. This world is spent.",
            panel_x,
            footer_y + 18.0,
            12.0,
            Colors::SUCCESS,
        );
        return action;
    }

    let blocked = state.seed_ship_blocked_by().is_some();
    if blocked {
        let gap = 8.0;
        let button_w = (panel_w - gap) * 0.5;
        if draw_button_sized(
            panel_x,
            footer_y,
            button_w,
            Dimensions::BUTTON_HEIGHT,
            "RESEARCH",
        ) && interactive
        {
            action = SeedShipAction::OpenResearch;
        }
        if draw_button_sized(
            panel_x + button_w + gap,
            footer_y,
            button_w,
            Dimensions::BUTTON_HEIGHT,
            if state.seed_ship.committed {
                "HALT DIVERSION"
            } else {
                "DIVERT PRODUCTION"
            },
        ) && interactive
        {
            action = SeedShipAction::ToggleCommitment;
        }
    } else if draw_button_sized(
        panel_x,
        footer_y,
        panel_w,
        Dimensions::BUTTON_HEIGHT,
        if state.seed_ship.committed {
            "HALT DIVERSION"
        } else {
            "DIVERT PRODUCTION TO THE YARD"
        },
    ) && interactive
    {
        action = SeedShipAction::ToggleCommitment;
    }
    let note = if blocked {
        "Tap RESEARCH to unlock the next stage. Drag the yard to scroll."
    } else if state.seed_ship.committed {
        "Harvest is being poured into the hull. Drag the yard to scroll."
    } else {
        "The yard is idle; drag the yard to scroll."
    };
    draw_ui_text(
        note,
        panel_x,
        footer_y + Dimensions::BUTTON_HEIGHT + 14.0,
        11.0,
        Colors::TEXT_DIM,
    );
    action
}

#[cfg(test)]
mod tests;
