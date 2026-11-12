# Captures on the Hardware and Skin Override Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The spec's remaining hardware controls on the Archetype stage: a pedal's LCD shows its active capture and a
click steps to the next one; an amp shows a row of capture lights; and any block can override its automatic skin, saved
in the preset. Boards: ArchCapLcd, ArchCapLights, ArchSkinPick.

**Architecture:** Pedal bodies are re-rendered with a blank LCD and the LCD's rectangle is written to each `skin.json`;
the app paints the capture text into that rectangle. Capture stepping reuses the existing model-switch path
(`BlockDetail::select_model`). The skin override is a `ChainBlock.skin: Option<String>` that `skins::skin_for` honours;
it is visual only, so the audio fingerprint ignores it.

**Tech Stack:** Blender 5.3 + `tools/render` scripts, Rust/egui, `UiDriver` tests.

**Spec:** docs/superpowers/specs/2026-11-11-archetype-ui-design.md ("On pedals, an LCD showing the active capture;
clicking it cycles captures. On amps, capture lights do the same." and "a per-block override saved in the preset").
Boards on https://claude.ai/artifact/CaaEinMfx2MVib3hPSNz7x (markup:
/private/tmp/claude-501/-Users-vasilispoulopoulos-projects-ronnie-3000/dd2fa3ae-82be-49e2-9dc5-af365a8b9a68/scratchpad/canvas/project/ArchWindow.dc.html,
search `r.lcd`, `lightList`, `skinList`).

## Global Constraints

- No maker or model names drawn by the app or baked into renders (the existing render-script test enforces this).
- LCD look: dark glass is part of the render; the app draws text in Roboto Mono, colour #9effb3 with a soft glow
  (a 4 px blur of the same colour at ~70 % alpha, or two offset translucent passes), centred, uppercase, fitted to the
  LCD width (shrink to a minimum of 7 px, then truncate without an ellipsis). While a capture downloads it shows "···".
- LCD interaction on the selected pedal only: hover shows the hint bubble "CAPTURE i / n · <full capture name> · click for
  next" and a 2 pt #ff6a1a ring 2 pt outside the LCD; click selects the next capture (wrapping). Neighbour pedals show
  their LCD text but are not interactive (a click still selects the neighbour, as today). Pedals with one capture show
  their name and do nothing on click.
