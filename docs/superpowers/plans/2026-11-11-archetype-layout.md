# Archetype Layout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the Archetype-style layout as a second view mode. It has a light header with the chain as gear icons and a
preset manager, a stage showing the selected block, a light global panel and a black utility bar. The current layout
stays the default and stays untouched.

**Architecture:**
- `shell::ViewMode::RonieFlow` already exists and is unused. It becomes the Archetype layout.
- `default::show` dispatches on `ChromeController::view_mode()`. The new layout lives in `crates/ui/src/default/archetype/`
  and reuses the existing state owners (`ChromeController`, `ChainGallery`, `BlockDetail`, `Faceplate`) by giving them
  different rects.
- Colours come from a runtime `Look`, stored in egui context data, so only code that opts in turns light.

**Tech Stack:** Rust 1.98, egui/eframe 0.37 (`Panel::top/bottom`, `CentralPanel`), the existing `UiDriver` for headless
tests.

**Spec:** `docs/superpowers/specs/2026-11-11-archetype-ui-design.md`

## Global Constraints

- **Design box:** 1024 design pixels wide and 578 tall (`shell::DESIGN_WIDTH`, `DESIGN_HEIGHT`), with whole-window zoom.
- **Archetype panel heights:** header 64, global panel 108, utility bar 36. The utility bar takes the hint strip's slot,
  so the window height logic is unchanged.
- **Archetype colours:**
  - Background `#f3f3f1`; panel `#fbfbfa`; edge/hairline `#d9d9d6`.
  - Text `#111111`; muted text `#5f5f5c`.
  - Utility bar `#0b0b0b` with text `#f2f2f2`; accent `#ff6a1a`.
  - Chain icon colours: selected `#111111`, on `#6b6b68`, bypassed `#b9b9b6`.
- **Accessibility labels are the test API, and must be kept:** "Previous preset", "Next preset", "Browse presets",
  "Save preset", "New preset", "Tuner", "Mono mode", "Stereo mode", "Undo", "Redo", "Add Tone", tone titles,
  "Back to main screen", and the faceplate control labels.
- **New labels:** "Add a block" (the + button) and "Settings" (utility bar).
- **No logos, brand scripts or artwork.** Chain icons come from `tile::block_glyph` / `draw_gear` (PR 2).
- **`ViewMode::ReferenceDefault` stays the default.** Every existing test must pass unchanged.
- **Selection:** `ronie --layout archetype` selects the new layout; `--layout reference` (or no flag) keeps the current one.

## Review Focus

1. **A long chain** (more blocks than fit in the header strip) must stay reachable: the strip scrolls with arrows, and
   icons never overlap the logo or the preset manager. Test in Task 3.
2. **Stereo (two lanes):** both lanes' blocks appear in the strip, lane A then lane B, with a divider. Test in Task 3.
3. **An empty chain (only the insert slot)** shows a stage prompt with an "Add a block" button, not a blank stage. Test
   in Task 4.
4. **A long preset name** is truncated with an ellipsis inside the preset pill and never moves the arrows. Test in Task 3.
5. **Settings, banner, tone browser and auth page** keep working in Archetype mode: settings still exits early, the
   banner still sits above the header, and the global panel hides while the browser or auth page is open, as today.
   Test in Task 2.

---

### Task 1: Runtime look (theme)

**Files:**
- Modify: `crates/ui/src/theme.rs` (add `Look`; leave `palette` and `Theme` alone)
- Test: `crates/ui/tests/look.rs`

**Interfaces:**
- Produces:
  - `pub struct Look { pub background, pub panel, pub edge, pub text, pub text_muted, pub utility, pub utility_text, pub accent, pub icon_selected, pub icon_on, pub icon_bypassed: Color32 }`
  - `Look::reference() -> Look`: dark values, matching today's black and palette colours.
  - `Look::archetype() -> Look`: the values in Global Constraints.
  - `Look::install(ctx: &egui::Context, look: Look)` and `Look::get(ctx: &egui::Context) -> Look`. Stored in
    `ctx.data_mut` under a fixed `egui::Id`; `get` falls back to `reference()`.

- [ ] **Step 1: Write the failing tests:**
  - `archetype_look_matches_the_spec`: `Look::archetype().background == Color32::from_rgb(0xf3,0xf3,0xf1)`, and likewise
    `panel`, `text`, `utility`, `accent`.
  - `get_falls_back_to_reference`: on a fresh `egui::Context`.
  - `install_then_get_round_trips`.
  - `archetype_text_is_readable`: WCAG contrast of `text` and `text_muted` on `background` and `panel` is at least 4.5,
    and of `utility_text` on `utility` at least 4.5 (a small contrast helper in the test file).
