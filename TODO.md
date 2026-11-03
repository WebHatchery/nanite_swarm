# Nanite Swarm — TODO

- [ ] Migrate the 111 test source files and test-only helpers under `src/` to
  crate-root `tests/`, removing production `#[cfg(test)]` hooks. Introduce
  `src/lib.rs` with intentional public APIs used by `main.rs` and the tests;
  preserve regression coverage without exposing all internals. Migrate
  `src/data/mod.rs` and `src/ui/mod.rs` to named module files when restructuring
  those roots. Complete this migration before expanding coverage (§2.3, §11.4).
- [ ] Review the 742 existing test cases by cohesive feature, including campaign,
  persistence, seed ships, and logistics. Consolidate related inputs into
  table-driven cases toward five tests per feature; retain distinct regressions
  and document why any feature needs more than five cases (§11.3).
- [ ] Replace text-loading followed by unlabeled parsing in `src/data/defs.rs`
  and `src/data/strings.rs` with typed toolkit JSON loaders. Preserve source
  labels and report parse failures instead of discarding errors into empty
  content/defaults; retain project-owned semantic validation. Remove redundant
  platform loader facades in `src/data/mod.rs` and their startup branches
  (§5.3, §6.2).
- [ ] Move `GameData::research_tree` and `ResearchNodeDef::to_node` from
  `src/data/defs.rs` and `src/data/defs/research.rs` into the research engine so
  data schemas no longer import engine types (§2.1).
- [ ] Make planetary input/HUD and settings renderers return explicit actions
  for the application dispatcher instead of mutating gameplay/settings while
  drawing. Start with `src/screens/planetary_view.rs`, its `input.rs` and
  `hud/build_palette/quick_actions.rs`, and `src/screens/settings_menu/`;
  extract cohesive handlers so functions such as `render_planetary_view` and
  `handle_input` stay within 100 lines (§4.1, §5.1, §7).
- [ ] Move remaining player-facing labels, help text, and notifications into
  typed JSON under `assets/`, extending `PlayerStrings` beyond the main menu.
  Cover settings and planetary HUD text, including bulk-action labels and
  relocation feedback; keep tutorial wording aligned with visible controls
  (§5.3, §7.5).
- [ ] Remove unused UI helpers such as `draw_button` in `src/ui/components.rs`
  and audit `#[allow(dead_code)]` in UI metrics, planetary HUD, and capture
  modules. Delete obsolete code or relocate test-only helpers; explain any
  intentional remaining lint allowances with comments (§1.4, §10.2).
- [ ] Add purpose-level `//!` documentation to the 23 production modules missing
  it, including `src/data/strings.rs`, `src/display.rs`, and
  `src/game_data_validation.rs`; correct `tests/code_standards.rs`'s obsolete
  claim that the source-size gate counts only non-test lines (§2.2, §9.2).

## UI_STYLE review — 2026-09-20

Audit only; no UI implementation or task completion is implied. The eight
existing architecture tasks above are retained. Coordinate renderer/action and
string changes with those tasks rather than creating duplicate migrations.
Read AGENTS.md, UI_STYLE.md, CODE_STANDARDS.md, GAME_DEVELOPMENT_GUIDE.md,
README.md and gdd.md; no project-local PROJECT_AGENTS.md was found.

Evidence: inspected current screen dispatch, planetary render/input/layout and
camera code, HUD drawers/rails, research, solar map, menu and notification code.
Visually inspected existing captures in `docs/verification/`: `ui_gameplay`,
`ui_focus`, `ui_desktop_build`, `ui_desktop_info`, `ui_research`,
`ui_interplanetary`, `ui_toolkit_tutorial_desktop` (1280x720 PNGs),
`ui_mobile_narrow`, `ui_mobile_info`, `ui_mobile_research_narrow`,
`ui_mobile_save_failed_menu` (360x640), `ui_mobile_seedship` (320x480), and
`ui_mobile_mainmenu_narrow` (240x400). These are historical captures, not a
fresh run: older tutorial captures show HIDE and different step counts while
current directive rendering does not. Do not treat them as current interaction
verification. No game launch, browser/touch exercise or publish was performed
for this documentation-only audit.

