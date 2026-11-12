# Dual-IR Cab Blend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** One cab block can hold a second IR ("IR B", another mic position of the same cab pack) and blend it with
IR A live, with per-IR level and a phase invert on B, like a two-mic cab section in the Archetype plugins. Shown on the
cab page as a MICS panel (boards ArchCabOneIr and ArchCabBlend).

**Architecture:** IR B's identity lives in `ChainBlock` (a change rebuilds the chain, as any model change does); the
blend, levels and invert live in `BlockParams` (live through `UpdateBlockParams`, no rebuild). The DSP block runs two
zero-latency convolvers inside one oversampling island and mixes them with smoothed gains. Persistence adds optional
JUCE properties with defaults, so old presets load unchanged.

**Tech Stack:** Rust workspace (`core`, `dsp`, `plugin`, `app`, `ui`, `oracle`), egui.

**Spec:** docs/superpowers/specs/2026-11-11-archetype-ui-design.md; user decision 2026-11-12: "real blend" (one block,
two IRs in parallel), not the lane-B split. Boards: ArchCabOneIr, ArchCabBlend on
https://claude.ai/artifact/CaaEinMfx2MVib3hPSNz7x (markup:
/private/tmp/claude-501/-Users-vasilispoulopoulos-projects-ronnie-3000/dd2fa3ae-82be-49e2-9dc5-af365a8b9a68/scratchpad/canvas/project/ArchWindow.dc.html,
search `irPanel`).

## Global Constraints

- v1 scope: IR B is **another model of the same tone** (`IrB { model_id }`); the type is shaped so `tone: Option<Tone>`
  can be added later. B can be set only on IR tones whose two models both compile to `IrKind::Cab`; otherwise B is
  refused (UI does not offer it).
- Defaults: `ir_blend = 0.5`, `ir_a_level = 0.5`, `ir_b_level = 0.5` (normalized, 0.5 = unity, same law as
  `input_gain`), `ir_b_invert = false`. With no B, the block behaves bit-identically to today.
- Blend law: linear amplitude crossfade, `gA = (1 − blend)·levelA`, `gB = blend·levelB`, each IR keeping its own
  unit-energy normalization gain; the cab pad applies once to the sum; B invert is a sign flip.
- No allocation on the audio thread; no added latency; both convolvers always run while B is set.
- Old presets and sessions load unchanged; new JUCE properties: `irBModelId` (Int64, only when B is set; removed when
  cleared), `irBlend`, `irALevel`, `irBLevel`, `irBInvert`.
- Blend/levels are per-block settings like Mix: not host-automatable in v1.
- The reference layout must not change except that its block detail may gain nothing (dual-IR UI is Archetype only).
- `make check` green.

## Engineering notes (from the survey, file:line as of main d0050e8)

- Data: `ChainBlock` `core/src/chain.rs:241-258`; `BlockParams` `chain.rs:194-209`, parsed from UI JSON `chain.rs:225-236`
  and JUCE `chain.rs:832-843`; `references_model` `chain.rs:331-340`; `Tone::for_storage` `core/src/tone.rs:223-241`
  (catalog tones keep only the active model — must keep B too); byte cache `core/src/cache.rs:35-42,196`.
- Compile/restore: `plugin/src/persistence.rs` `live_model` :955, `resolve_model_bytes` :1005,
  `embedded_model_bytes` :1031-1059, `restored_model`/`prepare_lane` :1061-1132, `compile_block_model` :1134-1167,
  `active_model_lacks_bytes` :847, `encode_block` :448-530 (clones `previous` trees :462 — remove `irBModelId` when
  B is cleared), `encode_model_cache` :580-682; `core/src/state.rs:144-269` `restore_offline` (one job per block today;
  `reconciled_block_ids` :226-264); `plugin/src/processor.rs:349-365` `share_model_bytes`.