- [ ] **Step 2: Run** `cargo test -p ronie-ui --test look`. Expected: FAIL (`Look` not found).
- [ ] **Step 3: Implement `Look` in `theme.rs`.**
- [ ] **Step 4: Run** `cargo test -p ronie-ui --test look`. Expected: PASS.
- [ ] **Step 5: Commit** `git commit -m "Add a runtime look for light and dark layouts"`.

### Task 2: View-mode switch and the Archetype skeleton

**Files:**
- Modify: `crates/ui/src/default/header.rs`
  - add `view_mode: ViewMode` to `ChromeController`, with `pub fn view_mode(&self) -> ViewMode` and
    `pub fn set_view_mode(&mut self, mode: ViewMode)`;
  - `show` (`:1914`) dispatches to `super::archetype::show(ui, chrome)` after the settings early exit and the banner.
- Create: `crates/ui/src/default/archetype/mod.rs` with `pub fn show(ui, chrome)`. It installs `Look::archetype()`,
  fills the background, and draws empty header (64), utility bar (36), global panel (108, hidden while
  `chrome.browser` or auth is active) and central stage, in the same panel order as the reference `show`.
- Modify: `crates/ui/src/default/mod.rs` (`mod archetype;`).
- Modify: `crates/ui/src/shell.rs`: before `crate::default::show`, call `chrome.set_view_mode(self.preferences.view_mode)`.
- Modify: `crates/app/src/bin/ronie.rs`: parse `--layout archetype|reference` into `UiPreferences.view_mode`. An unknown
  value is a usage error.
- Test: `crates/ui/tests/archetype_ui.rs`; `crates/app/src/bin/ronie.rs` unit tests.

**Interfaces:**
- Consumes: `Look::install` (Task 1).
- Produces:
  - `ChromeController::view_mode/set_view_mode`.
  - `archetype::show`.
  - A test helper in `archetype_ui.rs`: `fn archetype_driver(scenario_id: &str) -> (UiDriver, Arc<MockBackend>)`, which
    mirrors `chain_ui.rs::main_driver` but calls `set_view_mode(ViewMode::RonieFlow)`.

- [ ] **Step 1: Write the failing tests:**
  - `reference_is_the_default_view_mode`.
  - `archetype_mode_paints_a_light_background`: after `settle`, the `Look` in the context equals `Look::archetype()`.
  - `settings_still_open_in_archetype_mode`: open settings, and the settings page shows.
  - `the_global_panel_hides_while_the_browser_is_open`.
  - In `ronie.rs`: `parse(&["--layout","archetype"])` gives `RonieFlow`; `parse(&["--layout","nope"])` is an error.
- [ ] **Step 2: Run** `cargo test -p ronie-ui --test archetype_ui` and `cargo test -p ronie-app --bin ronie`.
  Expected: FAIL.
- [ ] **Step 3: Implement the switch, the skeleton and the CLI flag.**
- [ ] **Step 4: Run** the same commands, plus `cargo test -p ronie-ui`. Expected: PASS, with every existing UI test
  unchanged.
- [ ] **Step 5: Commit** `git commit -m "Add an Archetype view mode behind --layout archetype"`.

### Task 3: Archetype header: logo, chain strip, preset manager

**Files:**
- Create: `crates/ui/src/default/archetype/header.rs` with `pub(super) fn show_header(ui, chrome, state: &ChainState)`.
- Modify: `crates/ui/src/default/chain.rs`: expose `pub(crate) fn activate_block(&mut self, id: &BlockId)` (the body
  of today's `activate` for one block) and `pub(crate) fn insert_slot(&self) -> Option<BlockId>` (the trailing insert
  of lane A).
- Test: `crates/ui/tests/archetype_ui.rs`.

**Interfaces:**
- Consumes:
  - `tile::block_glyph(&ChainBlock) -> GearGlyph` and `tile::draw_gear(painter, rect, glyph)` (PR 2).
  - `Look` (Task 1); `archetype_driver` (Task 2).
- Produces: none for later tasks beyond the header labels.

