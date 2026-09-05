# Toolkit migration

Campaign storage now uses `KeySaveStore` and `BackupChain` with the established
`_backup`, `_backup_2` and `_backup_3` keys. Migration and campaign validation run
before raw-copy rotation, without offline simulation or schema relabeling.
Future primaries block fallback and overwrite. A campaign explicitly recovered
from corruption can save again by restoring a validated raw backup first. Read
and write failures propagate; recovery generation remains visible to the player.

Camera pan, cursor-pinned wheel/pinch zoom and viewport constraints use the
toolkit's input-independent `CameraTransform`. The serialized pan/zoom fields,
zoom limits and game-specific 64-pixel visibility policy remain unchanged.
The existing touch recognizer still owns gesture claims and the game routes
gestures around its panels. Grid/tile layout and centering a selected tile remain
game responsibilities. Zoom at a reached limit preserves pan exactly.

Directive/tutorial and launch-caption wrapping use shared measured text layout.
HUD fitting uses shared measured truncation; launch caption centering now measures
with the same toolkit font used to draw it. Existing line limits and panel widths
are retained. Catalog loading, particles, gestures, notifications and other
generic runtime facilities already use the toolkit.

Validation includes all game tests and strict all-feature Clippy, default
Windows/WebGL Preview publishing and tracking, plus desktop/mobile tutorial and
launch captures. Recovery fixtures cover all three exact-byte generations,
future primaries, corrupt candidates, failed rotation and saving after recovery.
