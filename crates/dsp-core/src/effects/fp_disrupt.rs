/// Fingerprint disruption — STFT-based spectral manipulation that attenuates peaks,
/// boosts adjacent bins, and applies micro phase modulation to alter the audio
/// fingerprint while preserving perceptual quality.

use crate::fft_utils::{stft, istft};
use rustfft::num_complex::Complex32;
use std::f32::consts::PI;

pub fn process_fp_disrupt(samples: &[f32], _sample_rate: u32, strength: f32) -> Vec<f32> {
    if strength <= 0.0 {
        return samples.to_vec();
    }

    let fft_size = 2048;
    let hop = fft_size / 2; // 50% overlap (was 25% — halves frame count)
    let mut frames = stft(samples, fft_size, hop);
    let half = fft_size / 2 + 1;
    let total_frames = frames.len();
    let min_peak_dist = 8;
    let threshold = 0.001f32;
    let peak_radius = 3;

    for (frame_idx, frame) in frames.iter_mut().enumerate() {
        // Compute magnitudes for peak detection
        let mags: Vec<f32> = (0..half)
            .map(|i| (frame[i].re * frame[i].re + frame[i].im * frame[i].im).sqrt())
            .collect();

        // Find local maxima with minimum distance constraint
        let mut peaks = Vec::new();
        let mut i = min_peak_dist;
        while i < half.saturating_sub(min_peak_dist) {
            if mags[i] < threshold {
                i += 1;
                continue;
            }
            let mut is_peak = true;
            for j in 1..=min_peak_dist {
                if i >= j && mags[i] < mags[i - j] {
                    is_peak = false;
                    break;
                }
                if i + j < half && mags[i] < mags[i + j] {
                    is_peak = false;
                    break;
                }
            }
            if is_peak {
                peaks.push(i);
                i += min_peak_dist;
            } else {
                i += 1;
            }
        }

        // Attenuate peaks by -(2 + strength*4) dB
        let atten_db = -(2.0 + strength * 4.0);
        let atten = 10.0f32.powf(atten_db / 20.0);

        // Boost adjacent non-peak bins by +(0.5 + strength*1.5) dB within radius 3
        let boost_db = 0.5 + strength * 1.5;
        let boost = 10.0f32.powf(boost_db / 20.0);

        for &peak in &peaks {
            frame[peak] = Complex32::new(frame[peak].re * atten, frame[peak].im * atten);

            for r in 1..=peak_radius {
                if peak + r < half && !peaks.contains(&(peak + r)) {
                    frame[peak + r] = Complex32::new(
                        frame[peak + r].re * boost,
                        frame[peak + r].im * boost,
                    );
                }
                if peak >= r && !peaks.contains(&(peak - r)) {
                    frame[peak - r] = Complex32::new(
                        frame[peak - r].re * boost,
                        frame[peak - r].im * boost,
                    );
                }
            }
        }

        // Micro phase modulation
        // phase += strength * 0.15 * sin(2*pi*frame/totalFrames*0.7) * bin/(N/2)
        for i in 0..half {
            let phase_mod = strength
                * 0.15
                * (2.0 * PI * frame_idx as f32 / total_frames as f32 * 0.7).sin()
                * i as f32
                / (fft_size as f32 / 2.0);
            let mag = (frame[i].re * frame[i].re + frame[i].im * frame[i].im).sqrt();
            let phase = frame[i].im.atan2(frame[i].re) + phase_mod;
            frame[i] = Complex32::new(mag * phase.cos(), mag * phase.sin());
            // Mirror conjugate
            if i > 0 && i < fft_size / 2 {
                frame[fft_size - i] = Complex32::new(frame[i].re, -frame[i].im);
            }
        }
    }

    // ISTFT to reconstruct
    let out = istft(&frames, fft_size, hop, samples.len());
    out[..samples.len().min(out.len())].to_vec()
}