**Layout (design pixels):**
- Logo and wordmark at x 24, as one "Back to main screen" target, in `Look.text`.
- The strip is centred between x 260 and x 1024−380:
  - one 52×56 button per tone block, labelled with the tone title;
  - the glyph drawn at 38 px in `icon_selected`, `icon_on` or `icon_bypassed`;
  - a 2 px underline under the selected block;
  - 6 px gaps, then a 1 px divider, then a 40×40 dashed "+" labelled "Add a block".
- Preset manager in the right 360 px:
  - row 1: "Previous preset" / "Next preset" arrows, plus text buttons "SAVE AS…" (the existing save popover, labelled
    "Save preset") and "New preset";
  - row 2: a 34 px pill labelled "Browse presets", showing `* name` when dirty. The name is truncated to fit with an
    ellipsis.
- The reference header's "Tuner", "Mono mode"/"Stereo mode" and "Undo"/"Redo" move to the utility bar (Task 5).

- [ ] **Step 1: Write the failing tests:**
  - `strip_shows_one_button_per_block`: scenario with tone blocks; each title is visible.
  - `clicking_a_block_opens_it`: `chrome.detail` is open for that block.
  - `double_clicking_a_block_bypasses_it`: the block's `params.enabled` becomes false in the backend.
  - `add_a_block_opens_the_tone_browser`.
  - `a_long_chain_scrolls_instead_of_overlapping` (Review Focus 1): with 14 blocks, every block button's rect lies
    within x 260…644, and "Scroll chain right" is visible.
  - `stereo_lists_lane_a_then_lane_b` (Review Focus 2): in a stereo scenario, block titles are ordered A then B by x.
  - `a_long_preset_name_does_not_move_the_arrows` (Review Focus 4): the "Next preset" rect is equal for a short and a
    60-character name.
- [ ] **Step 2: Run** `cargo test -p ronie-ui --test archetype_ui`. Expected: FAIL.
- [ ] **Step 3: Implement `show_header`, `activate_block` and `insert_slot`.** Reuse `header_hit` (`header.rs:2352`)
  for accessibility labels and the existing preset popovers via `chrome` (as `show_header` does at `header.rs:2114`).
- [ ] **Step 4: Run** `cargo test -p ronie-ui`. Expected: PASS.
- [ ] **Step 5: Commit** `git commit -m "Archetype header with the chain as gear icons"`.

### Task 4: Stage: the selected block always shows

**Files:**
- Modify: `crates/ui/src/default/archetype/mod.rs` (central stage).
- Test: `crates/ui/tests/archetype_ui.rs`.

**Interfaces:**
- Consumes: `ChainGallery::activate_block`, `insert_slot` (Task 3).

**Behaviour:**
- The stage fills with `Look.background` and shows, in order of precedence:
  1. the tuner, when it's on;
  2. the browser, when it's open;
  3. the block detail, when one is selected (`chrome.gallery.show_meters` plus `detail.show`, as the reference does).
- When nothing is selected and the chain has a tone block, select the first tone block of lane A, so a block always
  shows.
- With no tone blocks, show a centred prompt "Your chain is empty" and an "Add a block" button that opens the browser.

- [ ] **Step 1: Write the failing tests:**
  - `the_first_block_opens_by_itself`.
  - `an_empty_chain_prompts_to_add_a_block` (Review Focus 3): the label is visible, and clicking it opens the browser.
  - `the_tuner_takes_the_stage`.
- [ ] **Step 2: Run** `cargo test -p ronie-ui --test archetype_ui`. Expected: FAIL.
- [ ] **Step 3: Implement the stage.**
- [ ] **Step 4: Run** `cargo test -p ronie-ui`. Expected: PASS.
- [ ] **Step 5: Commit** `git commit -m "Archetype stage always shows a block"`.

### Task 5: Light global panel and black utility bar

**Files:**
- Modify: `crates/ui/src/default/faceplate.rs`: `show` reads `Look::get(ui.ctx())` for its fill (`panel`), top hairline
  (`edge`), labels (`text`) and values (`text_muted`). Knob faces are unchanged.
- Create: `crates/ui/src/default/archetype/utility.rs` with `pub(super) fn show_utility(ui, chrome, cpu: Option<f32>)`.
- Test: `crates/ui/tests/archetype_ui.rs`, plus the existing `crates/ui/tests/faceplate.rs` (must pass unchanged).

**Interfaces:**
- Consumes: `Look`; the existing `ChromeController` actions used by the reference header for tuner, undo/redo and
  mono/stereo, and by the account menu for Settings.

