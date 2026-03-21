import React, { useRef, useEffect, useCallback } from 'react';

interface WaveformProps {
  original: Float32Array | null;
  processed: Float32Array | null;
  sampleRate: number;
  playbackPosition: number; // 0-1 normalized position
  isFlashing: boolean;
  isPlaying: 'original' | 'processed' | null;
  onSeek: (position: number) => void; // 0-1 normalized
}

export function Waveform({ original, processed, sampleRate, playbackPosition, isFlashing, isPlaying, onSeek }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = 120 * dpr;
    canvas.style.height = '120px';

    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.scale(dpr, dpr);

    const w = rect.width;
    const h = 120;
    const mid = h / 2;

    // Track background colors based on which is active
    const origActive = isPlaying === 'original';
    const procActive = isPlaying === 'processed';

    // Original half background
    ctx.fillStyle = origActive ? '#0c1020' : '#08080c';
    ctx.fillRect(0, 0, w, mid);

    // Processed half background
    if (isFlashing) {
      ctx.fillStyle = '#0d1a1a';
    } else {
      ctx.fillStyle = procActive ? '#0a1414' : '#08080c';
    }
    ctx.fillRect(0, mid, w, mid);

    // Draw separator
    ctx.strokeStyle = '#1f2937';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, mid);
    ctx.lineTo(w, mid);
    ctx.stroke();

    // Draw waveform helper
    const drawWave = (data: Float32Array, yOffset: number, halfH: number, color: string, dimmed: boolean) => {
      const step = Math.max(1, Math.floor(data.length / w));
      ctx.globalAlpha = dimmed ? 0.35 : 1.0;
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let x = 0; x < w; x++) {
        const idx = Math.floor((x / w) * data.length);
        let min = 0, max = 0;
        for (let j = 0; j < step; j++) {
          const s = data[idx + j] || 0;
          if (s < min) min = s;
          if (s > max) max = s;
        }
        const y1 = yOffset + (1 - max) * halfH * 0.5;
        const y2 = yOffset + (1 - min) * halfH * 0.5;
        ctx.moveTo(x, y1);
        ctx.lineTo(x, y2);
      }
      ctx.stroke();
      ctx.globalAlpha = 1.0;
    };

    // When playing, dim the inactive track; when stopped, both full brightness
    if (original) {
      const dimOrig = isPlaying !== null && !origActive;
      drawWave(original, 0, mid, origActive ? '#60a5fa' : '#3b82f6', dimOrig);
    }
    if (processed) {
      const dimProc = isPlaying !== null && !procActive;
      const color = isFlashing ? '#5eeacc' : (procActive ? '#5eeacc' : '#2dd4bf');
      drawWave(processed, mid, mid, color, dimProc);
    }

    // Active track border indicator
    if (isPlaying !== null) {
      ctx.strokeStyle = origActive ? '#3b82f6' : '#2dd4bf';
      ctx.lineWidth = 2;
      const yStart = origActive ? 0 : mid;
      ctx.strokeRect(0, yStart, w, mid);
    }

    // Playback position line - always visible when audio is loaded
    if ((original || processed) && playbackPosition >= 0 && playbackPosition <= 1) {
      const x = Math.max(0.5, Math.min(playbackPosition * w, w - 0.5));
      ctx.strokeStyle = '#f59e0b';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, h);
      ctx.stroke();

      // Triangle at top
      ctx.fillStyle = '#f59e0b';
      ctx.beginPath();
      ctx.moveTo(x - 5, 0);
      ctx.lineTo(x + 5, 0);
      ctx.lineTo(x, 7);
      ctx.closePath();
      ctx.fill();

      // Triangle at bottom
      ctx.beginPath();
      ctx.moveTo(x - 5, h);
      ctx.lineTo(x + 5, h);
      ctx.lineTo(x, h - 7);
      ctx.closePath();
      ctx.fill();
    }

    // Labels
    ctx.font = '10px monospace';
    ctx.fillStyle = origActive ? '#93bbfc' : '#6b7280';
    ctx.fillText('ORIGINAL', 4, 12);
    ctx.fillStyle = procActive ? '#5eeacc' : '#6b7280';
    ctx.fillText('PROCESSED', 4, mid + 12);

    if (original) {
      ctx.fillStyle = '#6b7280';
      const dur = (original.length / sampleRate).toFixed(1);
      ctx.fillText(dur + 's', w - 30, 12);
    }
  }, [original, processed, sampleRate, playbackPosition, isFlashing, isPlaying]);

  useEffect(() => {
    draw();
  }, [draw]);

  // Also redraw on resize
  useEffect(() => {
    const handleResize = () => draw();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [draw]);

  const handleClick = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const position = x / rect.width;
    onSeek(Math.max(0, Math.min(1, position)));
  }, [onSeek]);

  return (
    <canvas
      ref={canvasRef}
      className={`w-full rounded-lg border cursor-pointer transition-colors duration-300 ${
        isFlashing ? 'border-teal-500/60' : 'border-gray-800'
      }`}
      style={{ height: '120px' }}
      onClick={handleClick}
    />
  );
}
