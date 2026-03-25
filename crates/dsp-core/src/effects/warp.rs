/// WSOLA (Waveform Similarity Overlap-Add) time stretching.
///
/// Preserves pitch while changing playback speed by finding similar
/// overlap regions via cross-correlation and blending with Hann windows.

use crate::fft_utils::hann_window;

/// Linear interpolation for boundary fill
#[inline]
fn lerp_sample(buf: &[f32], pos: f32) -> f32 {
    let idx = pos as usize;
    if idx + 1 >= buf.len() { return buf[buf.len().saturating_sub(1)]; }
    let frac = pos - idx as f32;
    buf[idx] * (1.0 - frac) + buf[idx + 1] * frac
}

pub fn process_warp(samples: &[f32], sample_rate: u32, rate: f32, grain_ms: f32) -> Vec<f32> {
    let sr = sample_rate;
    if (rate - 1.0).abs() < 0.01 {
        return samples.to_vec();
    }

    // For slow rates, scale up grain size for smoother output
    let effective_grain_ms = if rate < 0.8 {
        grain_ms * (1.0 / rate).sqrt().min(1.8)
    } else {
        grain_ms
    };

    let grain_n = ((effective_grain_ms * sr as f32 / 1000.0) as usize).max(256);
    let hop_out = grain_n / 4;
    let hop_in = ((hop_out as f32 * rate) as usize).max(1);
    let window = hann_window(grain_n);

    // Scale correlation search based on rate — slower rates need wider search
    let tolerance = if rate < 0.6 {
        hop_in.max(grain_n / 3)
    } else {
        hop_in.max(32).min(grain_n / 4)
    };
    let corr_len = if rate < 0.8 { 128 } else { 64 };
    let coarse_step = 4.max(tolerance / 24);

    let output_len = (samples.len() as f32 / rate) as usize;
    let mut output = vec![0.0f32; output_len];
    let mut norm = vec![0.0f32; output_len];

    let mut in_pos = 0usize;
    let mut out_pos = 0usize;

    while out_pos + grain_n <= output_len && in_pos + grain_n <= samples.len() {
        let best_offset = if tolerance < 16 {
            in_pos
        } else {
            let search_start = in_pos.saturating_sub(tolerance);
            let search_end = (in_pos + tolerance).min(samples.len().saturating_sub(grain_n));
            let mut best = in_pos;
            let mut best_corr = f32::MIN;

            // Coarse pass
            let mut pos = search_start;
            while pos <= search_end {
                let corr = dot_correlate(samples, pos, in_pos, corr_len);
                if corr > best_corr {
                    best_corr = corr;
                    best = pos;
                }
                pos += coarse_step;
            }

            // Fine pass around best match
            let fine_start = best.saturating_sub(coarse_step);
            let fine_end = (best + coarse_step).min(search_end);
            for pos in fine_start..=fine_end {
                let corr = dot_correlate(samples, pos, in_pos, corr_len);
                if corr > best_corr {
                    best_corr = corr;
                    best = pos;
                }
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

    // Normalize OLA
    for i in 0..output_len {
        if norm[i] > 1e-8 {
            output[i] /= norm[i];
        }
    }

    // Fix boundary silence: crossfade from rate-mapped source at start and end
    // The Hann window produces near-zero values at boundaries where only 1 grain covers
    let boundary = hop_out;
    let boundary_len = boundary.min(output_len);
    for i in 0..boundary_len {
        let src_pos = (i as f32 * rate).min((samples.len() - 1) as f32);
        let src_val = lerp_sample(samples, src_pos);
        let t = i as f32 / boundary_len as f32;
        // Smooth crossfade: source → OLA
        let blend = t * t; // quadratic ease-in for smoother transition
        output[i] = src_val * (1.0 - blend) + output[i] * blend;
    }
    if output_len > boundary {
        let end_start = output_len - boundary_len;
        for i in 0..boundary_len {
            let out_idx = end_start + i;
            let src_pos = (out_idx as f32 * rate).min((samples.len() - 1) as f32);
            let src_val = lerp_sample(samples, src_pos);
            let t = i as f32 / boundary_len as f32;
            let blend = t * t;
            // Fade from OLA → source at the end
            output[out_idx] = output[out_idx] * (1.0 - blend) + src_val * blend;
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
