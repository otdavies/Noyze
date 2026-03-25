import React from 'react';

interface TransportProps {
  hasOriginal: boolean;
  hasProcessed: boolean;
  isLooping: boolean;
  isPlaying: 'original' | 'processed' | null;
  activeTrack: 'original' | 'processed';
  onPlayPause: () => void;
  onToggleTrack: () => void;
  onExport: () => void;
  onRestart: () => void;
  onToggleLoop: () => void;
}

export function Transport({
  hasOriginal, hasProcessed, isLooping, isPlaying, activeTrack,
  onPlayPause, onToggleTrack, onExport, onRestart, onToggleLoop,
}: TransportProps) {
  const canPlay = activeTrack === 'processed' ? hasProcessed : hasOriginal;

  return (
    <div className="flex items-center gap-1.5">
      {/* Restart */}
      <button
        onClick={onRestart}
        disabled={!canPlay}
        className="w-9 h-9 flex items-center justify-center rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
        title="Restart"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <rect x="1" y="2" width="2" height="10" fill="currentColor"/>
          <path d="M12 7L5 2.5V11.5L12 7Z" fill="currentColor"/>
        </svg>
      </button>

      {/* Play / Pause */}
      <button
        onClick={onPlayPause}
        disabled={!canPlay}
        className={`w-11 h-11 flex items-center justify-center rounded-xl transition-all disabled:opacity-30 disabled:cursor-not-allowed ${
          isPlaying
            ? 'bg-teal-500 text-white hover:bg-teal-400'
            : 'bg-gray-800 text-gray-300 hover:bg-gray-700 hover:text-white'
        }`}
        title={isPlaying ? 'Pause' : 'Play'}
      >
        {isPlaying ? (
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <rect x="3" y="2" width="4" height="12" rx="1" fill="currentColor"/>
            <rect x="9" y="2" width="4" height="12" rx="1" fill="currentColor"/>
          </svg>
        ) : (
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 2L14 8L4 14V2Z" fill="currentColor"/>
          </svg>
        )}
      </button>

      {/* A/B toggle */}
      <button
        onClick={onToggleTrack}
        disabled={!hasOriginal}
        className={`h-9 px-3 rounded-lg text-xs font-bold tracking-wide transition-all disabled:opacity-30 disabled:cursor-not-allowed ${
          activeTrack === 'processed'
            ? 'bg-teal-500/15 text-teal-400 border border-teal-500/30'
            : 'bg-blue-500/15 text-blue-400 border border-blue-500/30'
        }`}
        title={`Listening to ${activeTrack === 'processed' ? 'processed' : 'original'} — click to switch`}
      >
        {activeTrack === 'processed' ? 'B' : 'A'}
      </button>

      {/* Loop */}
      <button
        onClick={onToggleLoop}
        className={`h-9 px-3 rounded-lg text-xs font-medium transition-colors ${
          isLooping
            ? 'bg-teal-500/15 text-teal-400 border border-teal-500/30'
            : 'bg-gray-800 text-gray-500 border border-gray-700 hover:text-gray-300'
        }`}
        title="Loop playback"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M11 4H4.5C3.12 4 2 5.12 2 6.5S3.12 9 4.5 9H9.5C10.88 9 12 10.12 12 11.5S10.88 14 9.5 14H3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" fill="none"/>
          <path d="M9 2L11.5 4L9 6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
        </svg>
      </button>

      <div className="flex-1" />

      {/* Export */}
      <button
        onClick={onExport}
        disabled={!hasProcessed}
        className="h-9 px-4 rounded-lg text-xs font-medium bg-gray-800 text-gray-400 hover:text-white hover:bg-gray-700 transition-colors border border-gray-700 disabled:opacity-30 disabled:cursor-not-allowed"
      >
        EXPORT WAV
      </button>
    </div>
  );
}

export function exportWav(
  audioData: Float32Array,
  channels: number,
  sampleRate: number,
  filename: string,
) {
  const numSamples = audioData.length / channels;
  const bytesPerSample = 2;
  const dataSize = numSamples * channels * bytesPerSample;
  const buffer = new ArrayBuffer(44 + dataSize);
  const view = new DataView(buffer);

  // RIFF header
  writeString(view, 0, 'RIFF');
  view.setUint32(4, 36 + dataSize, true);
  writeString(view, 8, 'WAVE');

  // fmt chunk
  writeString(view, 12, 'fmt ');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true); // PCM
  view.setUint16(22, channels, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * channels * bytesPerSample, true);
  view.setUint16(32, channels * bytesPerSample, true);
  view.setUint16(34, bytesPerSample * 8, true);

  // data chunk
  writeString(view, 36, 'data');
  view.setUint32(40, dataSize, true);

  let offset = 44;
  for (let i = 0; i < audioData.length; i++) {
    const s = Math.max(-1, Math.min(1, audioData[i]));
    const val = s < 0 ? s * 0x8000 : s * 0x7FFF;
    view.setInt16(offset, val, true);
    offset += 2;
  }

  const blob = new Blob([buffer], { type: 'audio/wav' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `noyze_${filename}.wav`;
  a.click();
  URL.revokeObjectURL(url);
}

function writeString(view: DataView, offset: number, str: string) {
  for (let i = 0; i < str.length; i++) {
    view.setUint8(offset + i, str.charCodeAt(i));
  }
}
