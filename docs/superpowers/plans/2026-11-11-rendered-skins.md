# Rendered Gear Skins Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** The Archetype block page shows a Blender render of the block's gear, and its GAIN IN, OUTPUT and MIX knobs turn the block's input gain, output gain and mix.

**Architecture:** Blender scripts under `tools/render/` render one transparent body per skin, plus a 32-frame filmstrip for each live knob and a JSON manifest that gives the knob positions. A build script turns the renders into WebP under `crates/ui/assets/skins/<skin>/`. `crates/ui/src/default/skins.rs` embeds the assets, decodes them once per context, and draws a skin with live knobs through the existing `widgets::knob::show` gestures. `BlockDetail` draws the render in its tone view when the Archetype layout asks for it.

**Tech Stack:** Blender 5.3 (Cycles, Metal), Python in Blender, cwebp/ImageMagick, Rust (egui, image 0.25 with webp, serde_json).

**Spec:** `docs/superpowers/specs/2026-11-11-archetype-ui-design.md` ("Rendering (PR 4)"); designs on the canvas: ArchGearAmp, ArchGearPedals, ArchGearCab, ArchWindow.

## Global Constraints

- No maker logos, brand scripts, artwork or signature trims in any render.
- Pedals: camera about 10° from straight down. Amps and cabs: front view about 9° above. One shared studio rig.
- Output: a transparent body image per skin, plus knob filmstrips. egui picks the filmstrip frame for each control's value.
- The render fills the stage at up to 300 design points tall (ArchWindow `r.h = 300`), with a soft drop shadow drawn by the app.
- A knob's value shows while you drag it, as "GAIN IN  +3.5 dB" (ArchWindow hint).
- The reference layout does not change.

## Review Focus

1. A skin with no asset (a family added later): the block page must fall back to the current artwork view, not panic.
2. Alpha at the render edges: no faint grey rectangle (the known render flaw). Pinned by a test on every body's border alpha.
3. HiDPI: the body must stay sharp at 2× on a 300-point stage, so bodies are at least 600 px tall or 1000 px wide.
4. Knob filmstrip frames that do not line up with the body leave a visible square: frames are feathered to a circle.
5. A drag on a render knob must open and close one backend gesture, like the trim knobs.

---

### Task 1: Render tooling

**Files:** Create `tools/render/{rig.py,pedals.py,gear.py,build_skins.sh,README.md}` (from the scratchpad prototypes).

- Knob builders return a handle that can set the knob's value (pointer objects move and rotate about the knob axis).
- After the body render: alpha below 4/255 becomes 0, the rest is remapped, and alpha fades to 0 within 12 px of the frame edge.
- For each live knob (`gain`, `output`, `mix`): 32 frames at values `i/31`, rendered with a crop border around the knob (side 2.5 × the knob radius in pixels), feathered to a circle.
- `skin.json`: `{ "id", "width", "height", "knobs": [{ "control": "gain"|"output"|"mix", "label": "GAIN IN"|"OUTPUT"|"MIX", "center": [x, y], "side": px, "frames": 32 }] }`, in body pixels.
- `build_skins.sh [skin...]` renders, scales the body to 1200 px on its long side, and writes `body.webp`, `knob-<control>.webp` (frames side by side) and `skin.json`.

Verify: render `od-green` and `british`; border alpha is 0; the manifest's knob centres sit on the knobs in the body.

### Task 2: Assets for every skin

Render all 23 skins (8 pedals, 9 amps, 6 cabs) into `crates/ui/assets/skins/`. Verify: total size under 6 MB.

### Task 3: `skins` module

**Files:** Create `crates/ui/src/default/skins.rs`; test `crates/ui/tests/skins.rs`.

- `pub fn skin_for(block: &ChainBlock) -> Option<&'static str>`: the family's `skin_id()` when an asset exists.
- `pub fn manifest(id: &str) -> Option<SkinManifest>`; `SkinManifest { id, width, height, knobs: Vec<SkinKnob> }`; `SkinKnob { control: KnobControl, label, center: [f32; 2], side: f32, frames: u32 }`; `KnobControl::{Gain, Output, Mix}`.
- `pub fn body_image(id) -> Option<egui::ColorImage>`, `knob_strip(id, control)`.
- Tests: every `GearFamily::skin_id()` in pedal, amp and cab categories has a manifest and a body; every body has zero alpha on its 2 px border; knob centres are inside the body; pedals and amps have the three knobs, cabs none.

### Task 4: Render on the Archetype block page

**Files:** Modify `crates/ui/src/default/block.rs` (`set_rendered(bool)`, `show_tone`), `crates/ui/src/default/archetype/mod.rs`; tests in `crates/ui/tests/archetype_ui.rs`.

- With `rendered` and a skin: the centre of the tone view shows the render (fit, at most 300 points tall) over a soft shadow; trims In, Mix and Out give way to the render's knobs, labelled "GAIN IN", "OUTPUT", "MIX"; the model picker stays.
- Each render knob uses `widgets::knob::show` and the trim gesture tracking; its value shows above it while hovered or dragged.
- Tests: the Archetype amp page exposes "GAIN IN", "OUTPUT" and "MIX" and no "In"; dragging "GAIN IN" up raises `input_gain`; the reference layout still shows "In"; a block with no skin still shows its artwork.

### Task 5: Docs and release

README and CHANGELOG; answer the spec's storage question (WebP in git, embedded). `make check`; capture the app; PR to the fork and merge.
