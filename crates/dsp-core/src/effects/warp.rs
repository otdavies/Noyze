/// WSOLA (Waveform Similarity Overlap-Add) time stretching.
///
/// Preserves pitch while changing playback speed by finding similar
/// overlap regions via cross-correlation and blending with Hann windows.

use crate::fft_utils::hann_window;

pub fn process_warp(samples: &[f32], sample_rate: u32, rate: f32, grain_ms: f32) -> Vec<f32> {
    let sr = sample_rate;
    if (rate - 1.0).abs() < 0.01 {
        return samples.to_vec();
    }

    let grain_n = ((grain_ms * sr as f32 / 1000.0) as usize).max(256);
    let hop_out = grain_n / 4;
    let hop_in = ((hop_out as f32 * rate) as usize).max(1);
    let window = hann_window(grain_n);

    // Lightweight search — only 64 samples for correlation, coarse step
    let tolerance = hop_in.max(32).min(grain_n / 4);
    let corr_len = 64;
    let coarse_step = 8.max(tolerance / 16);

    let output_len = (samples.len() as f32 / rate) as usize;
    let mut output = vec![0.0f32; output_len];
    let mut norm = vec![0.0f32; output_len];

    let mut in_pos = 0usize;
    let mut out_pos = 0usize;

    while out_pos + grain_n <= output_len && in_pos + grain_n <= samples.len() {
        let best_offset = if tolerance < 16 {
            // Extreme rate — skip search, use nominal position
            in_pos
        } else {
            let search_start = in_pos.saturating_sub(tolerance);
            let search_end = (in_pos + tolerance).min(samples.len().saturating_sub(grain_n));
            let mut best = in_pos;
            let mut best_corr = f32::MIN;

            let mut pos = search_start;
            while pos <= search_end {
                let corr = dot_correlate(samples, pos, in_pos, corr_len);
                if corr > best_corr {
                    best_corr = corr;
                    best = pos;
                }
                pos += coarse_step;
            }
            best
        };

        for i in 0..grain_n {
            if best_offset + i < samples.len() && out_pos + i < output_len {
                output[out_pos + i] += samples[best_offset + i] * window[i];
                norm[out_pos + i] += window[i];
            }
        }

        in_pos += hop_in;
        out_pos += hop_out;
    }

    for i in 0..output_len {
        if norm[i] > 1e-8 {
            output[i] /= norm[i];
        }
    }

    output
}

/// Fast unnormalized dot-product correlation — good enough for grain alignment.
#[inline]
fn dot_correlate(buf: &[f32], a_off: usize, b_off: usize, len: usize) -> f32 {
    let mut sum = 0.0f32;
    let end = len.min(buf.len().saturating_sub(a_off)).min(buf.len().saturating_sub(b_off));
    for i in 0..end {
        sum += buf[a_off + i] * buf[b_off + i];
    }
    sum
}