Implementation verification baseline (proposed, not an existing support promise):
1280x720 normal desktop; 960x540 short desktop; 360x640 phone; 320x480 small
phone; 240x400 legacy stress case. Check 599/600px and 899/900px widths around
planetary layout thresholds, plus 959/960px for Solar System. Declare actual
supported minimums in U1; use those as acceptance gates rather than silently
claiming that every legacy capture size is supported. Verify native and actual
embedded browser canvas sizes, default/enlarged text, display scaling and
touch-only paths. After each meaningful implementation slice run `./publish.ps1`
without parameters; replace equivalent captures directly in `docs/verification/`
and record sizes, interactions, results and limitations. Preserve essential
costs, warnings, undo/cancel, save retry and launch/delete confirmation paths.

Follow-up review of the existing audit (2026-09-20): retained U1–U10 and all
architecture tasks without marking any implementation complete. Rechecked the
current phase dispatcher, camera/metrics, HUD orchestration, research detail,
Solar System composition, menu, toast, yard, ending, Records and settings code.
Reinspected historical `ui_gameplay`, `ui_desktop_info`, `ui_research`,
`ui_interplanetary`, `ui_mobile_narrow`, `ui_mobile_research_narrow`, and
`ui_mobile_seedship`. Additional visual evidence: `ui_seedship` (360x640,
despite its unsuffixed name), `ui_desktop_ending` (1280x720),
`ui_mobile_ending_very_short` (360x400), `ui_mobile_settings_narrow` (240x400),
`ui_mobile_records_narrow` and `ui_mobile_factory_deck` (360x640).
These PNG dimensions are evidence sizes, not a support declaration. U11–U13
extend the existing audit; task IDs remain stable while placement follows
implementation dependencies. No fresh runtime or touch verification was added.

### Verified findings — code and historical visuals, not current play verification

Start with U1's composition/support decisions, then U2–U5's shared HUD and
interaction work. Apply those decisions to research, yard, travel and secondary
screens below. U10 is the remaining evidence gate, not a duplicate redesign.

- [ ] **U1 — Frame the colony around the current decision and document screen briefs.**
  **Scope/files:** New campaign, world arrival, normal map and drawer transitions;
  `README.md`, `gdd.md`, `src/state/camera.rs`,
  `src/state/game_state/construction.rs`, `src/screens/planetary_view/metrics.rs`,
  `input.rs`, and `hud.rs::center_camera_on_position`.
  **Evidence/problem:** README/GDD lack the seven-part UI_STYLE screen briefs
  and explicit viewport support contract. The opening capture places a small
  revealed island left of center with extensive fog and unused background;
  removing drawers has not itself made the colony the dominant focus.
  Construction uses `Camera::default()` (zero pan, zoom 1), and the clamp only
  requires 64 pixels of map to remain visible. This permits poor framing;
  the precise first-run experience still needs current runtime confirmation.
  **Change:** Record each phase's decision, focus, action, supporting/deferred
  facts, layout/camera and touch feedback. Establish normal/minimum sizes.
  Center and size the opening Core, drill site and connecting route in the
  usable map viewport; adapt framing for arrival and inspection. Preserve
  deliberate player pan/zoom on ordinary frames and saved-world revisits.
  Keep CORE as an explicit recovery control. Recompute the usable viewport
  when drawers open, without moving the selected target beneath them.
  **Acceptance:** Colony and next construction target are the first things
  noticed; normal play has at most 2–3 strongly emphasized regions. The player
  can see and select a useful route without first fixing the camera.
  **Verify:** Baseline sizes; opening, developed factory, Core ring and hostile
  arrival; drag/pinch, CORE, select a tile, open/close BUILD and INFO, resize,
  and confirm picking still matches the displayed tile. Capture before/after.

