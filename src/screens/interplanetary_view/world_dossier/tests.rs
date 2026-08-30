use super::world_telemetry_height;

#[test]
fn refined_stock_gets_its_own_dossier_row() {
    assert_eq!(world_telemetry_height(), 70.0);
}
