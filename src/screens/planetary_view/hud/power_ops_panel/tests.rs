use super::records_action;
use crate::screens::planetary_view::PlanetaryAction;
use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::ui::Pointer;

#[test]
fn achievements_row_opens_records_for_a_touch_release() {
    let pointer = Pointer {
        position: vec2(100.0, 100.0),
        released: true,
        ..Pointer::default()
    };

    assert_eq!(
        records_action(&pointer, Rect::new(78.0, 78.0, 44.0, 44.0)),
        PlanetaryAction::OpenRecords
    );
}

#[test]
fn achievements_row_does_not_open_when_the_release_misses() {
    let pointer = Pointer {
        position: vec2(140.0, 100.0),
        released: true,
        ..Pointer::default()
    };

    assert_eq!(
        records_action(&pointer, Rect::new(78.0, 78.0, 44.0, 44.0)),
        PlanetaryAction::None
    );
}
