use super::*;

#[test]
fn compact_records_reserve_space_for_the_durable_log() {
    let area_y = HEADER_HEIGHT + 16.0;
    let grid_h = records_grid_height(600.0, area_y);
    let grid = Rect::new(16.0, area_y, 768.0, grid_h);
    let log_y = grid.bottom() + GRID_TO_LOG_GAP;

    assert_eq!(grid_h, 310.0);
    assert!(log_y + LOG_MIN_H <= 600.0 - FOOTER_RESERVE);
}

#[test]
fn records_do_not_draw_a_card_under_the_summary_line() {
    let view = Rect::new(16.0, 112.0, 768.0, 310.0);

    assert!(record_card_fits_view(112.0, CARD_H, view));
    assert!(record_card_fits_view(346.0, CARD_H, view));
    assert!(!record_card_fits_view(347.0, CARD_H, view));
}