- [ ] **U2 — Recompose the planetary command rails and disclose advanced tools contextually.**
  **Depends on:** U1's screen brief; coordinate U3/U4 before final geometry.
  **Scope/files:** Normal play and BUILD; `hud/top_bar/focus.rs`,
  `hud/top_bar/mobile.rs`, `hud/bottom_bar/focus.rs`,
  `hud/bottom_bar/mobile.rs`, `hud/build_palette.rs` and
  `hud/build_palette/quick_actions.rs` under `src/screens/planetary_view/`.
  **Evidence/problem:** Current top rails group MENU with BUILD/RESEARCH/SHIP/MAP
  at the same emphasis; desktop repeats BUILD and INFO at both edges.
  BUILD presents demolition, blueprint, relocation, box selection and locked
  automation controls even in the captured one-drill opening. The bottom
  recovery slot alternates NEXT/CANCEL/RETRY/SAVE NOW/JUMP; an alert can replace
  CANCEL while a placement mode is armed. This mixes gameplay with utilities,
  consumes attention and makes recovery meaning unstable.
  **Change:** Give building/route work one command area and keep Menu/save
  recovery visually separate in a quiet, labeled utility area. Remove duplicate
  BUILD/INFO controls and the button-shaped, non-actionable ALERT 0 readout.
  Keep mode CANCEL independently reachable during every warning. Show relocation
  and district actions with a relevant selection; introduce automation when
  unlocked and put blueprint tools behind a visible secondary disclosure.
  `SAVE BP` is a gameplay blueprint action, not campaign saving: label it clearly.
  Remove repeated build instructions after first-use teaching; retain Help.
  **Acceptance:** The current build/research/expansion action has clear emphasis,
  Menu never shares its action group, and an early player is not presented with
  every factory tool. Costs, prerequisites and repair paths remain discoverable.
  **Verify:** Baseline sizes; touch from BUILD through placement, cancel, undo,
  selection/relocation, blueprint and automation. Trigger save failure during
  placement and confirm both cancel and retry remain reachable without a
  keyboard; check advanced tools remain available in dense late-game states.

- [ ] **U3 — Make onboarding visible in the default view and reserve input space only for visible overlays.**
  **Depends on:** U1/U2 composition.
  **Scope/files:** Opening bootstrap, completed tutorial, mobile map;
  `src/screens/planetary_view/hud.rs::draw_ui_panels`, `hud/mobile.rs`,
  `hud/directive_panel.rs`, `metrics.rs::is_cursor_over_ui_with_help`,
  `src/state/tutorial.rs`, `assets/tutorial.json`.
  **Evidence/problem (current code):** New campaigns start in desktop focus
  mode with no drawer; desktop step instructions are rendered only inside INFO,
  while the top rail merely says CORE BOOTSTRAP ACTIVE. On mobile, hit testing
  always reserves the 132px tutorial rectangle in Map mode, although drawing
  checks whether a tutorial exists and whether Help/FLOW/DECK is open. Thus
  invisible tutorial space can block map taps and gesture starts after completion.
  **Change:** Present the exact current instruction beside its relevant visible
  action in default play, without forcing the entire INFO stack open. Collapse
  learned prose automatically and provide a visible Help route to recall it.
  Compute overlay visibility and hit regions from the same layout/state used
  to draw them, including offline reports and diagnostics.
  **Acceptance:** A fresh player can follow the bootstrap without guessing which
  drawer contains the lesson. Completed lessons disappear; every uncovered map
  tile is interactive. Active guidance and warnings still block click-through.
  **Verify:** Touch-only opening through all current tutorial steps, completed
  tutorial and hostile-world arrival at baseline sizes. After completion, pan a
  selectable tile into the former tutorial area and tap/drag it; repeat with
  Help, offline report, FLOW and DECK opened and closed. These paths were not
  exercised during this audit; confirm the code finding in the running game.

- [ ] **U4 — Restore readable resource decisions and unambiguous, usable touch controls.**
  **Depends on:** U1/U2 space allocation; do not solve this by shrinking text.
  **Scope/files:** Planetary rails and compact layouts; `hud/top_bar/focus.rs`,
  `hud/top_bar/readouts.rs`, `hud/bottom_bar/mobile.rs`, `hud/bottom_bar/focus.rs`,
  `src/ui/components.rs`, `assets/ui_theme.json`, `README.md`.
  **Evidence/problem:** Desktop rates use 7px text; mobile packs six resource
  pools into a truncated 7–7.5px line. Desktop omits Alloy/Biomass below a 500px
  resource strip and hides the whole strip below 180px; the fixed 386px action
  group leaves little room at intermediate widths. The active desktop renderer
  also omits capacities/FULL despite older helpers and README promises.
  Mobile buttons are drawn at 22–30px high. Toolkit `touch_area` expands hit
  regions, so actual missed taps/overlap are unverified, not established defects.
  Separately, mobile `-`/`+` dispatch Slower/Faster while Help describes zoom;
  desktop uses them for zoom. That semantic mismatch is code-confirmed.
  **Change:** Prioritize power and action-relevant resources, with a visible
  resource disclosure for exact totals/capacities/rates. Keep shortages beside
  build/research/ship costs and saturation warnings near affected production.
  Reflow the intermediate-width rail before it drops all resource information.
  Establish readable text and touch target floors (target 44 CSS pixels for
  required controls), with room between hit areas. Give zoom and simulation
  speed distinct labels/controls and show the current speed; align Help/README.
  **Acceptance:** Players can read affordability, capacity pressure, current
  speed and zoom intent without deciphering tiny abbreviations. No required
  resource becomes inaccessible at a breakpoint; one tap activates one control.
  **Verify:** All baseline/breakpoint sizes, large stockpiles and long labels;
  tap adjacent buttons at edges with enlarged text/display scaling. Confirm
  zoom changes the map without time acceleration, speed changes time only,
  and touch-only zoom, pause, CORE and recovery remain accessible.

