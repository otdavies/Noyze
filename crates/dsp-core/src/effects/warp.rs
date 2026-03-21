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
    let hop_out = grain_n / 4; // 75% overlap
    let hop_in = (hop_out as f32 * rate) as usize;
    let tolerance = grain_n / 3; // wider search = better matches
    let window = hann_window(grain_n);

    // Correlation window: use a meaningful chunk of the grain for matching.
    // Too small = poor matches. Too large = slow. 1024 is ~23ms at 44.1kHz.
    let corr_len = grain_n.min(1024);

    let output_len = (samples.len() as f32 / rate) as usize;
    let mut output = vec![0.0f32; output_len];
    let mut norm = vec![0.0f32; output_len];

    let mut in_pos = 0usize;
    let mut out_pos = 0usize;

    while out_pos + grain_n < output_len && in_pos + grain_n < samples.len() {
        // Find best alignment using cross-correlation
        let search_start = in_pos.saturating_sub(tolerance);
        let search_end = (in_pos + tolerance).min(samples.len().saturating_sub(grain_n));

        let mut best_offset = in_pos;
        let mut best_corr = f32::MIN;

        // Coarse search (step 4)
        let mut pos = search_start;
        while pos <= search_end {
            let corr = cross_correlate(samples, pos, &output, out_pos, corr_len);
            if corr > best_corr {
                best_corr = corr;
                best_offset = pos;
            }
            pos += 4;
        }

        // Fine search around best coarse result
        let fine_start = best_offset.saturating_sub(4);
        let fine_end = (best_offset + 4).min(search_end);
        for pos in fine_start..=fine_end {
            let corr = cross_correlate(samples, pos, &output, out_pos, corr_len);
            if corr > best_corr {
                best_corr = corr;
                best_offset = pos;
            }
        }

        // Overlap-add with Hann window
        for i in 0..grain_n {
            if best_offset + i < samples.len() && out_pos + i < output_len {
                output[out_pos + i] += samples[best_offset + i] * window[i];
                norm[out_pos + i] += window[i] * window[i];
            }
        }

        in_pos += hop_in;
        out_pos += hop_out;
    }

    // Normalize by window overlap
    for i in 0..output_len {
        if norm[i] > 1e-8 {
            output[i] /= norm[i];
        }
    }

    output
}

fn cross_correlate(a: &[f32], a_off: usize, b: &[f32], b_off: usize, len: usize) -> f32 {
    let mut sum = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..len {
        let va = a.get(a_off + i).copied().unwrap_or(0.0);
        let vb = b.get(b_off + i).copied().unwrap_or(0.0);
        sum += va * vb;
        norm_a += va * va;
        norm_b += vb * vb;
    }
    // Normalized cross-correlation (cosine similarity)
    let denom = (norm_a * norm_b).sqrt();
    if denom > 1e-8 { sum / denom } else { 0.0 }
}
