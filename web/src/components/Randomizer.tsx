import React, { useState } from 'react';
import type { ChainConfig } from '../dsp/types';

interface RandomizerProps {
  onRandomize: (config: ChainConfig, label: string) => void;
  onReset: () => void;
  onToggleLoop: () => void;
  isLooping: boolean;
}

type VibeKey =
  | 'polished_shift' | 'warm_tape' | 'section_rework' | 'punch_up'
  | 'dream_hall' | 'ghost_touch' | 'cinematic_slow'
  | 'lo_fi_haze' | 'crystal_air' | 'deep_space' | 'vinyl_crackle'
  | 'neon_bounce' | 'subterranean';

interface EffectWeight {
  activation: number;
  params: Record<string, [number, number]>;
}

interface VibeDef {
  displayName: string;
  partners: VibeKey[];
  modifying: Record<string, EffectWeight>;
  mastering: Record<string, EffectWeight>;
}

const VIBES: Record<VibeKey, VibeDef> = {
  polished_shift: {
    displayName: 'Polished Shift',
    partners: ['ghost_touch', 'punch_up', 'warm_tape', 'crystal_air', 'neon_bounce'],
    modifying: {
      reshape: { activation: 0.9, params: { spread: [1.0, 2.0], center: [500, 2000] } },
      reverb: { activation: 0.3, params: { size: [0.2, 0.4], damping: [0.4, 0.7], mix: [0.1, 0.2] } },
      stereoWiden: { activation: 0.5, params: { width: [1.1, 1.5] } },
    },
    mastering: {
      autoEq: { activation: 0.85, params: { intensity: [0.4, 0.8] } },
      excite: { activation: 0.6, params: { freq: [3000, 8000], amount: [0.1, 0.3], tone: [0.4, 0.7] } },
      saturate: { activation: 0.4, params: { drive: [2, 8], warmth: [0.3, 0.6] } },
    },
  },
  warm_tape: {
    displayName: 'Warm Tape',
    partners: ['polished_shift', 'dream_hall', 'section_rework', 'lo_fi_haze', 'vinyl_crackle'],
    modifying: {
      reshape: { activation: 0.6, params: { spread: [0.8, 1.5], center: [400, 1500] } },
      reverb: { activation: 0.5, params: { size: [0.2, 0.5], damping: [0.4, 0.8], mix: [0.1, 0.3] } },
      tapeFlutter: { activation: 0.8, params: { rate: [1.0, 3.0], depth: [0.2, 0.5], mix: [0.4, 0.7] } },
    },
    mastering: {
      saturate: { activation: 0.9, params: { drive: [4, 16], warmth: [0.5, 0.9] } },
      autoEq: { activation: 0.7, params: { intensity: [0.3, 0.6] } },
    },
  },
  section_rework: {
    displayName: 'Section Rework',
    partners: ['punch_up', 'ghost_touch', 'polished_shift', 'neon_bounce', 'subterranean'],
    modifying: {
      beats: { activation: 0.85, params: { reverseProb: [0.1, 0.4] } },
      warp: { activation: 0.5, params: { rate: [0.8, 1.2], grainMs: [40, 120] } },
      stereoWiden: { activation: 0.4, params: { width: [1.2, 1.6] } },
    },
    mastering: {
      excite: { activation: 0.7, params: { freq: [2000, 6000], amount: [0.2, 0.4], tone: [0.5, 0.8] } },
      punch: { activation: 0.5, params: { attack: [0.5, 0.9], sustain: [0.3, 0.7] } },
    },
  },
  punch_up: {
    displayName: 'Punch Up',
    partners: ['warm_tape', 'polished_shift', 'section_rework', 'subterranean', 'neon_bounce'],
    modifying: {
      reshape: { activation: 0.4, params: { spread: [1.0, 1.8], center: [500, 2000] } },
      subBass: { activation: 0.6, params: { amount: [0.3, 0.7], freq: [80, 120] } },
    },
    mastering: {
      punch: { activation: 0.95, params: { attack: [0.6, 1.0], sustain: [0.2, 0.5] } },
      saturate: { activation: 0.7, params: { drive: [4, 14], warmth: [0.2, 0.5] } },
      excite: { activation: 0.8, params: { freq: [2000, 6000], amount: [0.2, 0.4], tone: [0.5, 0.9] } },
    },
  },
  dream_hall: {
    displayName: 'Dream Hall',
    partners: ['ghost_touch', 'warm_tape', 'polished_shift', 'deep_space', 'crystal_air'],
    modifying: {
      reverb: { activation: 0.95, params: { size: [0.6, 1.0], damping: [0.2, 0.6], mix: [0.3, 0.6] } },
      warp: { activation: 0.6, params: { rate: [0.5, 0.9], grainMs: [80, 200] } },
      reshape: { activation: 0.5, params: { spread: [0.8, 1.3], center: [300, 1000] } },
      stereoWiden: { activation: 0.7, params: { width: [1.3, 1.8] } },
    },
    mastering: {
      saturate: { activation: 0.4, params: { drive: [2, 6], warmth: [0.5, 0.8] } },
      autoEq: { activation: 0.5, params: { intensity: [0.2, 0.5] } },
    },
  },
  ghost_touch: {
    displayName: 'Ghost Touch',
    partners: ['polished_shift', 'warm_tape', 'punch_up', 'lo_fi_haze', 'deep_space'],
    modifying: {
      reshape: { activation: 0.7, params: { spread: [0.7, 1.2], center: [500, 2000] } },
      reverb: { activation: 0.4, params: { size: [0.3, 0.6], damping: [0.3, 0.7], mix: [0.1, 0.3] } },
      tapeFlutter: { activation: 0.5, params: { rate: [0.5, 2.0], depth: [0.1, 0.3], mix: [0.3, 0.5] } },
    },
    mastering: {
      autoEq: { activation: 0.8, params: { intensity: [0.3, 0.7] } },
      fpDisrupt: { activation: 0.6, params: { strength: [0.2, 0.6] } },
    },
  },
  cinematic_slow: {
    displayName: 'Cinematic Slow',
    partners: ['dream_hall', 'warm_tape', 'ghost_touch', 'deep_space', 'lo_fi_haze'],
    modifying: {
      beats: { activation: 0.7, params: { reverseProb: [0.1, 0.3] } },
      warp: { activation: 0.8, params: { rate: [0.4, 0.7], grainMs: [100, 200] } },
      reverb: { activation: 0.85, params: { size: [0.7, 1.0], damping: [0.1, 0.4], mix: [0.3, 0.6] } },
      stereoWiden: { activation: 0.6, params: { width: [1.2, 1.7] } },
    },
    mastering: {
      saturate: { activation: 0.5, params: { drive: [2, 6], warmth: [0.6, 0.9] } },
      autoEq: { activation: 0.6, params: { intensity: [0.2, 0.5] } },
    },
  },

  // === NEW VIBES ===

  lo_fi_haze: {
    displayName: 'Lo-Fi Haze',
    partners: ['warm_tape', 'ghost_touch', 'cinematic_slow', 'vinyl_crackle', 'dream_hall'],
    modifying: {
      tapeFlutter: { activation: 0.9, params: { rate: [1.5, 4.0], depth: [0.3, 0.7], mix: [0.5, 0.8] } },
      reverb: { activation: 0.6, params: { size: [0.3, 0.6], damping: [0.6, 0.9], mix: [0.15, 0.35] } },
      reshape: { activation: 0.5, params: { spread: [0.7, 1.2], center: [300, 1200] } },
    },
    mastering: {
      saturate: { activation: 0.85, params: { drive: [6, 18], warmth: [0.6, 1.0] } },
      autoEq: { activation: 0.7, params: { intensity: [0.3, 0.6] } },
    },
  },
  crystal_air: {
    displayName: 'Crystal Air',
    partners: ['polished_shift', 'dream_hall', 'punch_up', 'neon_bounce', 'deep_space'],
    modifying: {
      stereoWiden: { activation: 0.85, params: { width: [1.3, 1.9] } },
      reshape: { activation: 0.7, params: { spread: [1.2, 2.5], center: [1500, 4000] } },
      reverb: { activation: 0.5, params: { size: [0.2, 0.5], damping: [0.1, 0.4], mix: [0.1, 0.25] } },
    },
    mastering: {
      excite: { activation: 0.9, params: { freq: [4000, 10000], amount: [0.15, 0.35], tone: [0.6, 0.9] } },
      autoEq: { activation: 0.7, params: { intensity: [0.4, 0.8] } },
      punch: { activation: 0.4, params: { attack: [0.3, 0.6], sustain: [-0.2, 0.2] } },
    },
  },
  deep_space: {
    displayName: 'Deep Space',
    partners: ['dream_hall', 'cinematic_slow', 'ghost_touch', 'crystal_air', 'subterranean'],
    modifying: {
      reverb: { activation: 0.95, params: { size: [0.8, 1.0], damping: [0.05, 0.3], mix: [0.4, 0.7] } },
      stereoWiden: { activation: 0.8, params: { width: [1.5, 2.0] } },
      warp: { activation: 0.5, params: { rate: [0.6, 0.85], grainMs: [120, 200] } },
      tapeFlutter: { activation: 0.4, params: { rate: [0.5, 1.5], depth: [0.15, 0.35], mix: [0.3, 0.5] } },
    },
    mastering: {
      saturate: { activation: 0.3, params: { drive: [2, 5], warmth: [0.4, 0.7] } },
      autoEq: { activation: 0.5, params: { intensity: [0.2, 0.5] } },
    },
  },
  vinyl_crackle: {
    displayName: 'Vinyl Crackle',
    partners: ['warm_tape', 'lo_fi_haze', 'ghost_touch', 'cinematic_slow', 'punch_up'],
    modifying: {
      tapeFlutter: { activation: 0.85, params: { rate: [0.8, 2.5], depth: [0.2, 0.5], mix: [0.4, 0.7] } },
      reshape: { activation: 0.6, params: { spread: [0.8, 1.3], center: [400, 1500] } },
      subBass: { activation: 0.4, params: { amount: [0.2, 0.5], freq: [60, 100] } },
    },
    mastering: {
      saturate: { activation: 0.9, params: { drive: [8, 20], warmth: [0.7, 1.0] } },
      autoEq: { activation: 0.8, params: { intensity: [0.3, 0.7] } },
      fpDisrupt: { activation: 0.3, params: { strength: [0.1, 0.3] } },
    },
  },
  neon_bounce: {
    displayName: 'Neon Bounce',
    partners: ['punch_up', 'section_rework', 'polished_shift', 'crystal_air', 'subterranean'],
    modifying: {
      beats: { activation: 0.8, params: { reverseProb: [0.05, 0.2] } },
      subBass: { activation: 0.7, params: { amount: [0.4, 0.8], freq: [80, 140] } },
      stereoWiden: { activation: 0.6, params: { width: [1.2, 1.6] } },
      reshape: { activation: 0.5, params: { spread: [1.0, 1.8], center: [800, 2500] } },
    },
    mastering: {
      punch: { activation: 0.9, params: { attack: [0.7, 1.0], sustain: [0.1, 0.4] } },
      excite: { activation: 0.75, params: { freq: [3000, 7000], amount: [0.15, 0.35], tone: [0.5, 0.8] } },
      saturate: { activation: 0.5, params: { drive: [3, 10], warmth: [0.2, 0.5] } },
    },
  },
  subterranean: {
    displayName: 'Subterranean',
    partners: ['punch_up', 'deep_space', 'section_rework', 'neon_bounce', 'warm_tape'],
    modifying: {
      subBass: { activation: 0.95, params: { amount: [0.5, 1.0], freq: [60, 120] } },
      reverb: { activation: 0.5, params: { size: [0.4, 0.7], damping: [0.5, 0.8], mix: [0.15, 0.3] } },
      reshape: { activation: 0.4, params: { spread: [0.8, 1.4], center: [200, 800] } },
      warp: { activation: 0.3, params: { rate: [0.85, 1.0], grainMs: [60, 140] } },
    },
    mastering: {
      saturate: { activation: 0.7, params: { drive: [4, 12], warmth: [0.4, 0.8] } },
      punch: { activation: 0.6, params: { attack: [0.5, 0.8], sustain: [0.3, 0.6] } },
      autoEq: { activation: 0.5, params: { intensity: [0.3, 0.6] } },
    },
  },
};

