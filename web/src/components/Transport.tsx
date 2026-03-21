import React from 'react';

interface TransportProps {
  hasOriginal: boolean;
  hasProcessed: boolean;
  isLooping: boolean;
  isPlaying: 'original' | 'processed' | null;
  onPlayOriginal: () => void;
  onPlayProcessed: () => void;
  onStop: () => void;
  onExport: () => void;
  onRestart: () => void;
}

export function Transport({
  hasOriginal, hasProcessed, isLooping, isPlaying,
  onPlayOriginal, onPlayProcessed, onStop, onExport, onRestart,
}: TransportProps) {
  const btnBase = 'px-4 py-1.5 rounded text-xs font-medium transition-colors disabled:opacity-30 disabled:cursor-not-allowed';

  return (
    <div className="flex items-center gap-2 flex-wrap">
      <button
        onClick={onPlayOriginal}
        disabled={!hasOriginal}
        className={`${btnBase} ${isPlaying === 'original' ? 'bg-blue-500 text-white' : 'bg-gray-800 text-gray-300 hover:bg-gray-700'}`}
      >
        ORIGINAL
      </button>
      <button
        onClick={onPlayProcessed}
        disabled={!hasProcessed}
        className={`${btnBase} ${isPlaying === 'processed' ? 'bg-teal-500 text-white' : 'bg-gray-800 text-gray-300 hover:bg-gray-700'}`}
      >
        {isLooping ? 'PLAY LOOP' : 'PROCESSED'}
      </button>
      <button
        onClick={onStop}
        disabled={!isPlaying}
        className={`${btnBase} bg-gray-800 text-gray-300 hover:bg-gray-700`}
      >
        STOP
      </button>
      <button
        onClick={onRestart}
        disabled={!isPlaying}
        className={`${btnBase} bg-gray-800 text-gray-300 hover:bg-gray-700`}
        title="Restart from beginning"
      >
        RESTART
      </button>
      <div className="flex-1" />
      <button
        onClick={onExport}
        disabled={!hasProcessed}
        className={`${btnBase} bg-gray-800 text-gray-300 hover:bg-gray-700`}
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
