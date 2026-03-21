/* tslint:disable */
/* eslint-disable */

/**
 * Get default chain config as JSON
 */
export function default_config(): string;

/**
 * Process audio through the full effect chain (legacy single-call API).
 */
export function process_chain(input_l: Float32Array, input_r: Float32Array, ref_l: Float32Array, sample_rate: number, config_json: string): Float32Array;

/**
 * Process a single mono channel through all enabled effects (stepped API).
 */
export function process_mono(samples: Float32Array, ref_l: Float32Array, sample_rate: number, config_json: string): Float32Array;

/**
 * Apply stereo effects + interleave + normalize (stepped API).
 */
export function finalize_stereo(out_l: Float32Array, out_r: Float32Array, sample_rate: number, config_json: string): Float32Array;

/**
 * Interleave mono to stereo + normalize (stepped API).
 */
export function finalize_mono(out: Float32Array): Float32Array;

/**
 * Phase 1: Structural effects (beats, warp, ref_warp, seamless_loop).
 * Must process the full buffer. Fast — O(n), no FFT.
 */
export function process_structural(samples: Float32Array, ref_l: Float32Array, sample_rate: number, config_json: string): Float32Array;

/**
 * Phase 2: FX effects (reshape, reverb, saturate, etc.) on a chunk.
 * Caller should provide overlap at boundaries for STFT effects.
 */
export function process_fx_chunk(samples: Float32Array, sample_rate: number, config_json: string): Float32Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly default_config: () => [number, number];
    readonly process_chain: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
