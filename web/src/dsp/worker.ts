// Noyze DSP Worker - WASM audio processing
//
// Processes the full buffer through all effects in a single pass.
// No chunking — effects must process the entire buffer to maintain
// state continuity (reverb tails, filter states, LFO phases, STFT context).
//
// If a message arrives before WASM is ready, it is queued and processed
// once initialization completes. This supports the terminate-and-respawn
// cancellation pattern used by the main thread.

import init, {
  process_chain as wasm_process_chain,
  process_mono as wasm_process_mono,
  finalize_stereo as wasm_finalize_stereo,
  finalize_mono as wasm_finalize_mono,
} from '../../../crates/dsp-core/pkg/noyze_dsp.js';

let wasmReady = false;
let wasmError: string | null = null;
let pendingMessage: MessageEvent | null = null;

init()
  .then(() => {
    wasmReady = true;
    self.postMessage({ type: 'ready' });
    // Process any message that arrived while WASM was loading
    if (pendingMessage) {
      const e = pendingMessage;
      pendingMessage = null;
      handleProcessMessage(e);
    }
  })
  .catch((err: unknown) => {
    wasmError = `WASM initialization failed: ${String(err)}`;
  });

interface ChainConfig {
  beats: { reverseProb: number; reorder: boolean; halfTime: boolean; stutter: boolean; seed: number } | null;
  reshape: { spread: number; center: number } | null;
  reverb: { size: number; damping: number; mix: number } | null;
  warp: { rate: number; grainMs: number } | null;
  refWarp: { amount: number } | null;
  saturate: { drive: number; warmth: number } | null;
  excite: { freq: number; amount: number; tone: number } | null;
  punch: { attack: number; sustain: number } | null;
  autoEq: { preset: string; intensity: number } | null;
  fpDisrupt: { strength: number } | null;
  stereoWiden: { width: number } | null;
  subBass: { amount: number; freq: number } | null;
  deepen: { amount: number; freq: number } | null;
  tapeFlutter: { rate: number; depth: number; mix: number } | null;
  seamlessLoop: boolean;
}

let currentGen: number | undefined;

function progress(value: number) {
  self.postMessage({ type: 'progress', value, _gen: currentGen });
}

function toSnakeCase(config: ChainConfig): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(config)) {
    const snakeKey = key.replace(/([A-Z])/g, '_$1').toLowerCase();
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      const inner: Record<string, unknown> = {};
      for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
        inner[k.replace(/([A-Z])/g, '_$1').toLowerCase()] = v;
      }
      result[snakeKey] = inner;
    } else {
      result[snakeKey] = value;
    }
  }
  return result;
}

function processAudio(
  inputL: Float32Array,
  inputR: Float32Array | null,
  refL: Float32Array | null,
  config: ChainConfig,
  sr: number,
): { output: Float32Array; channels: number; sampleRate: number } {
  const configJson = JSON.stringify(toSnakeCase(config));
  const refData = refL || new Float32Array(0);
  const isStereo = inputR !== null && inputR.length === inputL.length;
  const startTime = performance.now();

  // Use stepped API (process_mono + finalize) when available
  const hasStepped = typeof wasm_process_mono === 'function';

  if (hasStepped) {
    progress(0.05);
    const outL = wasm_process_mono(inputL, refData, sr, configJson);

    if (isStereo) {
      progress(0.50);
      const outR = wasm_process_mono(inputR!, refData, sr, configJson);
      progress(0.90);
      const output = new Float32Array(wasm_finalize_stereo(outL, outR, sr, configJson));
      console.log(`[Noyze] ${(inputL.length / sr).toFixed(1)}s stereo: ${(performance.now() - startTime).toFixed(0)}ms`);
      progress(1);
      return { output, channels: 2, sampleRate: sr };
    } else {
      progress(0.90);
      const output = new Float32Array(
        config.stereoWiden
          ? wasm_finalize_stereo(outL, outL, sr, configJson)
          : wasm_finalize_mono(outL)
      );
      console.log(`[Noyze] ${(inputL.length / sr).toFixed(1)}s mono: ${(performance.now() - startTime).toFixed(0)}ms`);
      progress(1);
      return { output, channels: 2, sampleRate: sr };
    }
  }

  // Legacy single-call API
  progress(0.10);
  const result = wasm_process_chain(inputL, inputR || new Float32Array(0), refData, sr, configJson);
  console.log(`[Noyze] ${(inputL.length / sr).toFixed(1)}s: ${(performance.now() - startTime).toFixed(0)}ms`);
  progress(1);
  return { output: new Float32Array(result), channels: 2, sampleRate: sr };
}

function handleProcessMessage(e: MessageEvent) {
  const { type, inputL, inputR, refL, config, sampleRate, _gen } = e.data;
  currentGen = _gen;

  if (type !== 'process') return;

  try {
    const result = processAudio(
      new Float32Array(inputL),
      inputR ? new Float32Array(inputR) : null,
      refL ? new Float32Array(refL) : null,
      config,
      sampleRate,
    );
    self.postMessage({
      type: 'result',
      output: result.output,
      channels: result.channels,
      sampleRate: result.sampleRate,
      _gen,
    }, [result.output.buffer] as any);
  } catch (err) {
    self.postMessage({ type: 'error', message: String(err), _gen });
  }
}

// ---- WORKER MESSAGE HANDLER ----
self.onmessage = (e: MessageEvent) => {
  if (!wasmReady) {
    if (wasmError) {
      const { _gen } = e.data;
      self.postMessage({ type: 'error', message: wasmError, _gen });
    } else {
      // WASM still loading — queue this message (replaces any previous pending)
      pendingMessage = e;
    }
    return;
  }
  handleProcessMessage(e);
};