- [ ] **U5 — Turn INFO into a focused inspector instead of a four-panel dashboard.**
  **Depends on:** U1–U4; reuse their guidance, resources and command homes.
  **Scope/files:** Desktop/mobile INFO, selected terrain/building and failures;
  `hud.rs::RightStackLayout`, `hud/inspector_panel.rs` and children,
  `hud/power_ops_panel.rs`, `hud/directive_panel.rs`, `src/ui/components.rs`.
  **Evidence/problem:** Both INFO captures and current orchestration stack
  Selected Structure, Power Grid, Operations and Tutorial/Directive in equally
  bright bordered boxes. Generic counts and achievements compete with the
  selected object's condition; proportional panel allocation squeezes the
  inspector on short screens instead of deferring secondary systems.
  **Change:** Give the selected object its name, local effect, costs/payoffs,
  condition and relevant actions in one primary inspector. Move global grid
  detail to FLOW/power disclosure, achievement counts to Records, and guidance
  to U3's contextual location. Retain local power/wear/route blockers beside
  the object and link to wider diagnostics. Use spacing and quieter separators
  instead of a decorated box and heading for each group; scroll long details.
  **Acceptance:** Opening INFO answers what this tile does, what constrains it,
  and what can be done next. The map stays dominant on desktop; mobile has one
  readable, dismissible inspection decision rather than four competing panels.
  **Verify:** Baseline sizes; Drill, multi-input processor, Core, forest tradeoff,
  surveyed feature and damaged machinery. Tap selection → INFO → action → map;
  verify harvest consequences, purge confirmation, repair and local bonuses
  remain legible, including dense/long descriptions and enlarged text.

- [ ] **U6 — Focus research on the next choice and contain long explanations.**
  **Depends on:** U1 brief and U4 typography/target rules.
  **Scope/files:** Neural Network desktop/compact, active and locked research;
  `src/screens/research_view.rs`, `research_view/desktop.rs`, `sidebar.rs`,
  `sheet.rs`, `nodes.rs`, `layout.rs`, `compact.rs` and its node-info renderer.
  **Evidence/problem:** `ui_research.png` shows the active name twice, a long
  description spilling from the left panel into the tree, and the stat list
  extending below its right panel. `desktop::draw` draws the description without
  wrapping; `draw_swarm_sheet` has no content-height/scroll boundary. Legend and
  full stat sheet remain prominent beside the tree, which displays distant
  locked systems and seven-character labels. The phone capture reserves large
  areas for legend/instructions and details, leaving only a tree fragment.
  **Change:** Make the useful research frontier and one selected-node detail
  area primary. Merge active-job summary with details when they refer to the
  same node. Wrap/scroll descriptions and effects within bounds. Move the full
  stat sheet to visible optional detail and reduce the legend to a compact key.
  Introduce distant branches gradually or through an explicit full-tree view;
  keep immediate prerequisites, full selected names, cost and power-collapse
  lock reasons visible. Replace repeated gesture/footer prose with first-use Help.
  **Acceptance:** Players can identify the next useful technology and its cost
  without text crossing the network or unrelated stats taking equal emphasis.
  Tap inspection exposes the same necessary information as hover.
  **Verify:** Baseline sizes and enlarged text; opening target, long Adaptive
  Clocking description, late tree, parked progress and collapse lock. Touch pan,
  zoom, select/start research, and Back to both gameplay and Seed Ship.

