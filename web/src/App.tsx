import React, { useState, useRef, useCallback, useEffect } from 'react';
import type { ChainConfig } from './dsp/types';
import { defaultConfig } from './dsp/types';
import { Waveform } from './components/Waveform';
import { Transport, exportWav } from './components/Transport';
import { Randomizer } from './components/Randomizer';
import { EffectControls } from './components/EffectControls';
import { PresetBar } from './components/PresetBar';
import { MODIFYING_IDS, MASTERING_IDS } from './dsp/effectRegistry';

const MODIFYING_KEYS = MODIFYING_IDS as (keyof ChainConfig)[];
const MASTERING_KEYS = MASTERING_IDS as (keyof ChainConfig)[];

export default function App() {
  const [audioBuffer, setAudioBuffer] = useState<AudioBuffer | null>(null);
  const [processedAudio, setProcessedAudio] = useState<Float32Array | null>(null);
  const [processedChannels, setProcessedChannels] = useState(2);
  const [processedSampleRate, setProcessedSampleRate] = useState(44100);
  const [referenceBuffer, setReferenceBuffer] = useState<AudioBuffer | null>(null);
  const [chainConfig, setChainConfig] = useState<ChainConfig>(defaultConfig());
  const [isProcessing, setIsProcessing] = useState(false);
  const [vibeLabel, setVibeLabel] = useState<string | null>(null);
  const [isLooping, setIsLooping] = useState(false);
  const [isPlaying, setIsPlaying] = useState<'original' | 'processed' | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [refFileName, setRefFileName] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);
  const [playbackPosition, setPlaybackPosition] = useState(0);
  const [isFlashing, setIsFlashing] = useState(false);
  const [seekPosition, setSeekPosition] = useState(0); // 0-1, where playback should start
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const audioCtxRef = useRef<AudioContext | null>(null);
  const workerRef = useRef<Worker | null>(null);
  const sourceRef = useRef<AudioBufferSourceNode | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const refInputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const animFrameRef = useRef<number | null>(null);
  const playStartTimeRef = useRef(0);
  const playOffsetRef = useRef(0); // offset in seconds where playback started
  const playDurationRef = useRef(0); // total duration of playing buffer
  const jobGenRef = useRef(0); // generation counter for cancelling stale results
  const isProcessingRef = useRef(false); // tracks worker busy state (ref, not render-dependent)

  const createWorker = useCallback(() => {
    const worker = new Worker(
      new URL('./dsp/worker.ts', import.meta.url),
      { type: 'module' }
    );
    worker.onmessage = (e: MessageEvent) => {
      const msg = e.data;
      // 'ready' has no _gen — let it through silently
      if (msg.type === 'ready') return;
      // Ignore results from stale jobs
      if (msg._gen !== undefined && msg._gen !== jobGenRef.current) return;
      if (msg.type === 'progress') {
        setProgress(msg.value);
      } else if (msg.type === 'result') {
        isProcessingRef.current = false;
        setProcessedAudio(msg.output);
        setProcessedChannels(msg.channels);
        setProcessedSampleRate(msg.sampleRate);
        setIsProcessing(false);
        setProgress(0);

        setIsFlashing(true);
        setTimeout(() => setIsFlashing(false), 400);
      } else if (msg.type === 'error') {
        isProcessingRef.current = false;
        setIsProcessing(false);
        setErrorMessage(msg.message);
        console.error('Worker error:', msg.message);
      }
    };
    return worker;
  }, []);

  useEffect(() => {
    workerRef.current = createWorker();
    return () => workerRef.current?.terminate();
  }, [createWorker]);

  const getAudioCtx = useCallback(() => {
    if (!audioCtxRef.current) audioCtxRef.current = new AudioContext();
    return audioCtxRef.current;
  }, []);

  const decodeFile = useCallback(async (file: File): Promise<AudioBuffer> => {
    const ctx = getAudioCtx();
    const arrayBuffer = await file.arrayBuffer();
    return ctx.decodeAudioData(arrayBuffer);
  }, [getAudioCtx]);

  const processAudio = useCallback((config: ChainConfig, buffer: AudioBuffer | null, ref: AudioBuffer | null) => {
    if (!buffer) return;

    // Bump generation to invalidate any in-flight results
    const gen = ++jobGenRef.current;

    // If the worker is still processing a previous job, kill it and start fresh.
    // WASM processing is synchronous inside the worker so messages just queue up —
    // terminating is the only way to truly cancel.
    if (isProcessingRef.current && workerRef.current) {
      workerRef.current.terminate();
      workerRef.current = createWorker();
    }

    if (!workerRef.current) {
      workerRef.current = createWorker();
    }

    isProcessingRef.current = true;
    setIsProcessing(true);
    setProgress(0);
    setErrorMessage(null);

    const inputL = buffer.getChannelData(0);
    const inputR = buffer.numberOfChannels > 1 ? buffer.getChannelData(1) : null;
    const refL = ref ? ref.getChannelData(0) : null;

    workerRef.current.postMessage({
      type: 'process',
      inputL,
      inputR,
      refL,
      config,
      sampleRate: buffer.sampleRate,
      _gen: gen,
    });
  }, [createWorker]);

  // Debounced reprocess on config change
  useEffect(() => {
    if (!audioBuffer) return;
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      processAudio(chainConfig, audioBuffer, referenceBuffer);
    }, 300);
    return () => { if (debounceRef.current) clearTimeout(debounceRef.current); };
  }, [chainConfig, audioBuffer, referenceBuffer, processAudio]);

  const handleFileLoad = useCallback(async (file: File) => {
    try {
      const buffer = await decodeFile(file);
      setAudioBuffer(buffer);
      setFileName(file.name);
      setProcessedAudio(null);
      setSeekPosition(0);
      setPlaybackPosition(0);
    } catch { console.error('Failed to decode audio file'); }
  }, [decodeFile]);

  const handleRefLoad = useCallback(async (file: File) => {
    try {
      const buffer = await decodeFile(file);
      setReferenceBuffer(buffer);
      setRefFileName(file.name);
    } catch { console.error('Failed to decode reference file'); }
  }, [decodeFile]);

  const handleDragOver = useCallback((e: React.DragEvent) => { e.preventDefault(); }, []);
  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    const file = e.dataTransfer.files[0];
    if (file) handleFileLoad(file);
  }, [handleFileLoad]);

  // Animation loop for playback position
  const updatePlaybackPosition = useCallback(() => {
    const ctx = audioCtxRef.current;
    const source = sourceRef.current;
    if (!ctx || !source) return; // Don't reset position — just stop animating

    const elapsed = ctx.currentTime - playStartTimeRef.current + playOffsetRef.current;
    const duration = playDurationRef.current;
    if (duration > 0) {
      const pos = elapsed / duration;
      if (pos >= 1 && !source.loop) {
        // Playback ended naturally — onended will handle state cleanup
        return;
      }
      setPlaybackPosition(pos % 1);
    }
    animFrameRef.current = requestAnimationFrame(updatePlaybackPosition);
  }, []);

  // Internal: tear down audio source without touching React state
  const teardownSource = useCallback(() => {
    if (animFrameRef.current) {
      cancelAnimationFrame(animFrameRef.current);
      animFrameRef.current = null;
    }
    if (sourceRef.current) {
      try { sourceRef.current.stop(); } catch { /* already stopped */ }
      sourceRef.current.disconnect();
      sourceRef.current = null;
    }
  }, []);

  const stopPlayback = useCallback(() => {
    teardownSource();
    setIsPlaying(null);
  }, [teardownSource]);

  const playBuffer = useCallback((buffer: AudioBuffer, loop: boolean, which: 'original' | 'processed', startPosition = 0) => {
    // Tear down old source WITHOUT setting isPlaying=null (avoids flicker)
    teardownSource();

    const ctx = getAudioCtx();
    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.loop = loop;
    source.connect(ctx.destination);

    const offsetSeconds = startPosition * buffer.duration;
    playStartTimeRef.current = ctx.currentTime;
    playOffsetRef.current = offsetSeconds;
    playDurationRef.current = buffer.duration;

    // Guard onended against stale sources — only act if THIS source is still current
    source.onended = () => {
      if (sourceRef.current !== source) return; // Stale — a new source replaced us
      if (!loop) {
        sourceRef.current = null;
        setIsPlaying(null);
        setPlaybackPosition(0);
        if (animFrameRef.current) {
          cancelAnimationFrame(animFrameRef.current);
          animFrameRef.current = null;
        }
      }
    };

    source.start(0, offsetSeconds);
    sourceRef.current = source;
    // Set state atomically — no intermediate null
    setIsPlaying(which);
    animFrameRef.current = requestAnimationFrame(updatePlaybackPosition);
  }, [getAudioCtx, teardownSource, updatePlaybackPosition]);

  const makeProcessedBuffer = useCallback(() => {
    if (!processedAudio) return null;
    const ctx = getAudioCtx();
    const ch = processedChannels;
    const sr = processedSampleRate;
    const samplesPerCh = Math.floor(processedAudio.length / ch);
    const abuf = ctx.createBuffer(ch, samplesPerCh, sr);
    for (let c = 0; c < ch; c++) {
      const data = abuf.getChannelData(c);
      for (let i = 0; i < samplesPerCh; i++) data[i] = processedAudio[i * ch + c];
    }
    return abuf;
  }, [processedAudio, processedChannels, processedSampleRate, getAudioCtx]);

  const handlePlayOriginal = useCallback(() => {
    if (!audioBuffer) return;
    playBuffer(audioBuffer, false, 'original', seekPosition);
  }, [audioBuffer, playBuffer, seekPosition]);

  const handlePlayProcessed = useCallback(() => {
    const abuf = makeProcessedBuffer();
    if (!abuf) return;
    playBuffer(abuf, isLooping, 'processed', seekPosition);
  }, [makeProcessedBuffer, isLooping, playBuffer, seekPosition]);

  // Auto-resume after reprocessing
  useEffect(() => {
    if (processedAudio && seekPosition > 0 && isPlaying === 'processed') {
      // Re-trigger playback from stored position
      const abuf = makeProcessedBuffer();
      if (abuf) {
        playBuffer(abuf, isLooping, 'processed', seekPosition);
      }
    }
  }, [processedAudio]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleExport = useCallback(() => {
    if (!processedAudio) return;
    const name = fileName ? fileName.replace(/\.[^.]+$/, '') : 'output';
    exportWav(processedAudio, processedChannels, processedSampleRate, name);
  }, [processedAudio, processedChannels, processedSampleRate, fileName]);

  const handleRandomize = useCallback((config: ChainConfig, label: string) => {
    config.seamlessLoop = isLooping;
    setChainConfig(config);
    setVibeLabel(label);
  }, [isLooping]);

  const handleLoadPreset = useCallback((config: ChainConfig, name: string) => {
    setChainConfig({ ...config, seamlessLoop: isLooping });
    setVibeLabel(name);
  }, [isLooping]);

  const handleReset = useCallback(() => {
    stopPlayback();
    setChainConfig(defaultConfig());
    setVibeLabel(null);
    setProcessedAudio(null);
    setSeekPosition(0);
  }, [stopPlayback]);

  const handleToggleLoop = useCallback(() => {
    setIsLooping(prev => {
      const next = !prev;
      setChainConfig(c => ({ ...c, seamlessLoop: next }));
      return next;
    });
  }, []);

  const handleSeek = useCallback((position: number) => {
    setSeekPosition(position);

    // If currently playing, restart from new position
    if (isPlaying === 'original' && audioBuffer) {
      playBuffer(audioBuffer, false, 'original', position);
    } else if (isPlaying === 'processed') {
      const abuf = makeProcessedBuffer();
      if (abuf) {
        playBuffer(abuf, isLooping, 'processed', position);
      }
    } else {
      // Not playing, just show the position
      setPlaybackPosition(position);
    }
  }, [isPlaying, audioBuffer, isLooping, playBuffer, makeProcessedBuffer]);

  const handleRestart = useCallback(() => {
    setSeekPosition(0);
    if (isPlaying === 'original' && audioBuffer) {
      playBuffer(audioBuffer, false, 'original', 0);
    } else if (isPlaying === 'processed') {
      const abuf = makeProcessedBuffer();
      if (abuf) {
        playBuffer(abuf, isLooping, 'processed', 0);
      }
    }
  }, [isPlaying, audioBuffer, isLooping, playBuffer, makeProcessedBuffer]);

  const originalData = audioBuffer ? audioBuffer.getChannelData(0) : null;
  const processedData = processedAudio
    ? (() => {
        const ch = processedChannels;
        const len = Math.floor(processedAudio.length / ch);
        const mono = new Float32Array(len);
        for (let i = 0; i < len; i++) mono[i] = processedAudio[i * ch];
        return mono;
      })()
    : null;

  const sampleRate = audioBuffer?.sampleRate || 44100;

  const activeModLabels = MODIFYING_KEYS.filter(k => chainConfig[k] != null);
  const activeMasLabels = MASTERING_KEYS.filter(k => chainConfig[k] != null);

  return (
    <div
      className="min-h-screen bg-gray-950 text-gray-200 font-mono p-4 max-w-2xl mx-auto space-y-4"
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      {/* Header */}
      <div className="flex items-center gap-3 flex-wrap">
        <h1 className="text-xl font-bold tracking-widest text-white">NOYZE</h1>
        <div className="flex-1" />
        <button
          onClick={() => fileInputRef.current?.click()}
          className="px-3 py-1.5 rounded text-xs font-medium bg-gray-800 text-gray-300 hover:bg-gray-700 transition-colors border border-gray-700"
        >
          Load Audio
        </button>
        <button
          onClick={() => refInputRef.current?.click()}
          className="px-3 py-1.5 rounded text-xs font-medium bg-gray-800 text-gray-300 hover:bg-gray-700 transition-colors border border-gray-700"
        >
          Load Reference
        </button>
        <input ref={fileInputRef} type="file" accept="audio/*" className="hidden"
          onChange={e => { const f = e.target.files?.[0]; if (f) handleFileLoad(f); }} />
        <input ref={refInputRef} type="file" accept="audio/*" className="hidden"
          onChange={e => { const f = e.target.files?.[0]; if (f) handleRefLoad(f); }} />
      </div>

      {(fileName || refFileName) && (
        <div className="flex items-center gap-4 text-[10px] text-gray-500">
          {fileName && <span>Audio: {fileName}</span>}
          {refFileName && <span className="text-blue-400">Ref: {refFileName}</span>}
        </div>
      )}

      {/* Randomizer */}
      <Randomizer
        onRandomize={handleRandomize}
        onReset={handleReset}
        onToggleLoop={handleToggleLoop}
        isLooping={isLooping}
      />

      {/* Preset Bar */}
      <PresetBar config={chainConfig} onLoadPreset={handleLoadPreset} />

      {/* Status line */}
      <div className="flex items-center gap-2 flex-wrap min-h-[24px]">
        {vibeLabel && <span className="text-xs text-gray-400">{vibeLabel}</span>}
        {isProcessing && (
          <span className="text-xs text-yellow-500 animate-pulse">
            Processing... {Math.round(progress * 100)}%
          </span>
        )}
        <div className="flex-1" />
        {activeModLabels.map(k => (
          <span key={k} className="text-[10px] px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400">
            {String(k).toUpperCase()}
          </span>
        ))}
        {activeMasLabels.map(k => (
          <span key={k} className="text-[10px] px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-400">
            {String(k).toUpperCase()}
          </span>
        ))}
      </div>

      {/* Error banner */}
      {errorMessage && (
        <div className="flex items-center gap-2 p-3 rounded-lg bg-red-500/10 border border-red-500/30">
          <span className="text-xs text-red-400 flex-1">{errorMessage}</span>
          <button
            onClick={() => setErrorMessage(null)}
            className="text-red-400 hover:text-red-300 text-xs"
          >
            dismiss
          </button>
        </div>
      )}

      {/* Drop zone hint when no audio */}
      {!audioBuffer && (
        <div className="border-2 border-dashed border-gray-800 rounded-lg p-8 text-center">
          <p className="text-gray-600 text-sm">Drop an audio file here or click Load Audio</p>
        </div>
      )}

      {/* Waveform */}
      <Waveform
        original={originalData}
        processed={processedData}
        sampleRate={sampleRate}
        playbackPosition={playbackPosition}
        isFlashing={isFlashing}
        isPlaying={isPlaying}
        onSeek={handleSeek}
      />

      {/* Transport */}
      <Transport
        hasOriginal={!!audioBuffer}
        hasProcessed={!!processedAudio}
        isLooping={isLooping}
        isPlaying={isPlaying}
        onPlayOriginal={handlePlayOriginal}
        onPlayProcessed={handlePlayProcessed}
        onStop={stopPlayback}
        onExport={handleExport}
        onRestart={handleRestart}
      />

      {/* Effect Controls */}
      <EffectControls
        config={chainConfig}
        onChange={setChainConfig}
        hasReference={!!referenceBuffer}
      />
    </div>
  );
}
