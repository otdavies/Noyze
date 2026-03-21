/**
 * Effect Registry - Single source of truth for all effects.
 *
 * To add a new effect:
 * 1. Add a Rust implementation in crates/dsp-core/src/effects/your_effect.rs
 * 2. Add a wrapper fn + registry line in crates/dsp-core/src/registry.rs
 * 3. Add a TS processing function in worker.ts
 * 4. Add ONE entry to EFFECT_REGISTRY below
 *
 * That's it. The UI, types, config, and randomizer all derive from this registry.
 */

export type ParamType = 'slider' | 'toggle' | 'select' | 'seed';

export interface ParamDef {
  key: string;
  type: ParamType;
  label: string;
  // Slider params
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  defaultValue?: number | boolean | string;
  // Select params
  options?: { value: string; label: string }[];
}

export type EffectCategory = 'modifying' | 'mastering';

export interface EffectDef {
  /** Config key name (must match Rust field name in snake_case, TS uses camelCase) */
  id: string;
  /** Display name in UI */
  label: string;
  /** modifying or mastering */
  category: EffectCategory;
  /** Parameter definitions for UI generation */
  params: ParamDef[];
  /** Optional description shown in UI */
  description?: string;
  /** Default config values when toggled on */
  defaults: Record<string, unknown>;
  /** Sub-toggles (like REORDER, HALF-TIME for beats) */
  toggles?: { key: string; label: string; defaultValue: boolean }[];
  /** Whether this is a "seed" effect that has a REROLL button */
  hasSeed?: boolean;
}

// ============================================================
// THE REGISTRY - add new effects here
// ============================================================