- [ ] **U11 — Keep the active Seed Ship decision visible and hand off completed ships to travel.**
  **Depends on:** U1 screen brief, U4 readable controls, and U6 research return path;
  coordinate the completed-ship destination with U7.
  **Scope/files:** Seed Ship yard, blocked/producing/completed stages;
  `src/screens/seed_ship_view.rs::render_seed_ship_view`, `draw_stage`,
  `seed_ship_view/mobile.rs::draw_compact`, `compact_yard_geometry`,
  `seed_ship_view/mobile/stage.rs`, and `src/main/update/phases.rs`.
  **Evidence/problem:** Historical 360x640 and 320x480 yard captures give
  sealed stages substantial card space alongside the active stage. Current phone
  layout puts RESEARCH and DIVERT/HALT after the entire scrolling stage list;
  only short desktop uses a fixed action footer. Reading the active stage and
  controlling production can therefore require different scroll positions.
  In current code, completed yards replace these actions with prose saying
  the world is spent, while the remaining Back action returns to the colony.
  There is no direct Solar System action or explicit MAP handoff there. The
  completed state was inspected in code only, not in a running game or PNG.
  **Change:** Make the active stage, paid/required resources, boon, research
  blocker and diversion consequence the primary yard region. Collapse built
  stages and defer future-stage descriptions behind a visible stage overview;
  retain discoverable prerequisites and planning costs. Keep current production
  controls outside the phone scroll area. On completion, replace diversion
  controls with a clearly labeled Solar System handoff, showing that destination
  selection and launch confirmation happen there; preserve driver-readiness
  warnings and ordinary Back navigation. Do not auto-launch or change costs.
  **Acceptance:** Players can understand and halt/divert the current stage
  without scrolling past future technology. A finished ship has an obvious next
  gameplay action, while the colony remains available for continued production.
  **Verify:** 1280x720, 960x540, 360x640, 320x480 and 599/600px height/width
  transitions; default and 1.5x text. Touch through each stage, research/Back,
  scroll without toggling diversion, halt/resume, complete with online/offline
  driver, open Solar System, cancel a launch and return. Capture blocked,
  producing and complete states; confirm stage progress survives navigation.

- [ ] **U7 — Separate Solar System travel/launch decisions from freight editing.**
  **Depends on:** U1 brief; preserve the existing compact Worlds/Shipping split.
  **Scope/files:** Desktop Solar System and compact dossiers;
  `src/screens/interplanetary_view.rs`, `interplanetary_view/planet_panel.rs`,
  `shipping_panel.rs`, `world_dossier.rs` and their children, `mobile.rs`.
  **Evidence/problem:** Desktop simultaneously renders a world list, persistent
  shipping editor, orbit map, dense dossier and separate legend; the inspected
  capture gives all of them labeled bordered surfaces. Players choosing a
  destination must scan unrelated reserve/schedule controls and telemetry.
  This is a hierarchy finding, not an objection to freight comparisons when
  managing shipping is the current decision.
  **Change:** Use an explicit Worlds/Shipping task choice on desktop too, or
  open freight editing only from a selected route. Give destination comparison
  and its travel/launch action the main area; show launch cost, hostile-world
  constraints and driver readiness beside confirmation. Reveal stockpile,
  milestone and freight detail on demand. In Shipping, retain source identity,
  destination, reserve, nominal rate and next actionable blocker together.
  Remove redundant Planetary Map heading and permanent footer tutorial prose.
  **Acceptance:** At most 2–3 strongly emphasized areas serve the current task;
  selecting a destination does not require parsing a freight configuration form.
  No readiness, hazard or cargo-recovery warning is lost through disclosure.
  **Verify:** 1280x720, 960x540, 360x640, 320x480 and 959/960px transitions;
  touch travel, prepare/cancel/confirm launch, select a lane/source, change a
  remote order/reserve, and recover a missing/offline pad or driver.

