# CLAUDE.md — Noyze Audio Workbench

## What This Is

Noyze is a browser-based audio effects processor. Rust DSP compiled to WASM handles the heavy lifting; React renders the UI. Audio goes in, gets processed through a configurable effect chain, comes out the other side.

**Live**: https://otdavies.github.io/Noyze/

## Architecture (Three-Layer Stack)

```
┌─────────────────────────────────┐
│  React UI (web/src/)            │  Components, state, playback
├─────────────────────────────────┤
│  Web Worker (web/src/dsp/)      │  Threading bridge, camelCase↔snake_case
├─────────────────────────────────┤
│  Rust/WASM (crates/dsp-core/)   │  DSP algorithms, effect chain
└─────────────────────────────────┘
```

Data flows top-down: User loads audio → React decodes → Worker sends to WASM → WASM processes → Worker returns result → React plays/exports.

## Build & Run

```bash
# WASM (must build first — web/ imports from crates/dsp-core/pkg/)
cd crates/dsp-core && wasm-pack build --target web --release

# Web dev server
cd web && npm install && npm run dev

# Tests
cd web && npm test

# Production build
cd web && tsc && npm run build
```

**CI/CD**: GitHub Actions (`.github/workflows/deploy.yml`) builds WASM + web, runs tests, deploys to GitHub Pages on push to `claude/github-pages-setup-6wn9a`.

## Critical Invariants

These are the rules that keep the system correct. Violating any of these will cause bugs.

### 1. Effect Registry Is the Single Source of Truth

`web/src/dsp/effectRegistry.ts` drives everything: UI generation, parameter controls, categories, defaults. Do NOT hardcode effect-specific logic elsewhere. The UI, preset system, and randomizer all derive from the registry.

### 2. Three-Layer Effect Sync

Every effect must exist in exactly three places, kept in sync:

| Layer | Files | What to add |
|-------|-------|-------------|
| **Rust** | `effects/foo.rs`, `effects/mod.rs`, `configs.rs`, `registry.rs` | Processing fn, config struct, chain dispatch |
| **TypeScript types** | `dsp/types.ts` | Config interface, ChainConfig field |
| **Registry** | `dsp/effectRegistry.ts` | EffectDef entry with params, defaults, category |

If these diverge, configs will silently fail (Rust ignores unknown fields via serde, TS won't send fields that aren't in its types).

### 3. camelCase ↔ snake_case Contract

TypeScript uses `camelCase` (e.g., `subBass`, `grainMs`). Rust uses `snake_case` (e.g., `sub_bass`, `grain_ms`). The `toSnakeCase()` function in `worker.ts` bridges this automatically. **Never rename a TS config field without checking the Rust counterpart.** The mapping is mechanical: `fooBar` → `foo_bar`.

### 4. Effect Processing Order

The canonical order lives in `registry.rs::process_mono_chain`:

```
Modifying:  beats → reshape → reverb → warp → ref_warp → sub_bass → tape_flutter
Mastering:  saturate → excite → punch → auto_eq → fp_disrupt
Flags:      seamless_loop
Stereo:     stereo_widen (runs in finalize_stereo, after mono processing)
```

This order matters for audio quality. Structural effects (beats, warp) must come before spectral effects. Mastering must come after modifying. Normalization happens once at the very end.

### 5. null = Disabled

An effect is active when its ChainConfig field is a config object. It is disabled when the field is `null`. Not `undefined`, not zeroed params — `null`. This convention is consistent across TS, JSON serialization, and Rust (`Option<T>` maps to `null`).

### 6. Normalization Happens Once

`interleave_and_normalize()` in `lib.rs` normalizes to **0.98 peak** after ALL processing. Individual effects must NOT normalize their output. This preserves relative dynamics through the chain and leaves headroom.

### 7. Generation Counter Prevents Stale Results

`jobGenRef` increments on each process request. The worker tags results with `_gen`. The app ignores results where `_gen !== jobGenRef.current`. This prevents race conditions when the user changes config rapidly.

### 8. Interleaved Stereo Output

WASM always returns `[L,R,L,R,...]` interleaved stereo `Float32Array`. Even mono input gets duplicated to stereo via `finalize_mono`. All downstream code (playback, visualization, export) depends on this format.

### 9. Full-Buffer Processing

The current pipeline processes the entire audio buffer in one pass — no chunking. This is intentional: effects like reverb, beats, and warp need full-buffer context for state continuity. The two-phase API (`process_structural` + `process_fx_chunk`) exists for future streaming but isn't used yet.

## How to Add a New Effect

Follow these steps in order. The system is designed so that adding one registry entry auto-generates the UI.

1. **Rust implementation**: Create `crates/dsp-core/src/effects/your_effect.rs` with a `pub fn process_your_effect(samples: &[f32], sr: u32, /* params */) -> Vec<f32>` function
2. **Rust module**: Add `pub mod your_effect;` to `effects/mod.rs`
3. **Rust config**: Add a config struct to `configs.rs` with `#[derive(Serialize, Deserialize, Clone, Debug)]`
4. **Rust registry**: Add `pub your_effect: Option<YourEffectConfig>` to `ChainConfig` in `registry.rs`, add dispatch in `process_mono_chain` in the correct position (modifying or mastering section), add `None` to `default_config()`
5. **TS types**: Add interface to `types.ts`, add field to `ChainConfig`, add `null` to `defaultConfig()`
6. **TS registry**: Add one `EffectDef` entry to `EFFECT_REGISTRY` in `effectRegistry.ts` — the UI, randomizer integration, and preset system all flow from this