const VIBE_KEYS = Object.keys(VIBES) as VibeKey[];
const EQ_PRESETS = ['clean', 'warm', 'bright', 'full', 'dark'];

function rand(min: number, max: number) { return min + Math.random() * (max - min); }
function pick<T>(arr: T[]): T { return arr[Math.floor(Math.random() * arr.length)]; }
function clamp(v: number, lo: number, hi: number) { return Math.max(lo, Math.min(hi, v)); }

function blendWeights(
  a: Record<string, EffectWeight>,
  b: Record<string, EffectWeight>,
  wA: number,
  wB: number,
): Record<string, EffectWeight> {
  const merged: Record<string, EffectWeight> = {};
  const allKeys = new Set([...Object.keys(a), ...Object.keys(b)]);
  for (const k of allKeys) {
    const eA = a[k];
    const eB = b[k];
    if (eA && eB) {
      const params: Record<string, [number, number]> = {};
      const allParams = new Set([...Object.keys(eA.params), ...Object.keys(eB.params)]);
      for (const p of allParams) {
        const rA = eA.params[p] || eB.params[p];
        const rB = eB.params[p] || eA.params[p];
        params[p] = [rA[0] * wA + rB[0] * wB, rA[1] * wA + rB[1] * wB];
      }
      merged[k] = { activation: eA.activation * wA + eB.activation * wB, params };
    } else {
      const e = eA || eB;
      const w = eA ? wA : wB;
      merged[k] = { activation: e.activation * w, params: { ...e.params } };
    }
  }
  return merged;
}

