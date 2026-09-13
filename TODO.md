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
