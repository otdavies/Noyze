/**
 * DSP Worker tests
 *
 * Tests the worker message protocol, WASM integration path,
 * error handling, and generation-based job cancellation.
 * Uses mock WASM functions since the real module requires compilation.
 */
import { describe, it, expect, beforeAll, vi } from 'vitest';

// Collect messages posted by the worker
const messages: any[] = [];
vi.stubGlobal('self', {
  postMessage: (msg: any) => messages.push(msg),
  onmessage: null as any,
});

// Stub performance.now for timing
vi.stubGlobal('performance', { now: () => Date.now() });

// Mock WASM module
let mockWasmInitResolve: () => void;
const mockWasmInitPromise = new Promise<void>((resolve) => {
  mockWasmInitResolve = resolve;
});

vi.mock('../../../crates/dsp-core/pkg/noyze_dsp.js', () => ({
  default: () => mockWasmInitPromise,
  // Legacy API
  process_chain: (
    inputL: Float32Array,
    inputR: Float32Array,
    _refL: Float32Array,
    _sr: number,
    _configJson: string,
  ) => {
    const hasRight = inputR && inputR.length > 0;
    const len = inputL.length;
    const output = new Float32Array(len * 2);
    for (let i = 0; i < len; i++) {
      output[i * 2] = inputL[i] * 0.98;
      output[i * 2 + 1] = (hasRight ? inputR[i] : inputL[i]) * 0.98;
    }
    return output;
  },
  // Stepped API
  process_mono: (
    inputL: Float32Array,
    _refL: Float32Array,
    _sr: number,
    _configJson: string,
  ) => {
    const output = new Float32Array(inputL.length);
    for (let i = 0; i < inputL.length; i++) {
      output[i] = inputL[i] * 0.98;
    }
    return output;
  },
  finalize_stereo: (
    outL: Float32Array,
    outR: Float32Array,
    _sr: number,
    _configJson: string,
  ) => {
    const len = Math.min(outL.length, outR.length);
    const output = new Float32Array(len * 2);
    for (let i = 0; i < len; i++) {
      output[i * 2] = outL[i];
      output[i * 2 + 1] = outR[i];
    }
    return output;
  },
  finalize_mono: (out: Float32Array) => {
    const output = new Float32Array(out.length * 2);
    for (let i = 0; i < out.length; i++) {
      output[i * 2] = out[i];
      output[i * 2 + 1] = out[i];
    }
    return output;
  },
}));

let workerOnMessage: (e: MessageEvent) => void;

function generateSineWave(freq: number, sr: number, duration: number): Float32Array {
  const len = Math.floor(sr * duration);
  const buf = new Float32Array(len);
  for (let i = 0; i < len; i++) {
    buf[i] = Math.sin(2 * Math.PI * freq * i / sr) * 0.5;
  }
  return buf;
}

function peakAmplitude(data: Float32Array): number {
  let max = 0;
  for (let i = 0; i < data.length; i++) {
    const v = Math.abs(data[i]);
    if (v > max) max = v;
  }
  return max;
}

function sendProcess(config: any, inputL: Float32Array, gen = 1, inputR: Float32Array | null = null) {
  messages.length = 0;
  workerOnMessage({
    data: {
      type: 'process',
      inputL,
      inputR,
      refL: null,
      config,
      sampleRate: 44100,
      _gen: gen,
    },
  } as MessageEvent);
}

function defaultConfig(): any {
  return {
    beats: null, reshape: null, reverb: null, warp: null, refWarp: null,
    saturate: null, excite: null, punch: null, autoEq: null, fpDisrupt: null,
    stereoWiden: null, subBass: null, tapeFlutter: null, seamlessLoop: false,
  };
}

describe('DSP Worker', () => {
  const sr = 44100;
  const sineInput = generateSineWave(440, sr, 0.5);

  beforeAll(async () => {
    await import('./worker');
    workerOnMessage = (self as any).onmessage;
  });

  describe('before WASM init', () => {
    it('queues message when WASM not ready (no immediate error)', () => {
      sendProcess(defaultConfig(), sineInput);
      const error = messages.find(m => m.type === 'error');
      // Worker now queues messages until WASM is ready instead of returning an error.
      // This supports the terminate-and-respawn cancellation pattern.
      expect(error).toBeUndefined();
    });
  });

  describe('after WASM init', () => {
    beforeAll(() => {
      mockWasmInitResolve();
      return new Promise(resolve => setTimeout(resolve, 10));
    });

    it('produces non-silent stereo output', () => {
      sendProcess(defaultConfig(), sineInput);
      const result = messages.find(m => m.type === 'result');
      expect(result).toBeDefined();
      expect(result.channels).toBe(2);
      expect(result.output.length).toBe(sineInput.length * 2);

      const left = new Float32Array(sineInput.length);
      for (let i = 0; i < left.length; i++) left[i] = result.output[i * 2];
      expect(peakAmplitude(left)).toBeGreaterThan(0.1);
    });

    it('passes generation counter through', () => {
      sendProcess(defaultConfig(), sineInput, 42);
      const result = messages.find(m => m.type === 'result');
      expect(result._gen).toBe(42);
    });

    it('emits multiple progress updates', () => {
      sendProcess(defaultConfig(), sineInput);
      const progressMsgs = messages.filter(m => m.type === 'progress');
      expect(progressMsgs.length).toBeGreaterThan(1);
      // Should have at least: 0.05, 0.90, 1.0
      expect(progressMsgs[progressMsgs.length - 1].value).toBe(1);
    });

    it('handles stereo input', () => {
      const inputR = generateSineWave(880, sr, 0.5);
      sendProcess(defaultConfig(), sineInput, 1, inputR);
      const result = messages.find(m => m.type === 'result');
      expect(result).toBeDefined();
      expect(result.channels).toBe(2);
    });

    it('progress reports have generation counter', () => {
      sendProcess(defaultConfig(), sineInput, 99);
      const progressMsgs = messages.filter(m => m.type === 'progress');
      for (const msg of progressMsgs) {
        expect(msg._gen).toBe(99);
      }
    });

    it('output contains no NaN or Infinity', () => {
      sendProcess(defaultConfig(), sineInput);
      const result = messages.find(m => m.type === 'result');
      for (let i = 0; i < result.output.length; i++) {
        expect(Number.isFinite(result.output[i])).toBe(true);
      }
    });
  });
});