/// Apply random micro-variation to all parameter ranges for more diversity
function jitterParams(weights: Record<string, EffectWeight>, jitterAmount: number): Record<string, EffectWeight> {
  const result: Record<string, EffectWeight> = {};
  for (const [key, ew] of Object.entries(weights)) {
    const params: Record<string, [number, number]> = {};
    for (const [p, [lo, hi]] of Object.entries(ew.params)) {
      const range = hi - lo;
      const jitter = range * jitterAmount;
      params[p] = [
        lo + (Math.random() - 0.5) * jitter,
        hi + (Math.random() - 0.5) * jitter,
      ];
    }
    result[key] = {
      activation: clamp(ew.activation + (Math.random() - 0.5) * jitterAmount * 0.3, 0, 1),
      params,
    };
  }
  return result;
}

export function generateRandomConfig(): { config: ChainConfig; label: string } {
  const primaryKey = pick(VIBE_KEYS);
  const primary = VIBES[primaryKey];
  const partnerKey = pick(primary.partners);
  const partner = VIBES[partnerKey];

  // Weighted blend with more variety (3 possible distributions)
  const blendStyle = Math.random();
  let wP: number;
  if (blendStyle < 0.4) {
    // Sqrt distribution (original - primary-heavy)
    wP = 0.3 + 0.4 * Math.sqrt(Math.random());
  } else if (blendStyle < 0.7) {
    // Even blend
    wP = 0.35 + Math.random() * 0.3;
  } else {
    // Secondary-heavy (partner dominates)
    wP = 0.2 + 0.3 * Math.random();
  }
  const wS = 1 - wP;
  const pctP = Math.round(wP * 100);

  // Blend and jitter for uniqueness
  const jitterAmount = 0.15 + Math.random() * 0.25; // 15-40% variation
  let mod = blendWeights(primary.modifying, partner.modifying, wP, wS);
  let mas = blendWeights(primary.mastering, partner.mastering, wP, wS);
  mod = jitterParams(mod, jitterAmount);
  mas = jitterParams(mas, jitterAmount);

  // Build effect configs
  const tryBuild = (name: string, w: EffectWeight): Record<string, unknown> | null => {
    if (Math.random() > w.activation) return null;
    const p: Record<string, unknown> = {};
    for (const [k, r] of Object.entries(w.params)) {
      p[k] = rand(r[0], r[1]);
    }
    return p;
  };

  // Build modifying
  const beats = mod.beats ? tryBuild('beats', mod.beats) : null;
  if (beats) {
    // More diverse beat options
    beats.reorder = Math.random() > 0.35;
    beats.halfTime = Math.random() > 0.75;
    beats.stutter = Math.random() > 0.55;
    beats.seed = Math.floor(Math.random() * 100000);
    if (!beats.reverseProb) beats.reverseProb = rand(0.05, 0.45);
  }
  const reshape = mod.reshape ? tryBuild('reshape', mod.reshape) : null;
  const reverb = mod.reverb ? tryBuild('reverb', mod.reverb) : null;
  const warp = mod.warp ? tryBuild('warp', mod.warp) : null;
  const subBass = mod.subBass ? tryBuild('subBass', mod.subBass) : null;
  const tapeFlutter = mod.tapeFlutter ? tryBuild('tapeFlutter', mod.tapeFlutter) : null;
  const stereoWiden = mod.stereoWiden ? tryBuild('stereoWiden', mod.stereoWiden) : null;

  // Build mastering
  const saturate = mas.saturate ? tryBuild('saturate', mas.saturate) : null;
  const excite = mas.excite ? tryBuild('excite', mas.excite) : null;
  const punch = mas.punch ? tryBuild('punch', mas.punch) : null;
  let autoEq = mas.autoEq ? tryBuild('autoEq', mas.autoEq) : null;
  if (autoEq) autoEq.preset = pick(EQ_PRESETS);
  const fpDisrupt = mas.fpDisrupt ? tryBuild('fpDisrupt', mas.fpDisrupt) : null;

  // Count active
  const modEffects = [beats, reshape, reverb, warp, subBass, tapeFlutter, stereoWiden];
  const masEffects = [saturate, excite, punch, autoEq];
  let modActive = modEffects.filter(Boolean).length;
  let masActive = masEffects.filter(Boolean).length;

  // Enforce minimums: 2 modifying + 1 mastering
  const config: ChainConfig = {
    beats: beats as ChainConfig['beats'],
    reshape: reshape as ChainConfig['reshape'],
    reverb: reverb as ChainConfig['reverb'],
    warp: warp as ChainConfig['warp'],
    refWarp: null,
    saturate: saturate as ChainConfig['saturate'],
    excite: excite as ChainConfig['excite'],
    punch: punch as ChainConfig['punch'],
    autoEq: autoEq as ChainConfig['autoEq'],
    fpDisrupt: fpDisrupt as ChainConfig['fpDisrupt'],
    stereoWiden: stereoWiden as ChainConfig['stereoWiden'],
    subBass: subBass as ChainConfig['subBass'],
    tapeFlutter: tapeFlutter as ChainConfig['tapeFlutter'],
    seamlessLoop: false,
  };

  // Force modifying if less than 2, choosing from the new effects too
  if (modActive < 2) {
    const fallbacks: Array<() => void> = [
      () => { if (!config.reshape) { config.reshape = { spread: rand(0.8, 2.0), center: rand(500, 2000) }; modActive++; } },
      () => { if (!config.stereoWiden) { config.stereoWiden = { width: rand(1.1, 1.6) }; modActive++; } },
      () => { if (!config.tapeFlutter) { config.tapeFlutter = { rate: rand(1.0, 3.0), depth: rand(0.2, 0.5), mix: rand(0.4, 0.7) }; modActive++; } },
      () => { if (!config.subBass) { config.subBass = { amount: rand(0.3, 0.6), freq: rand(70, 120) }; modActive++; } },
      () => { if (!config.reverb) { config.reverb = { size: rand(0.3, 0.7), damping: rand(0.3, 0.7), mix: rand(0.15, 0.4) }; modActive++; } },
    ];
    // Shuffle fallbacks for variety
    for (let i = fallbacks.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [fallbacks[i], fallbacks[j]] = [fallbacks[j], fallbacks[i]];
    }
    for (const fb of fallbacks) {
      if (modActive >= 2) break;
      fb();
    }
  }

  // Force mastering if less than 1
  if (masActive < 1) {
    const masFallbacks: Array<() => void> = [
      () => { if (!config.autoEq) { config.autoEq = { preset: pick(EQ_PRESETS), intensity: rand(0.4, 0.8) }; } },
      () => { if (!config.saturate) { config.saturate = { drive: rand(3, 8), warmth: rand(0.3, 0.6) }; } },
      () => { if (!config.excite) { config.excite = { freq: rand(3000, 8000), amount: rand(0.1, 0.3), tone: rand(0.4, 0.7) }; } },
    ];
    pick(masFallbacks)();
  }

  // Random chance (20%) to add one more "bonus" effect for extra spice
  if (Math.random() < 0.2) {
    const bonuses: Array<() => void> = [
      () => { if (!config.stereoWiden) config.stereoWiden = { width: rand(1.1, 1.4) }; },
      () => { if (!config.subBass) config.subBass = { amount: rand(0.2, 0.5), freq: rand(70, 110) }; },
      () => { if (!config.tapeFlutter) config.tapeFlutter = { rate: rand(1.0, 2.5), depth: rand(0.15, 0.35), mix: rand(0.3, 0.5) }; },
      () => { if (!config.punch) config.punch = { attack: rand(0.3, 0.7), sustain: rand(0.1, 0.4) }; },
    ];
    pick(bonuses)();
  }

  const label = `${primary.displayName} / ${partner.displayName} -- ${pctP}/${100 - pctP} blend`;
  return { config, label };
}