- [ ] **U8 — Bound event feedback and keep lasting consequences in state/history.**
  **Depends on:** U1/U3 overlay geometry; coordinate with U5–U7 layouts.
  **Scope/files:** Gameplay, research, Solar System and offline return;
  `src/screens/toasts.rs`, `planetary_view/upkeep_render.rs::draw_notifications`,
  `planetary_view/hud/top_bar/notices.rs`, `src/main/update.rs`,
  `src/screens/records_view/log.rs`.
  **Evidence/problem:** Inspected focus/Solar System captures show four or five
  simultaneous success cards covering world/tree space. Current desktop toast
  limit is unbounded (`usize::MAX`); compact rendering merely takes the last
  three. The mobile anchor is top-bar +96px while current tutorial height is
  132px, so the claimed reserved strip does not use actual guidance bounds.
  Notifications are correctly transient already; do not convert them into
  permanent status panels or claim all results currently lack history.
  **Change:** Queue/coalesce related achievements and completions into one or
  two brief notices, with warning priority and placement outside current
  targets/guidance. Use live overlay bounds. Keep unresolved save/power/route
  conditions in their state/action homes, and make important detailed outcomes
  retrievable through existing Records/offline history, extending it only
  where inspection demonstrates a gap.
  **Acceptance:** A burst of successes cannot obscure the next action or bury
  a critical failure; after it expires the changed state remains understandable.
  Calm play has no event prose remaining as wallpaper.
  **Verify:** Baseline sizes; burst achievements plus tutorial advancement,
  research completion, collapse, save failure and offline catch-up. Allow all
  toasts to expire, inspect the lasting result/history, and tap the intended
  underlying action without accidental activation or blocked content.

- [ ] **U9 — Give the title/menu a primary play action and separate save management.**
  **Depends on:** U1 menu brief and U4 readability rules.
  **Scope/files:** Fresh/saved campaign menus, save failure and delete confirmation;
  `src/screens/main_menu.rs`, `main_menu/mobile.rs`, `src/main/update/phases.rs`,
  `src/data/strings.rs` and associated player strings.
  **Evidence/problem:** Current menus build a uniform command stack containing
  slot cycling, New Game, Continue, Load, Save, Settings, Delete and Quit.
  Continue/Load dispatch to the same load path. Inspected phone captures give
  every entry the same emphasis and shrink the stack to 20px rows at 240x400;
  a separate Mission Briefing card remains even when its text is truncated.
  This retains a demonstration-style command/briefing composition; template
  ancestry itself was not established and should not be claimed as a defect.
  **Change:** Emphasize New Game for an empty slot and the appropriate existing
  campaign entry otherwise. Put slot selection/load/delete/save in a distinct
  save-management area; consolidate redundant Continue/Load presentation while
  preserving intentional behavior. Keep Settings/Quit quiet and separate.
  Reduce briefing to a readable short introduction; scroll secondary controls
  at minimum size instead of squeezing them. Keep failures beside Save/Retry
  and explicitly preserve two-step deletion and the active-campaign save guard.
  **Acceptance:** Players immediately see how to enter play, can identify the
  selected save, and cannot mistake Delete or Settings for the primary action.
  Recovery instructions remain complete rather than truncated away.
  **Verify:** Baseline sizes, saved/empty slots, active campaign, failed save,
  backup recovery and armed deletion; touch through New Game, Continue/load,
  change slot, retry, cancel/confirm delete and Settings/Back. Distinguish native
  Quit from the browser menu when checking available actions.

- [ ] **U12 — Make the completion report readable and separate continued play from menu navigation.**
  **Depends on:** U1 completion brief and U4 typography/target rules.
  **Scope/files:** Desktop/phone campaign completion;
  `src/screens/campaign_complete_view.rs::render_campaign_complete_view`,
  `campaign_complete_view/mobile.rs::draw`, `report.rs`, and
  `src/main/update/phases.rs::update_campaign_complete`.
  **Evidence/problem:** The desktop and 360x400 ending captures put KEEP
  OPTIMISING and MAIN MENU side by side with identical treatment inside the
  report. Gameplay continuation and navigation share a single action group.
  The short-phone lifetime totals become a very small abbreviated line; current
  code uses 7px text and truncation there, and compresses the world ledger into
  another line. The detailed report loses readability to fit fixed geometry.
  This is not a claim that the two buttons are clipped in the inspected PNGs.
  **Change:** Keep the completion result and concise consumption summary central,
  with an emphasized continue-building action. Relocate Main Menu into a separate
  quiet navigation area. Put full resource names, totals and world Core/structure
  details in a readable scrollable report or visible disclosure; remove the tiny
  concatenated summaries instead of shrinking them further. Preserve all report
  data, the no-loss ending, and the existing save-on-menu behavior.
  **Acceptance:** Players immediately understand that the campaign is complete
  and building can continue. Full results are readable on the supported minimum,
  and the gameplay choice is visually distinct from leaving for the menu.
  **Verify:** 1280x720, 960x540, 360x640, 320x480 and the historical 360x400 stress
  case; 1.0x/1.5x text, long durations and large totals. Touch to expand/scroll
  results, continue building, and separately return to menu/load; confirm the
  finished campaign and world ledger are retained. Check both 599/600px layouts.