export const EFFECT_REGISTRY: EffectDef[] = [
  // ---- MODIFYING ----
  {
    id: 'beats',
    label: 'BEATS',
    category: 'modifying',
    defaults: { reverseProb: 0.3, reorder: true, halfTime: false, stutter: false, seed: 0 },
    toggles: [
      { key: 'reorder', label: 'REORDER', defaultValue: true },
      { key: 'halfTime', label: 'HALF-TIME', defaultValue: false },
      { key: 'stutter', label: 'STUTTER', defaultValue: false },
    ],
    hasSeed: true,
    params: [
      { key: 'reverseProb', type: 'slider', label: 'reverse prob', min: 0, max: 1, defaultValue: 0.3 },
    ],
  },
  {
    id: 'reshape',
    label: 'RESHAPE',
    category: 'modifying',
    defaults: { spread: 1.5, center: 1000 },
    params: [
      { key: 'spread', type: 'slider', label: 'spread', min: 0.5, max: 3, step: 0.1, defaultValue: 1.5 },
      { key: 'center', type: 'slider', label: 'center', min: 100, max: 4000, step: 10, unit: ' Hz', defaultValue: 1000 },
    ],
  },
  {
    id: 'reverb',
    label: 'REVERB',
    category: 'modifying',
    defaults: { size: 0.5, damping: 0.5, mix: 0.3 },
    params: [
      { key: 'size', type: 'slider', label: 'size', min: 0.1, max: 1, defaultValue: 0.5 },
      { key: 'damping', type: 'slider', label: 'damping', min: 0, max: 1, defaultValue: 0.5 },
      { key: 'mix', type: 'slider', label: 'mix', min: 0, max: 0.8, defaultValue: 0.3 },
    ],
  },
  {
    id: 'warp',
    label: 'WARP',
    category: 'modifying',
    defaults: { rate: 1.0, grainMs: 80 },
    params: [
      { key: 'rate', type: 'slider', label: 'rate', min: 0.25, max: 4, step: 0.05, defaultValue: 1.0 },
      { key: 'grainMs', type: 'slider', label: 'grain', min: 20, max: 200, step: 5, unit: ' ms', defaultValue: 80 },
    ],
  },
  {
    id: 'subBass',
    label: 'SUB BASS',
    category: 'modifying',
    defaults: { amount: 0.5, freq: 100 },
    description: 'Adds sub-octave harmonics for deeper bass',
    params: [
      { key: 'amount', type: 'slider', label: 'amount', min: 0, max: 1, defaultValue: 0.5 },
      { key: 'freq', type: 'slider', label: 'freq', min: 40, max: 200, step: 5, unit: ' Hz', defaultValue: 100 },
    ],
  },
  {
    id: 'tapeFlutter',
    label: 'TAPE FLUTTER',
    category: 'modifying',
    defaults: { rate: 2.0, depth: 0.4, mix: 0.6 },
    description: 'Analog tape wow & flutter modulation',
    params: [
      { key: 'rate', type: 'slider', label: 'rate', min: 0.5, max: 6, step: 0.1, unit: ' Hz', defaultValue: 2.0 },
      { key: 'depth', type: 'slider', label: 'depth', min: 0, max: 1, defaultValue: 0.4 },
      { key: 'mix', type: 'slider', label: 'mix', min: 0, max: 1, defaultValue: 0.6 },
    ],
  },
  {
    id: 'stereoWiden',
    label: 'STEREO WIDEN',
    category: 'modifying',
    defaults: { width: 1.5 },
    description: 'Mid/side stereo field enhancement',
    params: [
      { key: 'width', type: 'slider', label: 'width', min: 0.5, max: 2.0, step: 0.05, defaultValue: 1.5 },
    ],
  },
  {
    id: 'refWarp',
    label: 'REF WARP',
    category: 'modifying',
    defaults: { amount: 0.2 },
    params: [
      { key: 'amount', type: 'slider', label: 'amount', min: 0, max: 0.5, defaultValue: 0.2 },
    ],
  },

  // ---- MASTERING ----
  {
    id: 'saturate',
    label: 'SATURATE',
    category: 'mastering',
    defaults: { drive: 6, warmth: 0.3 },
    params: [
      { key: 'drive', type: 'slider', label: 'drive', min: 1, max: 24, step: 0.5, unit: ' dB', defaultValue: 6 },
      { key: 'warmth', type: 'slider', label: 'warmth', min: 0, max: 1, defaultValue: 0.3 },
    ],
  },
  {
    id: 'excite',
    label: 'EXCITE',
    category: 'mastering',
    defaults: { freq: 3000, amount: 0.2, tone: 0.5 },
    params: [
      { key: 'freq', type: 'slider', label: 'freq', min: 1000, max: 12000, step: 100, unit: ' Hz', defaultValue: 3000 },
      { key: 'amount', type: 'slider', label: 'amount', min: 0, max: 0.5, defaultValue: 0.2 },
      { key: 'tone', type: 'slider', label: 'tone', min: 0, max: 1, defaultValue: 0.5 },
    ],
  },
  {
    id: 'punch',
    label: 'PUNCH',
    category: 'mastering',
    defaults: { attack: 0.5, sustain: 0 },
    params: [
      { key: 'attack', type: 'slider', label: 'attack', min: -1, max: 1, defaultValue: 0.5 },
      { key: 'sustain', type: 'slider', label: 'sustain', min: -1, max: 1, defaultValue: 0 },
    ],
  },
  {
    id: 'autoEq',
    label: 'AUTO EQ',
    category: 'mastering',
    defaults: { preset: 'full', intensity: 0.7 },
    params: [
      {
        key: 'preset', type: 'select', label: 'preset', defaultValue: 'full',
        options: [
          { value: 'clean', label: 'Clean' },
          { value: 'warm', label: 'Warm' },
          { value: 'bright', label: 'Bright' },
          { value: 'full', label: 'Full' },
          { value: 'dark', label: 'Dark' },
        ],
      },
      { key: 'intensity', type: 'slider', label: 'intensity', min: 0, max: 1, defaultValue: 0.7 },
    ],
  },
  {
    id: 'fpDisrupt',
    label: 'FP DISRUPT',
    category: 'mastering',
    defaults: { strength: 0.5 },
    description: 'Disrupts audio fingerprint constellation maps. Sounds identical, different fingerprint.',
    params: [
      { key: 'strength', type: 'slider', label: 'strength', min: 0, max: 1, defaultValue: 0.5 },
    ],
  },
];

// ============================================================
// Derived helpers - these auto-generate from the registry
// ============================================================

/** All modifying effect IDs */
export const MODIFYING_IDS = EFFECT_REGISTRY.filter(e => e.category === 'modifying').map(e => e.id);

/** All mastering effect IDs */
export const MASTERING_IDS = EFFECT_REGISTRY.filter(e => e.category === 'mastering').map(e => e.id);

/** All effect IDs */
export const ALL_EFFECT_IDS = EFFECT_REGISTRY.map(e => e.id);

/** Lookup an effect definition by ID */
export function getEffectDef(id: string): EffectDef | undefined {
  return EFFECT_REGISTRY.find(e => e.id === id);
}

/** Generate a default ChainConfig from the registry */
export function defaultConfigFromRegistry(): Record<string, unknown> {
  const config: Record<string, unknown> = {};
  for (const effect of EFFECT_REGISTRY) {
    config[effect.id] = null;
  }
  config.seamlessLoop = false;
  return config;
}