- DSP: `dsp/src/ir/decode.rs:18-121` (`compile_ir`, per-IR trim aligns onsets, resample to 48 kHz),
  `ir/mod.rs:9-69` (`CompiledIr`, cab pad), `ir/engine.rs:42-127` (`PreparedIr`, `process_base` :111 to become
  `pub(crate)`), `block/mod.rs:25-38` (`BlockModel`, `ModelProcessor`), :98-108 (`BlockControls::with_params`),
  :147-235 (`prepare_block`), :318-500 (process order; IR at :450-461), `prepare.rs:85-92` (IR tail),
  `oversampling.rs:140-185` (`base_rate_island`), `command.rs:17-22`, `rt_chain.rs:372-404`.
- Live vs rebuild: `plugin/src/audio_session.rs:41-105` (`ChainFingerprint` ignores params),
  `app/src/audio/engine/state.rs:600-662` (`sync`).
- UI: `ui/src/session_edit.rs` `select_model` :847, `select_model_with_bytes` :852-882, `edit_block_params_gesture`
  :785-820, `change_block` :1098-1107; `ui/src/services/load_flow.rs:194-317` (`ModelSwitchFlow`, one generation shared
  by all details — a B switch cancels an in-flight A switch; acceptable, test it); `ui/src/default/block.rs:333-350`
  (`select_model`), `model_select.rs` (picker, reuse for B); Archetype cab stage in `block.rs` `show_gear_stage`.
- Existing tests to extend: `dsp/tests/{ir.rs,ir_allocation.rs,block_allocation.rs,rt_chain.rs}`,
  `core/tests/{tone_chain,session_state,presets,persistence_fuzz,cache}.rs`, `plugin/tests/processor.rs`,
  `app/tests/audio_allocations.rs`, `ui/tests/archetype_ui.rs`, `session_edit.rs` inline tests,
  `ui/tests/load_flow.rs`, oracle state probes `oracle/src/state.rs`.

## Review Focus

- B's bytes silently dropped on save, restore offline, copy/duplicate or cache retain (any of `references_model`,
  `for_storage`, restore jobs missed). End-to-end: set B, save a preset, restart offline, B still plays.
- Blend drag: no rebuild, no swap fade, no allocation; undo is one step per drag.
- A preset written with B loaded into the original plugin (oracle state probe): it loads and plays A.
- Switching A's capture while B is downloading (shared switch generation): no stale bytes land in the wrong slot.
- Mixed lengths / source sample rates (44.1 k and 96 k IRs; one stereo, one mono): no latency change, no channel
  mismatch.

---

### Task 1: Core model and persistence shape

**Files:** `core/src/chain.rs`, `core/src/tone.rs`, `core/src/state.rs`, `core/src/cache.rs` (only if needed); tests
in `core/tests/`.

**Produces:** `pub struct IrB { pub model_id: ModelId }`; `ChainBlock.ir_b: Option<IrB>`; `BlockParams { ir_blend,
ir_a_level, ir_b_level, ir_b_invert, .. }` with the defaults above; `references_model` true for B;
`Tone::for_storage(active, extra: &[ModelId])` (or an equivalent that keeps B); JSON (`from_value`, UI JSON) and JUCE
(`from_juce_tree`) read/write with defaults; `restore_offline` creates a job per (block, model) and compares B in
reconciliation.

- [ ] Tests: `an_old_block_tree_loads_with_default_blend_and_no_b`; `a_block_with_b_round_trips_through_juce`;
  `for_storage_keeps_the_b_model_of_a_catalog_tone`; `references_model_includes_b`;
  `restore_offline_prepares_b`; `a_persistence_fuzz_case_with_b` (extend the fuzz generator).
- [ ] Implement (update all `BlockParams {` / `ChainBlock {` literals; prefer `..Default::default()`), `make check`,
  commit "Let a cab block name a second IR and its blend".

### Task 2: DSP pair

**Files:** `dsp/src/block/mod.rs`, `dsp/src/ir/engine.rs`, `dsp/src/prepare.rs`; tests in `dsp/tests/`; bench
`dsp/benches/full_chain.rs`.

**Consumes:** Task 1 `BlockParams` fields. **Produces:** `BlockModel::IrPair { a: Arc<CompiledIr>, b: Arc<CompiledIr> }`,
`ModelProcessor::IrPair`, `BlockControls` blend/level/invert fields; one `base_rate_island` call running A then B on a
preallocated scratch (`2 * max_frames`), mixing with 48 kHz `LinearGain` smoothers (~50 ms); the pair's block
normalization is 1 (per-IR normalization folded into the gains); tail = max of both.

