use super::*;

#[test]
fn line_crossing_tree_is_clipped_to_both_edges() {
    let tree = Rect::new(10.0, 20.0, 100.0, 80.0);
    let (from, to) = clip_line(tree, vec2(-20.0, 60.0), vec2(140.0, 60.0)).unwrap();
    assert_eq!(from, vec2(10.0, 60.0));
    assert_eq!(to, vec2(110.0, 60.0));
}

#[test]
fn line_wholly_outside_tree_is_rejected() {
    let tree = Rect::new(10.0, 20.0, 100.0, 80.0);
    assert!(clip_line(tree, vec2(-20.0, 4.0), vec2(140.0, 4.0)).is_none());
}

#[test]
fn line_already_inside_tree_is_unchanged() {
    let tree = Rect::new(10.0, 20.0, 100.0, 80.0);
    let from = vec2(30.0, 40.0);
    let to = vec2(90.0, 70.0);
    assert_eq!(clip_line(tree, from, to), Some((from, to)));
}
