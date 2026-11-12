# Replace, Remove and Smart Placement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix three pieces of user feedback on the Archetype layout: (1) there is no easy way to replace a block (e.g. a
head with another); (2) removing a block is hard to find; (3) the "+ PEDAL before the amp" slot is redundant if + places
gear sensibly. Boards: ArchReplace, ArchStripMenu on https://claude.ai/artifact/CaaEinMfx2MVib3hPSNz7x (markup:
/private/tmp/claude-501/-Users-vasilispoulopoulos-projects-ronnie-3000/dd2fa3ae-82be-49e2-9dc5-af365a8b9a68/scratchpad/canvas/project/ArchWindow.dc.html,
search `fromReplace`, `newActs`, `stripMenu`).

**Architecture:** The add menu gains a replace mode targeting one block; a landed tone keeps the replaced block's
settings (restored without an extra history step, like `PlaceAddedBlock`). The block page's text actions become pill
buttons; the chain strip gets a right-click menu; Delete/Backspace removes the selected block. Adding with + (no slot)
places the block by gear category instead of at the end of the lane, and the amp-side stage slots are removed.

**Tech Stack:** Rust, egui/eframe, `UiDriver` tests (`crates/ui/tests/archetype_ui.rs`).

**Spec:** docs/superpowers/specs/2026-11-11-archetype-ui-design.md; user feedback 2026-11-12: "no ez way to replace an
element, for example a head with another", "I dont see how i can remove an element … bad ux", "why can i add a pedal to
the left of a head? I can add a pedal any way".

## Global Constraints

- Replace mode: REPLACE (page button or strip menu) opens the black add menu straight on the block's gear category, under
  the REPLACE button (caret pointing at it), header "‹ REPLACE · <CATEGORY>" and right text "keeps trims, EQ and place";
  no lane picker. Rows read "SWAP" (the current tone reads "CURRENT", not clickable); "DOWNLOADING" as today. Back goes
  to the categories (still replacing the same block). OPEN THE TONE BROWSER / SHOW ALL / LOAD A FILE… also replace that
  block. While the menu is open the REPLACE button is drawn solid #111 with white content.
- A replace keeps the block's id, position, `params` (enabled/bypass, gains, mix, normalize, slim size, EQ, IR blend
  fields) and drops `ir_b` and the `skin` override (they belong to the old tone). One undo restores the old tone and
  params exactly. A failed/cancelled replace leaves the block untouched.
- Page actions: three pill buttons right-aligned in the title row, 28 pt tall, radius 14, 1 px border, 10 pt semibold
  tracked labels with 12 pt icons: BYPASS (power icon; selected state = solid #111 with white content when the block is
  bypassed), REPLACE (swap-arrows icon), REMOVE (trash icon, text and icon #b3261e, border #e4b4ad). Accessible names
  "Bypass" (toggle with selected state), "Replace", "Remove".
- Strip right-click: a dark menu (fill #0b0b0b, 200 pt wide, radius 8, caret to the icon) under the right-clicked icon
  with a muted title (the block's title, uppercased, truncated with "…"), rows BYPASS (with "double-click" hint in mono
  muted), REPLACE…, DUPLICATE, a hairline, REMOVE (#ff8a80, with "⌫" hint). Escape or a click outside closes it.
  Double-click on an icon still toggles bypass. Right-click no longer bypasses directly.
- Delete or Backspace removes the selected block when no text field has focus and no menu is open; one undo restores it.
- Smart placement for + (no slot, Placement::End today): Pedal → before the first amp, full rig or cab in the lane (else
  end); Amp and Amp+Cab → before the first cab (else end); Cab → after the last amp/full rig (else end); Outboard and
  Space → end. The add menu's right header text says where it goes ("before '02 Vox AC30/6 To…", "after …", or "to the
  end of the chain"), titles shortened to 18 characters with "…" like the slot menu.
- Stage slots: remove "+ PEDAL" before an amp and "+ CAB" after an amp; keep "+ PEDAL" after a pedal run.
- The reference layout must not change.
- `make check` green.

## Review Focus

- Replacing a block that is downloading a capture, or whose B IR is downloading: the replace wins, no stale bytes land.
- Replace with a tone of another category (via Back → another category): allowed; the block keeps params; the skin
  override is dropped.
- Delete while typing in the add-menu search field or the preset name: nothing is removed.
- Right-click on an insert/empty slot or the + button: no block menu.
- Smart placement in stereo lane B and in a lane with no amp/cab.

---

### Task 1: Replace in place

**Files:** `crates/ui/src/default/archetype/add_menu.rs`, `archetype/mod.rs`, `crates/ui/src/default/header.rs`
(`ChromeController`), `crates/ui/src/default/block.rs` (REPLACE request), `crates/ui/src/session_edit.rs` +
`crates/ui/src/backend.rs` (a no-history `RestoreReplacedParams`-style action, or a `keep_params` flag on the load path —
choose the smaller change; one undo must restore everything). Tests `archetype_ui.rs`, session_edit inline.

**Produces:** `AddMenu::open_replace(block_id, category, anchor)`; a menu mode that turns Add/browser/file into a replace
of that block; `ChromeController::replace_tone(block_id, tone)` (and the browser/file equivalents) that keep params.

- [ ] Tests: `replace_opens_the_menu_on_the_blocks_category`; `swapping_a_head_keeps_its_place_and_settings` (gain/EQ/
  bypass kept, same position and id); `one_undo_restores_the_replaced_block`; `the_current_tone_reads_current`;
  `replace_via_the_browser_and_a_file_keeps_settings`; `a_replace_drops_the_skin_override_and_ir_b`;
  `a_replace_wins_over_a_capture_download_of_that_block`.
- [ ] Implement, run, commit "Replace a block in place from the add menu".

### Task 2: Visible actions, strip menu, Delete key

**Files:** `block.rs` (title row), `archetype/header.rs` (strip right-click menu), `archetype/mod.rs` (Delete key).
Tests `archetype_ui.rs` (update `right_clicking_a_block_bypasses_it` → `right_clicking_a_block_opens_its_menu`).

- [ ] Tests: `the_page_shows_bypass_replace_and_remove_buttons` (selected state on bypass);
  `right_clicking_a_block_opens_its_menu` (rows; Escape closes); `the_strip_menu_bypasses_replaces_duplicates_and_removes`;
  `double_click_still_bypasses`; `delete_removes_the_selected_block_and_undo_brings_it_back`;
  `delete_while_typing_removes_nothing`; `right_click_on_plus_or_an_empty_slot_opens_no_menu`.
- [ ] Implement per the Global Constraints, run, commit "Make block actions easy to find".

### Task 3: Smart placement and fewer slots

**Files:** `crates/ui/src/default/header.rs` (`add_tone` default placement), `archetype/mod.rs` (`stage_slots`),
`add_menu.rs` (header text), `block.rs` (slot drawing). Tests `archetype_ui.rs`.

- [ ] Tests: `a_pedal_from_plus_lands_before_the_amp`; `an_amp_from_plus_lands_before_the_cab`;
  `a_cab_from_plus_lands_after_the_amp`; `outboard_lands_at_the_end`; `placement_works_in_lane_b`;
  `a_lane_without_gear_adds_at_the_end`; `the_menu_says_where_it_will_add`; `an_amp_page_has_no_side_slots`;
  `a_pedal_run_still_ends_with_a_slot`. Update or remove the obsolete amp-slot tests.
- [ ] Implement, `make check`, capture the replace menu, the page buttons and the strip menu with the testbed
  (temporary edits, reverted), compare with the boards, commit "Place added gear by type and drop the amp-side slots".
