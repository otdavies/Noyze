import type { ChainConfig } from './types';
import { defaultConfig } from './types';

export interface Preset {
  name: string;
  config: ChainConfig;
  factory?: boolean;
}

const STORAGE_KEY = 'noyze-presets';

// ============================================================
// Factory presets — curated starting points
// ============================================================

const FACTORY_PRESETS: Preset[] = [
  {
    name: 'INIT',
    config: defaultConfig(),
    factory: true,
  },
  {
    name: 'WARM TAPE',
    config: {
      ...defaultConfig(),
      saturate: { drive: 0.3, warmth: 0.6 },
      tapeFlutter: { rate: 2.5, depth: 0.4, mix: 0.35 },
      autoEq: { preset: 'warm', intensity: 0.7 },
    },
    factory: true,
  },
  {
    name: 'LOFI CRUNCH',
    config: {
      ...defaultConfig(),
      saturate: { drive: 0.6, warmth: 0.8 },
      autoEq: { preset: 'dark', intensity: 0.5 },
      tapeFlutter: { rate: 4.0, depth: 0.6, mix: 0.4 },
      reverb: { size: 0.3, damping: 0.7, mix: 0.15 },
    },
    factory: true,
  },
  {
    name: 'BRIGHT MASTER',
    config: {
      ...defaultConfig(),
      autoEq: { preset: 'bright', intensity: 0.6 },
      excite: { freq: 3000, amount: 0.4, tone: 0.6 },
      saturate: { drive: 0.15, warmth: 0.3 },
      punch: { attack: 0.5, sustain: 0.3 },
    },
    factory: true,
  },
  {
    name: 'FULL BODY',
    config: {
      ...defaultConfig(),
      autoEq: { preset: 'full', intensity: 0.7 },
      subBass: { amount: 0.4, freq: 60 },
      saturate: { drive: 0.2, warmth: 0.5 },
      punch: { attack: 0.6, sustain: 0.4 },
    },
    factory: true,
  },
  {
    name: 'WIDE ROOM',
    config: {
      ...defaultConfig(),
      reverb: { size: 0.6, damping: 0.4, mix: 0.25 },
      stereoWiden: { width: 0.6 },
      autoEq: { preset: 'clean', intensity: 0.4 },
    },
    factory: true,
  },
  {
    name: 'HALF-TIME',
    config: {
      ...defaultConfig(),
      beats: { reverseProb: 0.0, reorder: false, halfTime: true, stutter: false, seed: 0 },
      saturate: { drive: 0.2, warmth: 0.4 },
      autoEq: { preset: 'dark', intensity: 0.4 },
    },
    factory: true,
  },
  {
    name: 'GLITCH',
    config: {
      ...defaultConfig(),
      beats: { reverseProb: 0.3, reorder: true, halfTime: false, stutter: true, seed: 42 },
      reshape: { spread: 1.8, center: 1200 },
      saturate: { drive: 0.4, warmth: 0.3 },
    },
    factory: true,
  },
  {
    name: 'VINYL DIG',
    config: {
      ...defaultConfig(),
      saturate: { drive: 0.5, warmth: 0.9 },
      tapeFlutter: { rate: 1.5, depth: 0.3, mix: 0.5 },
      autoEq: { preset: 'warm', intensity: 0.8 },
      subBass: { amount: 0.3, freq: 55 },
      fpDisrupt: { strength: 0.3 },
    },
    factory: true,
  },
  {
    name: 'CLEAN LOUD',
    config: {
      ...defaultConfig(),
      autoEq: { preset: 'clean', intensity: 0.6 },
      saturate: { drive: 0.1, warmth: 0.2 },
      excite: { freq: 4000, amount: 0.3, tone: 0.5 },
      punch: { attack: 0.7, sustain: 0.5 },
    },
    factory: true,
  },
];

// ============================================================
// Storage
// ============================================================

function loadUserPresets(): Preset[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    return JSON.parse(raw) as Preset[];
  } catch {
    return [];
  }
}

function saveUserPresets(presets: Preset[]): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(presets));
}

export function getAllPresets(): Preset[] {
  return [...FACTORY_PRESETS, ...loadUserPresets()];
}

export function getFactoryPresets(): Preset[] {
  return FACTORY_PRESETS;
}

export function getUserPresets(): Preset[] {
  return loadUserPresets();
}

export function savePreset(name: string, config: ChainConfig): Preset {
  const presets = loadUserPresets();
  const existing = presets.findIndex(p => p.name === name);
  const preset: Preset = { name, config: structuredClone(config) };

  if (existing >= 0) {
    presets[existing] = preset;
  } else {
    presets.push(preset);
  }

  saveUserPresets(presets);
  return preset;
}

export function deletePreset(name: string): void {
  const presets = loadUserPresets().filter(p => p.name !== name);
  saveUserPresets(presets);
}

export function renamePreset(oldName: string, newName: string): void {
  const presets = loadUserPresets();
  const preset = presets.find(p => p.name === oldName);
  if (preset) {
    preset.name = newName;
    saveUserPresets(presets);
  }
}
