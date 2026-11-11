# Archetype-style interface with rendered gear

Design canvas: https://claude.ai/artifact/CaaEinMfx2MVib3hPSNz7x (private; share it before linking it anywhere).

## Intent

Make Ronnie look and work like the Neural DSP Archetype plugins (Archetype: Nolly X is the reference), adapted to
Ronnie's free chain. Each block shows the gear its capture models, as a photoreal render, so a 5150 capture always looks
like a high-gain head and a TS808 capture always looks like a green treadle overdrive.

## Goals

- **Archetype layout.**
  - Light chrome, IBM Plex Sans.
  - Preset manager top right: `← | →  DELETE  SAVE  SAVE AS…  ⋮` over a dropdown, with `*` when there are unsaved changes.
  - A white global panel: Input, Gate (toggle), Pitch with tuning, Input mode, Doubler/Spread, Bass/Middle/Treble,
    Output, and edge meters.
  - A black utility bar: Tuner, MIDI, Optimize with latency, TONE3000, Settings, CPU, version, window size.
- **The chain is the section strip.** One outline icon per block, with **+** at the end opening a black Archetype-style
  menu (categories, then library and TONE3000 results, then load from a file). Click a block to open it; double-click
  to bypass it, which greys its icon.
- **One skin per gear family**, chosen automatically (see Identification), with a per-block override saved in the
  preset.
- **Ronnie's real controls on the hardware.**
  - Gain in, Output and Mix knobs.
  - Normalize and LITE/FULL as toggle switches.
  - On pedals, an LCD showing the active capture; clicking it cycles captures. On amps, capture lights do the same.
  - The footswitch bypasses.

## Non-goals

- New DSP. Delay and reverb pedals can't be captured with NAM; space comes from space IRs.
- Tone controls that a NAM capture doesn't have (no fake Bass/Treble on amp faceplates).

## Anti-goals

- No makers' logos, brand scripts, artwork or signature trims (for example diamond plate or the EVH stripes). Skins
  evoke a family through layout and materials. Badges show the capture's own name.

## Identification (PR 1)

`ronie_core::gear::identify(&GearHints) -> GearIdentity { category, family, tone_type, source }`.

Hints are tried in priority order, and the first keyword hit wins:

1. `.nam` metadata `gear_make` and `gear_model`. Local imports keep these, along with `gear_type` and `tone_type`.
2. TONE3000 `makes`.
3. The tone title.

The declared category (TONE3000 `gear`, else `.nam` `gear_type`) limits which families may match. Keywords match whole
words after lowercasing, with punctuation turned into spaces. When nothing matches, the category's default applies:
amps by tone type (clean, crunch, hi-gain), pedals by tone type (dark for drive, light for clean), and a neutral cab.

Each `GearFamily` names a `skin_id()` (the rendered skin) and an `icon_id()` (the chain-strip icon).

## Rendering (PR 4)

- Skins are rendered in Blender from scripts:
  - `pedals.py`: five enclosure forms (treadle, standard, compact, row-of-three, wide).
  - `gear.py`: amps and cabs.
- All skins share one studio lighting rig. Pedals use a camera about 10° from straight down; amps and cabs use a front
  view about 9° above.
- Output: a transparent body image per skin, plus knob and switch filmstrips. egui draws the body and picks the filmstrip
  frame for each control's value.

## Pull requests

1. Gear identification in `crates/core`, plus a fix: local `.nam` imports read top-level `metadata`. They used to look
   under `config`, so `gear_type` was lost for real files.
2. Chain-strip icons and the + flow (`crates/ui`).
3. Archetype layout and theme (`crates/ui`).
4. Rendered skins: the scripts under `tools/`, the assets, and egui drawing with turnable knobs.

## Open questions

- Light theme as the default, or as a second theme next to the current dark one?
- How to store the images: compressed WebP or AVIF in git, Git LFS, or rendered at release time. Decided in PR 4: WebP
  in git under `crates/ui/assets/skins/`, embedded in the binary (about 4.6 MB for 23 skins).
