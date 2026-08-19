use super::*;

#[test]
fn procedural_star_hash_stays_in_normalized_space() {
    for seed in 0..500 {
        assert!((0.0..=1.0).contains(&hash01(seed)));
    }
}

#[test]
fn orbit_points_keep_the_declared_ellipse_radius_before_rotation() {
    let center = vec2(40.0, 70.0);
    let point = orbit_position(center, 20.0, 8.0, 0.0, 31.0);
    assert!(((point - center).length() - 20.0).abs() < 0.001);
}