That's it. No UI component changes needed.

## Code Conventions

### Naming
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `EFFECT_REGISTRY`, `MODIFYING_IDS`)
- **Functions**: `camelCase` in TS, `snake_case` in Rust
- **Types/Interfaces**: `PascalCase` with `Config` suffix for effect configs
- **Components**: `PascalCase` files and exports
- **Props**: `PascalCase` with `Props` suffix

### State Management
- All app state lives in `App.tsx` via `useState` hooks — no external state library
- Refs for non-rendered values: `useRef` for AudioContext, Worker, DOM nodes, timers
- `useCallback` for all handlers passed as props
- Config changes debounced at 300ms before triggering reprocessing

### Styling
- Tailwind CSS exclusively — no CSS modules, no styled-components
- Custom CSS only in `styles/index.css` for range input sliders
- Dark theme: `gray-950` background, `gray-200` text
- Accent colors: `teal/cyan` for primary, `blue` for modifying badges, `amber` for mastering badges

### Error Handling
- Worker errors surface as dismissible banners via `errorMessage` state
- WASM parse errors log to console and return input unchanged (graceful degradation)
- Audio source teardown wrapped in try/catch for already-stopped sources

## Testing

**Framework**: Vitest (`web/src/dsp/processing.test.ts`)

Tests verify:
- Worker message protocol (process/result/error/progress)
- Generation counter propagation
- WASM init state handling (before/after ready)
- Stereo and mono processing paths
- Output validity (no NaN/Infinity)

**Run**: `cd web && npm test` (also runs in CI)

**When to add tests**: Any change to the worker protocol, message format, or processing pipeline should include a test update. Effect algorithm correctness is validated through listening tests (qualitative), but the plumbing must be tested.

## Known Technical Debt

1. **ChainConfig duplication**: `worker.ts` redeclares the `ChainConfig` interface instead of importing from `types.ts`. The `toSnakeCase()` bridge makes it work, but changes to config shape must be updated in both places.
2. **Saturate sample rate**: `saturate.rs` uses hardcoded `dt = 1.0 / 44100.0` for its one-pole filter instead of the passed `sample_rate`. Works for 44.1kHz but would be wrong for 48kHz input.
3. **Single `as any` cast**: `worker.ts:137` — `[result.output.buffer] as any` for Transferable. Minor but could be typed properly.

## File Map (Key Files Only)

```
CLAUDE.md                              ← you are here
web/
  src/
    App.tsx                            ← root component, all state, playback logic
    dsp/
      types.ts                         ← TypeScript config interfaces (ChainConfig)
      effectRegistry.ts                ← SINGLE SOURCE OF TRUTH for effects
      worker.ts                        ← Web Worker, WASM bridge, camelCase→snake_case
      presets.ts                       ← factory + localStorage user presets
      processing.test.ts               ← worker protocol tests
    components/
      Controls.tsx                     ← reusable primitives (Toggle, Slider, Select)
      EffectControls.tsx               ← registry-driven effect UI generation
      Randomizer.tsx                   ← vibe blending + random config generation
      PresetBar.tsx                    ← preset load/save/navigate
      Transport.tsx                    ← playback controls + WAV export
      Waveform.tsx                     ← canvas waveform visualization
crates/dsp-core/
  src/
    lib.rs                             ← WASM entry points (process_mono, finalize_*)
    registry.rs                        ← ChainConfig struct, effect dispatch, processing order
    configs.rs                         ← all effect config structs (Rust side)
    fft_utils.rs                       ← STFT, FFT planner, Hann windowing
    effects/                           ← one file per effect algorithm
      mod.rs                           ← module declarations
      beats.rs, reverb.rs, saturate.rs, etc.
  pkg/                                 ← compiled WASM output (imported by web/)
```

## Self-Reinforcing Rules

These patterns are designed so that following them naturally prevents bugs:

1. **Registry-driven UI**: Adding an effect to `effectRegistry.ts` auto-generates its controls. You can't forget the UI.
2. **Serde tolerance**: Rust's `ChainConfig` uses `Option<T>` for every effect. Missing fields deserialize as `None`, so old configs still work with new code.
3. **Generation counter**: The `_gen` pattern means you can't accidentally show stale results, even under rapid config changes.
4. **Debounced reprocessing**: The 300ms debounce in `App.tsx` means rapid slider changes don't flood the worker.
5. **Graceful degradation**: WASM config parse errors return the input unchanged. Bad config never crashes — it just does nothing.
6. **Normalization at the end**: By normalizing only once in `interleave_and_normalize`, you can't accidentally clip intermediate stages.
7. **The checklist works**: The "How to Add a New Effect" section above is the exact checklist the codebase was built with. Following it guarantees a working effect.

## What NOT to Do

- **Don't normalize inside effects** — only `interleave_and_normalize` does this
- **Don't add React state outside App.tsx** for app-level concerns — component-local state for UI-only state is fine (e.g., PresetBar dropdown open state)
- **Don't skip the registry** — if an effect isn't in `effectRegistry.ts`, the UI won't know about it
- **Don't use `undefined` for disabled effects** — always `null`
- **Don't change the interleaved output format** — playback, visualization, and export all depend on `[L,R,L,R,...]`
- **Don't process on the main thread** — always use the Web Worker
- **Don't add dependencies without justification** — this project intentionally has minimal deps (React, Tailwind, Vite, that's it)