- [ ] Tests: `a_pair_at_blend_zero_matches_a_alone` (bit-identical) and `..._one_matches_b_alone`;
  `a_pair_at_half_blend_is_the_mean` (impulse kernels, analytic); `inverting_b_subtracts_it`;
  `mixed_lengths_and_rates_add_no_latency` (44.1 k + 96 k, mono + stereo); `the_pair_works_through_2x_4x_8x`;
  `the_pair_path_does_not_allocate` (incl. param changes); `changing_the_blend_keeps_engine_state` (rt_chain,
  `UpdateBlockParams`, no swap fade). Add a bench case.
- [ ] Implement, `make check`, commit "Blend two cab IRs inside one block".

### Task 3: Compile, restore and save B

**Files:** `plugin/src/persistence.rs`, `plugin/src/processor.rs`, `plugin/src/audio_session.rs` (if needed),
`app/src/audio/engine/state.rs` (if needed); tests in `plugin/tests/processor.rs`, `oracle`.

**Consumes:** Tasks 1-2. **Produces:** `live_model`/`restored_model`/`prepare_lane`/`compile_block_model` build
`BlockModel::IrPair` when B is set (both models resolved; B missing bytes → A alone and the block reports
`model_loading` for B, never a crash); `active_model_lacks_bytes` covers B; `encode_block` writes/removes the new
properties; `encode_model_cache` embeds B.

- [ ] Tests: `a_session_with_b_saves_and_restores_offline_with_both_irs`; `clearing_b_removes_its_properties`;
  `a_missing_b_plays_a_alone`; a mixed-kind pair (cab + reverb) is refused; oracle: existing single-IR differentials
  unchanged, and a state probe that the reference loads a session written with B.
- [ ] Implement, `make check`, commit "Save and restore a cab's second IR".

### Task 4: Session editing and the MICS panel

**Files:** `ui/src/backend.rs` (`UiAction`), `ui/src/session_edit.rs`, `ui/src/services/load_flow.rs`,
`ui/src/default/block.rs`, `ui/src/default/model_select.rs` (reuse), `ui/src/default/archetype/*`; tests
`ui/tests/archetype_ui.rs`, `ui/tests/load_flow.rs`, `session_edit.rs` inline.

**Consumes:** Tasks 1-3. **Produces:** `UiAction::SelectSecondModel { block_id, model_id: Option<ModelId> }` and
`SelectSecondModelWithBytes` (or a slot field on the existing actions); a slot-aware `ModelSwitchTicket` (its "still
current" check names the slot); blend/levels/invert through `edit_block_params_gesture` (one undo step per drag); the
Archetype cab stage shows the MICS panel per the boards: render shifted left, a 300 pt panel right with "MICS" and
"1 IR"/"2 IRs blended", row A (black "A" badge, capture picker, level readout), and either a dashed "ADD A SECOND IR"
button with the help line "Blend a second mic: pick another mic position from this cab. Both play at once; BLEND mixes
them." or row B (accent "B" badge, picker, level) plus "Ø B" toggle, "REMOVE B", and a BLEND knob labelled A … B with a
"65 / 35" style readout. Levels are adjustable by dragging the readout (or a small knob) — keep it consistent with how
the existing light page edits values. Only for IR tones with at least two cab models; otherwise no panel.

- [ ] Tests: `a_cab_with_two_mics_offers_a_second_ir`; `adding_b_picks_another_mic_and_shows_the_blend`;
  `a_blend_drag_is_one_undo_step`; `removing_b_restores_a_alone`; `invert_toggles_b_phase`;
  `a_single_model_cab_has_no_second_ir`; `switching_a_while_b_downloads_lands_each_in_its_slot` (load_flow);
  `the_reference_layout_has_no_mics_panel`.
- [ ] Implement, `make check`, capture the two boards with the testbed (temporary edit, reverted), compare, commit
  "Show a cab's two mics and their blend".
