//! Session-only navigation for planetary gameplay drawers.

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

/// Which desktop drawer is currently covering one edge of the world view.
///
/// Desktop gameplay starts with no drawer open. The map remains the primary
/// surface until the player explicitly asks for construction or inspection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DesktopPanel {
    #[default]
    None,
    Build,
    Info,
}

/// The two touch-friendly sections of the solar-system screen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InterplanetaryMobilePanel {
    #[default]
    Worlds,
    Shipping,
}
