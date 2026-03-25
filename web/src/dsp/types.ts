export interface BeatsConfig {
  reverseProb: number;
  reorder: boolean;
  halfTime: boolean;
  stutter: boolean;
  seed: number;
}

export interface ReshapeConfig {
  spread: number;
  center: number;
}

export interface ReverbConfig {
  size: number;
  damping: number;
  mix: number;
}

export interface WarpConfig {
  rate: number;
  grainMs: number;
}

export interface RefWarpConfig {
  amount: number;
}

export interface SaturateConfig {
  drive: number;
  warmth: number;
}

export interface ExciteConfig {
  freq: number;
  amount: number;
  tone: number;
}

export interface PunchConfig {
  attack: number;
  sustain: number;
}

export interface AutoEqConfig {
  preset: string;
  intensity: number;
}

export interface FpDisruptConfig {
  strength: number;
}

export interface StereoWidenConfig {
  width: number;
}

export interface SubBassConfig {
  amount: number;
  freq: number;
}

export interface TapeFlutterConfig {
  rate: number;
  depth: number;
  mix: number;
}

export interface DeepenConfig {
  amount: number;
  freq: number;
}

export interface ChainConfig {
  beats: BeatsConfig | null;
  reshape: ReshapeConfig | null;
  reverb: ReverbConfig | null;
  warp: WarpConfig | null;
  refWarp: RefWarpConfig | null;
  saturate: SaturateConfig | null;
  excite: ExciteConfig | null;
  punch: PunchConfig | null;
  autoEq: AutoEqConfig | null;
  fpDisrupt: FpDisruptConfig | null;
  stereoWiden: StereoWidenConfig | null;
  subBass: SubBassConfig | null;
  deepen: DeepenConfig | null;
  tapeFlutter: TapeFlutterConfig | null;
  seamlessLoop: boolean;
}

export type VibeName =
  | 'Polished Shift'
  | 'Warm Tape'
  | 'Section Rework'
  | 'Punch Up'
  | 'Dream Hall'
  | 'Ghost Touch'
  | 'Cinematic Slow';

export function defaultConfig(): ChainConfig {
  return {
    beats: null,
    reshape: null,
    reverb: null,
    warp: null,
    refWarp: null,
    saturate: null,
    excite: null,
    punch: null,
    autoEq: null,
    fpDisrupt: null,
    stereoWiden: null,
    subBass: null,
    deepen: null,
    tapeFlutter: null,
    seamlessLoop: false,
  };
}