export function Randomizer({ onRandomize, onReset, onToggleLoop, isLooping }: RandomizerProps) {
  const [currentLabel, setCurrentLabel] = useState<string | null>(null);

  const handleRandomize = () => {
    const { config, label } = generateRandomConfig();
    setCurrentLabel(label);
    onRandomize(config, label);
  };

  return (
    <div className="space-y-2">
      <button
        onClick={handleRandomize}
        className="w-full py-3 rounded-lg font-bold text-sm tracking-widest
                   bg-gradient-to-r from-teal-600 to-cyan-500
                   hover:from-teal-500 hover:to-cyan-400
                   text-white transition-all active:scale-[0.98]"
      >
        RANDOMIZE
      </button>
      <div className="flex items-center gap-2">
        <button
          onClick={onToggleLoop}
          className={`px-4 py-1.5 rounded-full text-xs font-medium transition-colors ${
            isLooping
              ? 'bg-teal-500/20 text-teal-400 border border-teal-500/40'
              : 'bg-gray-800 text-gray-500 border border-gray-700'
          }`}
        >
          LOOP
        </button>
        <button
          onClick={() => { setCurrentLabel(null); onReset(); }}
          className="px-4 py-1.5 rounded-full text-xs font-medium
                     bg-gray-800 text-gray-500 border border-gray-700
                     hover:text-gray-300 hover:border-gray-600 transition-colors"
        >
          RESET
        </button>
        {currentLabel && (
          <span className="ml-2 text-xs text-gray-500 truncate">{currentLabel}</span>
        )}
      </div>
    </div>
  );
}
