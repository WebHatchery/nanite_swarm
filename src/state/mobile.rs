//! Session-only navigation for the portrait planetary HUD.

/// Which full-width mobile drawer is currently covering the planetary map.
/// This is intentionally not save data: a loaded world should reopen on the
/// map, where the player can choose the drawer that matters now.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MobilePanel {
    #[default]
    Map,
    Build,
    Info,
}
