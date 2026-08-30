use super::*;

#[test]
fn compact_layout_gives_the_tree_most_of_an_800_pixel_screen() {
    let layout = ResearchLayout::for_screen(800.0, 600.0, 72.0);
    assert!(layout.compact);
    assert_eq!(layout.tree.w, 768.0);
    assert!(layout.tree.h >= 250.0);
    assert!(layout.intel.y >= layout.tree.y + layout.tree.h);
}

#[test]
fn wide_layout_keeps_both_sidebars() {
    let layout = ResearchLayout::for_screen(1280.0, 720.0, 72.0);
    assert!(!layout.compact);
    assert_eq!(layout.intel.w, 280.0);
    assert_eq!(layout.legend.w, 260.0);
    assert!(layout.tree.w > 350.0);
}

#[test]
fn portrait_layout_separates_controls_tree_and_detail_drawer() {
    let layout = ResearchLayout::for_screen(360.0, 640.0, 72.0);

    assert!(layout.compact);
    assert_eq!(layout.tree.x, 12.0);
    assert_eq!(layout.tree.w, 336.0);
    assert!(layout.tree.y > 150.0);
    assert!(layout.tree.bottom() < layout.intel.y);
    assert!(layout.intel.bottom() <= 612.0);
}