- Capture lights (amps with ≥ 2 captures): a pill centred under the amp render, 2 pt above the stage bottom: "CAPTURE"
  (9.5 pt semibold, tracking, #5f5f5c), 12 pt round lights 8 pt apart (off #3a3a38 with edge #1d1d1c; on #ffb347, edge
  #c87a1e, glow), then the active capture's short name in mono 10 pt. At most 8 lights; with more than 8 captures the 8
  lights are a window that keeps the active one inside, and "‹" / "›" buttons at the ends page by one. Clicking a light
  selects that capture. Accessible: a radio group "Captures" with one radio per light named by its capture.
- Skin picker: a chip bottom-left of the GEAR stage, "SKIN" + "Auto · <family name>" or "<family name>", opening a light
  list upward (230 pt wide): "Auto" (with the automatic family on the right), then every skin of the block's gear
  category by friendly name; the current choice highlighted. Friendly names: od-green "Green overdrive", od-gold "Gold
  overdrive", dist-black "Black distortion", fuzz "Fuzz", boost "Boost", comp "Compressor", default-dark "Dark pedal",
  default-light "Light pedal", highgain "High-gain head", modern-us "US modern head", british "British head", chime
  "Chime combo", clean "Clean combo", bass "Bass head", default-clean "Clean amp", default-crunch "Crunch amp",
  default-hi-gain "Hi-gain amp", cab-412-modern "Modern 4x12", cab-412-vintage "Vintage 4x12", cab-212-chime "2x12 cab",
  cab-112-clean "1x12 cab", cab-810-bass "8x10 bass cab", cab-default "Cab".
- The skin override never rebuilds or touches the audio chain; it is undoable (one step) and saved in presets/sessions as
  the JUCE property `skinOverride` (string, only when set); old presets load with Auto.
- The reference layout must not change.
- `make check` green.

## Review Focus

- Clicking the LCD rapidly while a capture downloads: one switch at a time (existing ticket generation), the display ends
  on the capture that actually landed.
- A skin override of a category that no longer matches (the block's tone was replaced by another category): the override
  is ignored (Auto) rather than drawing a pedal skin for an amp.
- Undo/redo of a skin change and of a capture step each take one step and do not rebuild audio for the skin change.
- A pedal whose skin has no LCD in its manifest (none today, but the field is optional): no display drawn, no crash.
- An amp with exactly 8 and with 9+ captures: the light window and paging behave; the window snaps to the active light on every capture change, and manual ‹ › paging may move it out of view until the next change.

---

### Task 1: Blank LCDs and their rectangles

**Files:** `tools/render/pedals.py`, `tools/render/common.py`, `tools/render/README.md`, `crates/ui/assets/skins/<8
pedals>/*`, `crates/ui/src/default/skins.rs`, `crates/ui/tests/skins.rs`.

**Produces:** pedal renders with the LCD glass but no LCD text; `skin.json` gains `"lcd": {"center": [x, y], "size": [w,
h]}` in body pixels (the glass rectangle, projected like the knobs); `SkinManifest.lcd: Option<SkinLcd { center: [f32;
2], size: [f32; 2] }>` parsed (optional) and `SkinLcd::rect_in(body: Rect, manifest) -> Rect` like `SkinKnob::rect_in`.

- [ ] Tests: `every_pedal_skin_has_an_lcd_inside_its_body` (the 8 pedal ids, rect within body bounds, non-zero);
  `amps_and_cabs_have_no_lcd`; `an_lcd_rect_scales_with_the_body`; existing skin tests stay green (including the shadow
  and haze tests — the re-render goes through the current `common.py`, which softens shadows).
- [ ] Implement: the `lcd(...)` helper registers the glass rectangle for the manifest and draws no text (keep the
  `cfg["lcd"]` strings out of the scene; delete them from the config or leave them unused); `finish` writes `"lcd"`;
  re-render the 8 pedals with `tools/render/build_skins.sh <id>` (default samples), check each body visually (Read the
  PNG), commit the assets with the code. Update the README's file list.
- [ ] Run, `make check`, commit "Render pedals with a blank LCD and record where it is".

### Task 2: The pedal LCD shows and steps the capture

**Files:** `crates/ui/src/default/block.rs` (`show_render`, `show_pedalboard`, `show_neighbour`), `skins.rs` (paint
helper). Test `crates/ui/tests/archetype_ui.rs`.

**Produces:** `skins::paint_lcd(painter, rect, text, ctx)`; the selected pedal's LCD is an interactive widget
"Capture display" (accessible name "Capture display: <full name>") whose click calls the existing capture-switch path
with the next model id (wrapping); `fn lcd_text(model_name) -> String` (uppercase).

- [ ] Tests: `a_pedal_lcd_shows_its_capture`; `clicking_the_lcd_steps_to_the_next_capture` (wraps at the end);
  `a_single_capture_pedal_does_nothing_on_click`; `a_neighbour_lcd_is_shown_but_not_interactive`;
  `the_lcd_shows_dots_while_the_capture_downloads`; `rapid_lcd_clicks_end_on_the_landed_capture`;
  `a_skin_without_an_lcd_draws_none` (a manifest with `lcd: None`).
- [ ] Implement per the Global Constraints, run, commit "Show and step a pedal's capture on its LCD".

### Task 3: Amp capture lights

**Files:** `block.rs` (amp stage), maybe a small `crates/ui/src/default/capture_lights.rs`. Test `archetype_ui.rs`.

- [ ] Tests: `an_amp_with_captures_shows_its_lights`; `clicking_a_light_selects_its_capture`;
  `a_single_capture_amp_has_no_lights`; `nine_captures_page_the_light_window` (active stays visible, "‹"/"›" page);
  `the_active_light_follows_a_capture_change_from_the_picker`; `cabs_and_pedals_have_no_lights`.
- [ ] Implement per the Global Constraints, run, commit "Show an amp's captures as lights".

### Task 4: Skin override

**Files:** `crates/core/src/chain.rs` (`ChainBlock.skin`, UI JSON + JUCE read), `crates/plugin/src/persistence.rs`
(`encode_block` writes/removes `skinOverride`), `crates/plugin/src/audio_session.rs` (`ChainFingerprint` ignores `skin`),
`crates/ui/src/backend.rs` (`UiAction::SetBlockSkin { block_id, skin: Option<String> }`), `crates/ui/src/session_edit.rs`
(one history step), `crates/ui/src/default/skins.rs` (`skin_for` honours a same-category override; `friendly_name(id)`;
`skins_in(category)`), `block.rs` (chip + list). Tests in core, plugin, ui.

- [ ] Tests: `a_skin_override_round_trips_through_juce` and `an_old_block_has_no_skin_override`;
  `changing_the_skin_does_not_rebuild_the_chain` (fingerprint unchanged); `the_skin_chip_lists_the_category_and_auto`;
  `picking_a_skin_redraws_the_render_and_is_one_undo_step`; `an_override_of_another_category_falls_back_to_auto`;
  `auto_shows_the_automatic_family_name`.
- [ ] Implement, `make check`, capture the three boards with the testbed (temporary edit, reverted), compare, commit
  "Let a block choose its skin".