**Utility bar:**
- 36 px tall, `Look.utility` fill.
- Left: "TUNER" (label "Tuner"), "MONO"/"STEREO" (labels "Mono mode"/"Stereo mode"), "UNDO"/"REDO" (labels
  "Undo"/"Redo"), "SETTINGS" (label "Settings").
- Right: "CPU x.x%" (from the same source as the hint strip) and the account name or "Sign in".
- Text is 11 px semibold, uppercase, `Look.utility_text`, with 12 px padding and 1×12 `#4a4a4a` separators.

- [ ] **Step 1: Write the failing tests:**
  - `faceplate_uses_the_light_panel_in_archetype_mode`: the faceplate's fill in the shape output is `Look::archetype().panel`.
  - `utility_tuner_toggles_the_tuner`.
  - `utility_settings_opens_settings`.
  - `utility_mono_stereo_switch_modes`.
  - `utility_undo_redo_follow_history`.
- [ ] **Step 2: Run** `cargo test -p ronie-ui --test archetype_ui`. Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `cargo test -p ronie-ui`, including `--test faceplate`. Expected: PASS.
- [ ] **Step 5: Commit** `git commit -m "Light global panel and black utility bar"`.

### Task 6: Verify in the running app and document

**Files:**
- Modify: `README.md` (one line under First steps: `ronie --layout archetype`).
- Modify: `CHANGELOG.md` (an Unreleased entry).

- [ ] **Step 1: Run** `make check`. Expected: exit 0.
- [ ] **Step 2: Run** `cargo run --release -p ronie-app --bin ronie -- --dummy --layout archetype`, and capture the window
  with `screencapture -l<window id>` (or `-x` for the screen). Check the header, stage, panel and utility bar against
  the canvas, and save the image to the scratchpad for the PR description.
- [ ] **Step 3: Commit** `git commit -m "Document the Archetype layout"`.

---

## Phase 2: final review fixes and alignment with the approved designs

Designs (canvas, 1024×614): `ArchGearAmp`, `ArchGearPedals`, `ArchGearCab`, `ArchEq`, `ArchInfo`, `ArchEmpty`,
`ArchStereo`, `ArchStrip`. The user approved them on 2026-11-11. The rendered gear on the GEAR tab arrives in PR 4.

### Task 7: Review fixes (no new UI)
- **Banner:** the background fill starts below any banner, by filling `ui.available_rect_before_wrap()`, not `max_rect`.
  Test `the_background_leaves_the_banner_visible`: on a banner scenario, no light background rect is painted over
  y < 44.
- **No dark gallery flash:** in Archetype mode `show_stage` never falls through to `gallery.show`. When the stage is
  free after the syncs, it opens a block in the same frame. Test `removing_a_block_opens_its_neighbour_without_a_flash`:
  one `frame()` after "Remove block", "Add Tone" (a gallery tile) is not visible, and the next block is selected.
- **Focus after a change** (design `ArchStrip`):
  - after add or swap, the loaded block opens;
  - after remove, the neighbour opens (the next block, else the previous one).

  Remember the last focus in egui temp memory.
- **Popover triggers:** the preset pill, SAVE AS… and Account set `overlay_trigger`, so a second click closes the
  popover. Test `a_second_click_on_the_preset_pill_closes_the_browser`.
- **Dark surfaces stay dark:** decks draw under `Look::reference()`, and the input-mode chip glyph stays white. Test
  `deck_captions_stay_light_on_the_dark_deck`.

### Task 8: Align with the designs
- **Utility bar** (`ArchEmpty`/`ArchGearAmp` footer):
  - left: TUNER | MIDI | OPTIMIZE | TONE3000 | SETTINGS;
  - right: CPU x.x% | the account name, or SIGN IN.

  MIDI opens settings. OPTIMIZE toggles `tuning::set_optimize` and is green when on. TONE3000 opens the tone browser
  for the selected lane. The account item opens the account menu with its trigger.
- **Preset manager:** Undo and Redo icons sit after the preset arrows.
- **Lanes:** a Mono/Stereo switch ("Mono mode"/"Stereo mode") in the global panel after Input, in Archetype mode only.
- **Block page:** no "Back to chain" link in Archetype mode.
- **Stereo +:** adds to the lane of the selected block. The lane choice inside the black + menu comes with that menu
  (a later PR).