- [ ] **U13 — Reflow settings before text collides and make speed steppers directional.**
  **Depends on:** U1 supported minimum and U4 target/typography rules; coordinate
  action dispatch with the existing architecture task above.
  **Scope/files:** Display/Audio settings and return path;
  `src/screens/settings_menu.rs::settings_layout`, `settings_menu/display.rs`,
  `settings_menu/audio.rs`, `src/display.rs`, `src/main/update/phases.rs`.
  **Evidence/problem:** In the historical 240x400 Display capture the subtitle
  runs beneath Back and return guidance sits among the bottom controls.
  This establishes a stress-size limitation, not failure at a declared minimum.
  Current geometry keeps a minimum 240px panel plus margins and fixed rows as
  text scale increases. Independently, Display's Initial game speed treats
  either nonzero `stepper_row` result as increment: minus advances the speed
  exactly like plus. That direction defect is code-confirmed, not touch-tested.
  **Change:** Reserve Back/navigation and the tab row, then scroll/reflow settings
  within remaining space at the declared minimum and enlarged text. Remove the
  redundant selected-tab heading and permanent Back instruction; put essential
  setting explanations beside the appropriate control without truncating their
  meaning. Apply the stepper's signed direction to initial speed, with deliberate
  documented endpoint behavior; keep initial speed distinct from a world's live
  pause/speed. Preserve Audio Test signal, persistence and scale recovery.
  **Acceptance:** Players can read, change and reverse each setting, reach Back,
  and restore a usable text scale with touch alone. Minus means lower speed;
  plus means higher speed except at the explicitly chosen endpoint behavior.
  **Verify:** 1280x720, 960x540, 360x640, 320x480 and 240x400 stress case; both
  tabs at 0.75x, 1.0x and 1.5x text. Touch adjacent stepper controls, reverse a
  speed change, exercise endpoints, test audio, reopen settings after saving,
  and return to a campaign without changing its pause state unexpectedly.

### Further inspection required — not visually or interactively verified defects

- [ ] **U10 — Close the current-build/browser evidence gap before accepting the redesign.**
  **Depends on:** U1's declared viewport contract; execute relevant checks during
  U2–U9 and U11–U13, then close gaps here rather than duplicating their fixes.
  **Scope/files:** `src/capture_scenes.rs` and scene fixtures, `src/main.rs`
  capture entry, `game_page.json`, `src/display.rs`,
  `src/screens/settings_menu/`, `seed_ship_view/`, `campaign_complete_view/`,
  `records_view/`, planetary FLOW/DECK and `docs/verification/`.
  **Observed limitation:** Existing captures have differing tutorial versions;
  native PNGs do not prove CSS-sized browser readability, gesture routing or
  current scene parity. Only the selected captures listed above were inspected.
  The follow-up inspected yard, ending, Display settings, one mobile DECK and
  one mobile Records capture; U11–U13 record the specific new findings. This
  still does not verify live scroll/gesture behavior, completed-yard handoff,
  Audio layouts, full FLOW/DECK, dense Records or browser embedding. Settings
  reduced-motion behavior also needs a direct check: a stopped planetary pulse
  does not alone establish that research, yard and ending animation honors it.
  **Change:** Refresh supported representative scenes using the existing capture
  harness, replacing equivalent files. Run a fresh campaign and dense saved
  campaign in native and embedded browser views. Record actual canvas/CSS size,
  text scale, input mode and revision with results. Inspect title → bootstrap →
  factory → research → yard → launch → hostile arrival → recovery → ending,
  plus save/settings/Records return paths. Turn only reproduced residual issues
  into concrete tasks under this section, merging with existing tasks where
  appropriate.
  **Acceptance:** Every changed screen has current normal/minimum, dense,
  selected and failure evidence as applicable. No completion claim rests only
  on compilation, layout arithmetic or historical captures; limitations and
  publish results are reported explicitly.
  **Verify:** Full baseline and supported minimum matrix; touch without keyboard,
  hover or right-click for start, inspect, drag-route, zoom, dismissal, scrolling,
  build, research, ship diversion, launch confirmation, pause, recovery and
  continue-building ending choice. Check reduced motion and enlarged text;
  inspect whether any warning/action is lost when drawers or overlays coexist.

## Settings and display

- [x] Add a Fullscreen toggle to the Display tab and persist/apply the player's choice.
