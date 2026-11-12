# Add From the Stage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On the Archetype GEAR stage, dashed "+" slots sit where gear goes: "+ PEDAL" after a pedal run, "+ PEDAL"
before an amp and "+ CAB" after an amp with no cab after it. A slot opens the add menu on that category, next to the
slot, and the tone lands at that position in the lane, as on the boards ArchSlotPedals, ArchSlotAmp and ArchSlotMenu.

**Architecture:** The block page draws the slots beside the renders and reports a click as a slot request (category and
position). The archetype layout opens the existing `AddMenu` straight on `Results(category)` with a placement and an
anchor rect; the menu draws beside the anchor with its caret pointing at it. `ChromeController::add_tone` takes an
optional placement; when the tone lands, a new `UiAction::PlaceAddedBlock` moves the block to its position **without a
history step**, so one undo removes the add.

**Tech Stack:** Rust, egui/eframe, `UiDriver` tests (`crates/ui/tests/archetype_ui.rs`).

**Spec:** docs/superpowers/specs/2026-11-11-archetype-ui-design.md; boards ArchSlotPedals, ArchSlotAmp, ArchSlotMenu on
https://claude.ai/artifact/CaaEinMfx2MVib3hPSNz7x (markup:
/private/tmp/claude-501/-Users-vasilispoulopoulos-projects-ronnie-3000/dd2fa3ae-82be-49e2-9dc5-af365a8b9a68/scratchpad/canvas/project/ArchWindow.dc.html,
search `isSlot`, `fromSlot`, `whereText`). Builds on the add menu of docs/superpowers/plans/2026-11-11-add-menu.md.

## Global Constraints

- The reference layout must not change.
- Slot look: rounded 10, 1.5 pt dashed border #a9a9a6 on a 35 % white fill, plus icon and label 10 pt semibold with
  tracking in #5f5f5c; while its menu is open the slot is solid #111 with white content. Sizes: pedal-run slot 110×176,
  amp-side slots 96×160, vertically centred on the renders, 18 pt from the neighbouring render.
- Labels and accessible names: "PEDAL" / "Add a pedal after these" (after a run), "PEDAL" / "Add a pedal before the amp",
  "CAB" / "Add a cab after the amp".
- "+ CAB" shows only when the next tone block after the amp in its lane is not a cab. The pedal-run slot shows on the
  pedals stage only.
- A slot-opened menu: header back button with the category, right text "after <TITLE OF THE LAST PEDAL>" / "before
  <AMP TITLE>" / "after <AMP TITLE>" (titles shortened to 18 characters with "…"), no lane picker (the lane is the
  slot's), anchored beside the slot (left of it when it does not fit on the right) with the caret on that side. The
  + button in the header stays dashed.
- One undo removes a tone added from a slot (no extra history step for the placement).
- `make check` green.

## Review Focus

