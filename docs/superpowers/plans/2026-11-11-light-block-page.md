# Light Block Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** In the Archetype layout, the selected block opens on a light page that matches the approved ArchWindow
boards: a title row with BYPASS / REPLACE / REMOVE, GEAR / EQ / INFO tabs, a 300-point render stage where consecutive
pedals share the stage, and light EQ and INFO pages.

**Architecture:** `BlockDetail` gains a page style (`Reference` keeps today's dark card, `Archetype` draws the light
page). Archetype mode sets it next to `set_rendered`. The pedal run around the selected block is computed in
`archetype/mod.rs` from the chain state and handed to the detail, which draws the neighbours read-only and reports a
click on one as a selection request. The EQ editor, the info panel and the model picker read `Look::get(ctx)` and swap
their dark colours for light ones when the archetype look is installed.

**Tech Stack:** Rust, egui/eframe, the `UiDriver` test harness (`crates/ui/tests/archetype_ui.rs`).

**Spec:** docs/superpowers/specs/2026-11-11-archetype-ui-design.md; design boards ArchWindow, ArchGearAmp, ArchGearPedals,
ArchGearCab, ArchEq, ArchInfo on the canvas https://claude.ai/artifact/CaaEinMfx2MVib3hPSNz7x.

## Global Constraints

- The reference layout must not change (its tests stay green, no capture diff).
- No maker names or logos are drawn by the app; titles show the capture's own name, as in the design.
- Colours come from `Look::archetype()`: background #f3f3f1, panel #fbfbfa, edge #d9d9d6, text #111111, muted
  #5f5f5c, accent #ff6a1a; tab inactive #6b6b68.
- `make check` green.

## Review Focus

- A local `.nam` (no TONE3000 page): INFO tab still opens and shows the file's metadata, no sign-in prompt for nothing.
- A pedal run where one neighbour has no skin (unknown family): it is not drawn and the run breaks there.
- Clicking a neighbour pedal while dragging the selected pedal's knob: the drag gesture closes.
- Bypassed block: BYPASS reads as active and the render dims.
- Switching tabs mid EQ drag closes the EQ gesture (existing `close_hidden_drags`).

---

### Task 1: The light page frame

**Files:** Modify `crates/ui/src/default/block.rs`, `crates/ui/src/default/archetype/mod.rs`. Test
`crates/ui/tests/archetype_ui.rs`.

**Produces:** `BlockDetail::set_page(BlockPage)` with `pub enum BlockPage { Reference, Archetype }` (re-exported
next to `BlockView`); `BlockDetail::view() -> BlockView`.

- [ ] Tests: `the_block_page_has_a_title_row_and_tabs` (main-mono, amp selected: visible "GEAR", "EQ", "INFO" tabs,
  "BYPASS", "REPLACE", "REMOVE", and text "AMP · '02 VOX AC30/6 TOP BOOST"); `the_eq_tab_opens_the_eq` (click "EQ" →
  `detail.view() == BlockView::Eq`); `bypass_toggles_the_block` (click "BYPASS" → block params.enabled false);
  `remove_removes_the_block`; `replace_opens_the_browser_for_the_block`.
- [ ] Implement: `show_detail` branches on page. Archetype page: no card; inset 24/12/24/14 like the design; title row
  36 pt (category word PEDAL/AMP/CAB/EQ… · uppercased title, 10 pt semibold, tracking; meta line 11 pt muted:
  "NAM capture from TONE3000 · 6 captures in this tone", "IR from TONE3000 · 24 mic positions", local: "Local .nam file");
  right text actions 10 pt; tabs row 30 pt with 2 pt underline on the active tab, hairline below. INFO tab only for
  non-local tones (as today). Body below goes to the per-tab views.
- [ ] Run the tests, `make check`, commit.

### Task 2: The GEAR stage

**Files:** `block.rs`, `model_select.rs`. Test `archetype_ui.rs`.

- [ ] Tests: `the_amp_render_fills_the_designed_stage_height` (the "Amp render" rect is ≥ 240 pt tall at the
  1024×614 viewport, ≤ 300); existing render knob tests stay green; `a_bypassed_block_dims_its_render` is covered by
  reading `BYPASS` state (selected = enabled false → BYPASS button `selected`).
- [ ] Implement: on the archetype page the GEAR body is the stage: meters slimmer at the sides, render fitted to
  `RENDER_HEIGHT` with the space left after a 36 pt picker row, knob hint bubble, the gear note bottom-right (muted
  10 pt, design copy). Model picker drawn light when `Look::get` is archetype (fill #e5e5e2, border #cfcfcc, text
  #111), 320 pt wide, centred under the render. No skin → the existing artwork + trims view on the light page, with
  light text.
- [ ] Run, commit.

### Task 3: Pedals share the stage

**Files:** `block.rs`, `archetype/mod.rs`. Test `archetype_ui.rs` (scenario main-mono with a second pedal inserted
after blk-1 in the test).

**Produces:** `BlockDetail::set_stage_neighbours(before: Vec<ChainBlock>, after: Vec<ChainBlock>)`,
`BlockDetail::take_select_request() -> Option<BlockId>`; `pub(super) fn pedal_run(state, id) -> (Vec<ChainBlock>,
Vec<ChainBlock>)` in archetype/mod.rs: the contiguous tone blocks in the same lane around `id` whose skin is a pedal,
only when the selected block is itself a pedal.

- [ ] Tests: `consecutive_pedals_share_the_stage` (two "Pedal render" images visible; title "PEDALS · A → B");
  `clicking_a_neighbour_pedal_selects_it` (gallery selection becomes that block); `an_amp_shares_the_stage_with_no_one`.
- [ ] Implement: renders laid out left to right with 6 pt gap, all at the same height, scaled down together to fit;
  the selected one has live knobs; neighbours draw at their params' knob positions, accessible label "Pedal render:
  <title>", click → select request. Title uses the run's titles joined with " → ".
- [ ] Run, commit.

### Task 4: Light EQ and INFO

**Files:** `eq.rs`, `block_info.rs`, `block.rs`. Test `archetype_ui.rs`.

- [ ] Tests: `the_eq_page_has_band_chips_and_pre_post` (EQ tab: "Band 1".."Band 6" select buttons, "EQ power",
  "PRE" toggle visible); `the_info_page_shows_the_tone_title_and_page_link` (INFO tab: text "'02 Vox AC30/6 Top
  Boost", "Open on TONE3000" button).
- [ ] Implement: on the archetype page the EQ body is a rounded #fbfbfa card holding the curve view (light grid
  #e5e5e2, curve #111, spectrum accent at 16 %, selected handle accent), and a footer row: EQ power, band chips 1–6
  (selected filled accent), the band readout in mono ("BELL · 220 Hz · −2.5 dB · Q 1.20"), PRE/POST switch on the
  right. EqEditor and BlockInfoPanel read `Look::get(ctx)` for their text, grid and fill colours. INFO: 200 pt
  artwork, title 18 pt bold, by-line, metadata, the info panel's description and links, "Open on TONE3000".
- [ ] Run, `make check`, capture the four boards with the testbed, compare against the design, commit.
