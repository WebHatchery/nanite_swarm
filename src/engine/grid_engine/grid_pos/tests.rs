use super::GridPos;

#[test]
fn in_bounds_rejects_negative_and_out_of_range() {
    assert!(GridPos::new(0, 0).in_bounds(4, 4));
    assert!(GridPos::new(3, 3).in_bounds(4, 4));
    assert!(!GridPos::new(4, 0).in_bounds(4, 4));
    assert!(!GridPos::new(0, 4).in_bounds(4, 4));
    assert!(!GridPos::new(-1, 0).in_bounds(4, 4));
}

#[test]
fn neighbors_are_four_directional() {
    let neighbors = GridPos::new(5, 5).neighbors();
    assert_eq!(
        neighbors,
        [
            GridPos::new(4, 5),
            GridPos::new(6, 5),
            GridPos::new(5, 4),
            GridPos::new(5, 6),
        ]
    );
}

#[test]
fn distance_is_manhattan() {
    assert_eq!(GridPos::new(0, 0).distance(GridPos::new(3, 4)), 7);
    assert_eq!(GridPos::new(2, 2).distance(GridPos::new(2, 2)), 0);
}

#[test]
fn index_roundtrip() {
    let width = 10;
    for i in 0..(width * 3) {
        let pos = GridPos::from_index(i as usize, width);
        assert_eq!(pos.to_index(width), i as usize);
    }
}