- A slot add while the chain changes underneath (the run's last pedal is removed before the download lands): the block
  is placed after the nearest remaining block of the run, or at the end of the lane.
- Stereo: slots act on the lane of the selected block, including lane B.
- Undo right after a slot add removes the new block in one step and leaves the run as it was.
- A narrow window where the slot does not fit beside the renders: renders shrink together (as the pedalboard already
  scales) and the slot stays visible and clickable.
- Opening a slot's menu while a + menu download is running: the running download is replaced (LoadFlow gate), and no
  stale DOWNLOADING row remains.

---

### Task 1: Placement in the chain

**Files:** `crates/ui/src/backend.rs` (`UiAction`), `crates/ui/src/session_edit.rs`, `crates/ui/src/default/header.rs`
(`ChromeController::add_tone`, `open_added_block`). Tests: `session_edit.rs` unit tests, `archetype_ui.rs`.

**Interfaces — Produces:**
- `UiAction::PlaceAddedBlock { block_id: BlockId, side: ChainSide, index: usize }`: moves the block to `index` in
  `side`'s lane (clamped before the trailing insert) without pushing history. Refuses insert blocks.
- `pub enum Placement { End, After(BlockId), Before(BlockId) }` (in `header.rs` or a small module next to it, `pub`
  for the archetype layout and tests).
- `ChromeController::add_tone(side, insert_id, tone, placement: Placement)` (existing callers pass `Placement::End`).
  When the added block lands (where `open_added_block` runs) and the placement is not `End`, it resolves the index from
  the current chain (After(id) → index of id + 1; Before(id) → index of id; a missing anchor → End) and dispatches
  `PlaceAddedBlock` before opening the block.

- [ ] Tests (unit): `placing_an_added_block_adds_no_history_step` (load into the insert, place it at 0, one undo →
  chain equals the one before the load); `placing_clamps_before_the_insert`; `placing_an_insert_is_refused`.
- [ ] Tests (archetype_ui): `a_tone_added_after_a_block_lands_after_it` (main-mono with a cloned pedal as in
  `pedalboard_driver`; `add_tone(.., Placement::After(first pedal))` → the new block sits right after it);
  `a_vanished_anchor_falls_back_to_the_end`.
- [ ] Implement, run, `make check`, commit "Place an added block next to the gear it was added from".

### Task 2: Slots on the stage

**Files:** `crates/ui/src/default/block.rs` (`show_gear_stage`, `show_pedalboard`), `archetype/mod.rs`. Test
`archetype_ui.rs`.

**Interfaces — Produces:**
- `pub struct SlotRequest { pub category: GearCategory, pub placement: Placement, pub anchor: Rect }`;
  `BlockDetail::set_stage_slots(slots: StageSlots)` where `StageSlots { pedal_after: Option<BlockId>,
  pedal_before: Option<BlockId>, cab_after: Option<BlockId> }` is computed in `archetype/mod.rs` from the chain (next
  to `pedal_run`); `BlockDetail::take_slot_request() -> Option<SlotRequest>`; `BlockDetail::set_open_slot(Option<
  GearCategory>)` draws that slot solid.

- [ ] Tests: `a_pedal_run_ends_with_a_pedal_slot` (pedalboard driver: a button "Add a pedal after these" right of the
  last "Pedal render"); `an_amp_has_a_pedal_slot_before_it` ("Add a pedal before the amp" left of "Amp render");
  `an_amp_followed_by_a_cab_has_no_cab_slot` and `an_amp_with_no_cab_after_it_has_a_cab_slot`; `a_cab_page_has_no_slots`;
  `slots_stay_visible_in_a_narrow_window` (viewport 800 wide: the slot rect is inside the stage and clickable).
- [ ] Implement per the Global Constraints. The renders and slots scale down together when they do not fit.
- [ ] Run, commit "Show slots on the stage where gear can go".

### Task 3: A slot opens the menu where it points

**Files:** `archetype/add_menu.rs`, `archetype/mod.rs`. Test `archetype_ui.rs`.

**Interfaces — Consumes:** Task 1 `Placement`, `add_tone(.., placement)`; Task 2 `take_slot_request`, `set_open_slot`.
**Produces:** `AddMenu::open_at(category: GearCategory, placement: Placement, anchor: Rect, lane: ChainSide)`; the menu
keeps `placement` and `anchor: Option<Rect>`; `AddMenuAction::Add` carries through to `add_tone` with the placement.

- [ ] Tests: `the_pedal_slot_opens_the_menu_on_pedals` (click "Add a pedal after these": "Back to categories" and
  "PEDAL" header, text "after <last pedal title>", no "LANE A"/"LANE B" radios even in stereo, the menu rect sits beside
  the slot rect and does not cover it, and the header + button is not drawn solid); `adding_from_the_pedal_slot_puts_the_pedal_after_the_run`
  (library pedal → lands after the run, opens on the stage); `adding_from_the_amp_slot_puts_the_pedal_before_the_amp`;
  `the_cab_slot_adds_a_cab_after_the_amp`; `one_undo_removes_a_slot_add`; `back_from_a_slot_menu_shows_categories_and_keeps_the_placement`
  (a category picked after Back still lands at the slot's position); `a_slot_add_in_lane_b_stays_in_lane_b` (stereo).
- [ ] Implement: when `anchor` is set, place the 380-wide menu to the right of the anchor with 10 pt gap if it fits in
  the window, else to its left; vertical centre on the anchor, clamped inside the stage; caret on the side facing the
  anchor. Header right text per the Global Constraints. While open, `set_open_slot(Some(category))`.
- [ ] Run, `make check`, capture the three states with the testbed (temporary code edit, reverted before commit, as in
  the add-menu Task 4), compare with ArchSlotPedals / ArchSlotAmp / ArchSlotMenu, fix differences, commit "Open the add
  menu from a stage slot".
