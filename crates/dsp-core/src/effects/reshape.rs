/// Reshape — spectral envelope warping via magnitude smoothing.
///
/// Extracts a smooth spectral envelope using a moving-average filter on
/// log-magnitudes (replacing the previous cepstral analysis which required
/// 2 extra FFTs per frame). The envelope is then warped with a power-law
/// function and applied as gain.

use crate::fft_utils::{stft, istft};
use rustfft::num_complex::Complex32;

pub fn process_reshape(samples: &[f32], sample_rate: u32, spread: f32, center: f32) -> Vec<f32> {
    let fft_size = 2048;
    let hop = fft_size / 2;
    let mut frames = stft(samples, fft_size, hop);
    let half = fft_size / 2 + 1;

    let center_bin = (center / (sample_rate as f32 / fft_size as f32)).round() as usize;
    let center_bin = center_bin.clamp(1, half - 1);

    let smooth_radius: usize = 15;

    // Reusable buffers (allocated once)
    let mut log_mag = vec![0.0f32; half];
    let mut envelope = vec![0.0f32; half];
    let mut target_envelope = vec![0.0f32; half];
    // Running-sum buffer for O(1)-per-bin moving average
    let mut prefix_sum = vec![0.0f32; half + 1];

    for frame in frames.iter_mut() {
        // Compute log magnitudes
        for i in 0..half {
            let mag = (frame[i].re * frame[i].re + frame[i].im * frame[i].im).sqrt();
            log_mag[i] = (mag + 1e-10).ln();
        }

        // Moving-average smoothing via prefix sum — O(half), replaces 2 FFTs
        prefix_sum[0] = 0.0;
        for i in 0..half {
            prefix_sum[i + 1] = prefix_sum[i] + log_mag[i];
        }
        for i in 0..half {
            let lo = if i >= smooth_radius { i - smooth_radius } else { 0 };
            let hi = (i + smooth_radius + 1).min(half);
            let count = (hi - lo) as f32;
            envelope[i] = (prefix_sum[hi] - prefix_sum[lo]) / count;
        }

        // Build warped envelope using power-law stretch
        for i in 0..half {
            let rel = if i < center_bin {
                let t = i as f32 / center_bin as f32;
                center_bin as f32 * t.powf(1.0 / spread)
            } else {
                let t = (i - center_bin) as f32 / (half - center_bin) as f32;
                center_bin as f32 + (half - center_bin) as f32 * t.powf(spread)
            };
            let src = rel.clamp(0.0, (half - 1) as f32);
            let idx = src as usize;
            let frac = src - idx as f32;
            let e0 = envelope[idx];
            let e1 = envelope.get(idx + 1).copied().unwrap_or(e0);
            target_envelope[i] = e0 * (1.0 - frac) + e1 * frac;
        }

        // Apply gain ratio
        for i in 0..half {
            let gain = (target_envelope[i] - envelope[i]).clamp(-2.5, 2.5);
            let multiplier = gain.exp();
            frame[i] = Complex32::new(frame[i].re * multiplier, frame[i].im * multiplier);
            if i > 0 && i < fft_size / 2 {
                frame[fft_size - i] = Complex32::new(frame[i].re, -frame[i].im);
            }
        }
    }

    let out = istft(&frames, fft_size, hop, samples.len());
    out[..samples.len().min(out.len())].to_vec()
}
