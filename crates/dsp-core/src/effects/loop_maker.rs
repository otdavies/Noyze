/// Seamless loop maker — cross-correlates end region against start to find the
/// best splice point, snaps to zero crossing, and applies equal-power crossfade
/// with micro fade-in/out at the edges.

use std::f32::consts::PI;

pub fn process_loop(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    let fade_samples = (sample_rate as f32 * 2.0) as usize; // 2-second search region
    if samples.len() < fade_samples * 2 {
        return samples.to_vec();
    }

    let end_region_start = samples.len() - fade_samples;
    let start_region = &samples[..fade_samples];
    let end_region = &samples[end_region_start..];

    // --- Two-pass cross-correlation search ---

    let mut best_offset = 0usize;
    let mut best_corr = f32::MIN;

    // Coarse search
    let coarse_step = (fade_samples / 200).max(1);
    let mut off = 0;
    while off + 1024 <= fade_samples {
        let corr = correlate(&end_region[off..], start_region, 1024.min(fade_samples - off));
        if corr > best_corr {
            best_corr = corr;
            best_offset = off;
        }
        off += coarse_step;
    }

    // Fine search around best coarse result
    let fine_start = best_offset.saturating_sub(coarse_step);
    let fine_end = (best_offset + coarse_step).min(fade_samples.saturating_sub(64));
    for off in fine_start..=fine_end {
        if off + 512 <= fade_samples {
            let corr = correlate(&end_region[off..], start_region, 512);
            if corr > best_corr {
                best_corr = corr;
                best_offset = off;
            }
        }
    }

    // Snap to zero crossing
    let search_r = (sample_rate as f32 * 0.005) as usize;
    let snap_pos = crate::fft_utils::find_zero_crossing(end_region, best_offset, search_r);

    // Trim output up to the loop point
    let loop_end = end_region_start + snap_pos;
    let crossfade_len = snap_pos.min(fade_samples / 2).min(loop_end);

    let mut output = samples[..loop_end].to_vec();

    // Equal-power crossfade (cos/sin)
    for i in 0..crossfade_len {
        let t = i as f32 / crossfade_len as f32;
        let fade_out = (PI * 0.5 * (1.0 - t)).cos();
        let fade_in = (PI * 0.5 * t).sin();
        let end_idx = output.len() - crossfade_len + i;
        let start_sample = samples[i];
        output[end_idx] = output[end_idx] * fade_out + start_sample * fade_in;
    }

    // Micro fade-in/out (64 samples) at edges
    let micro = 64.min(output.len() / 2);
    let out_len = output.len();
    for i in 0..micro {
        let t = i as f32 / micro as f32;
        output[i] *= t;
        output[out_len - 1 - i] *= t;
    }

    output
}

fn correlate(a: &[f32], b: &[f32], len: usize) -> f32 {
    let n = len.min(a.len()).min(b.len());
    let mut sum = 0.0f32;
    for i in 0..n {
        sum += a[i] * b[i];
    }
    sum
}
