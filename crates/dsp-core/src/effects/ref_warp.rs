/// Reference audio morphing — cross-correlates source blocks against a reference
/// signal, blending the best-matching section back into the source.

pub fn process_ref_warp(
    samples: &[f32],
    reference: &[f32],
    sample_rate: u32,
    amount: f32,
) -> Vec<f32> {
    if reference.is_empty() || amount <= 0.0 {
        return samples.to_vec();
    }

    let amount = amount.clamp(0.0, 1.0);
    let block_size = (sample_rate as f32 * 0.5) as usize; // 500ms blocks
    let crossfade_len = (sample_rate as f32 * 0.02) as usize; // 20ms crossfade
    let mut output = Vec::with_capacity(samples.len());

    let mut pos = 0;
    while pos < samples.len() {
        let end = (pos + block_size).min(samples.len());
        let block_len = end - pos;

        // Use up to 2048 samples for cross-correlation search
        let search_len = block_len.min(2048);

        let mut best_offset = 0usize;
        let mut best_corr = f32::MIN;

        // Coarse search — stride through reference at ~200 evenly spaced points
        let coarse_step = (reference.len() / 200).max(1);
        let mut off = 0;
        while off + block_len <= reference.len() {
            let corr = correlate(
                &samples[pos..pos + search_len.min(block_len)],
                &reference[off..off + search_len.min(reference.len() - off)],
            );
            if corr > best_corr {
                best_corr = corr;
                best_offset = off;
            }
            off += coarse_step;
        }

        // Fine search — sample-accurate around best coarse hit
        let fine_start = best_offset.saturating_sub(coarse_step);
        let fine_end =
            (best_offset + coarse_step).min(reference.len().saturating_sub(block_len));
        for off in fine_start..=fine_end {
            if off + search_len.min(block_len) <= reference.len() {
                let corr = correlate(
                    &samples[pos..pos + search_len.min(block_len)],
                    &reference[off..off + search_len.min(reference.len() - off)],
                );
                if corr > best_corr {
                    best_corr = corr;
                    best_offset = off;
                }
            }
        }

        // Blend: out = source * (1 - amount) + ref_match * amount
        for i in 0..block_len {
            let ref_sample = if best_offset + i < reference.len() {
                reference[best_offset + i]
            } else {
                0.0
            };
            output.push(samples[pos + i] * (1.0 - amount) + ref_sample * amount);
        }

        pos = end;
    }

    // 20ms crossfades at block boundaries
    let mut boundary = block_size;
    while boundary + crossfade_len < output.len() {
        for j in 0..crossfade_len {
            let t = j as f32 / crossfade_len as f32;
            if boundary + j > 0 && boundary + j < output.len() {
                let prev = output[boundary + j - 1];
                let curr = output[boundary + j];
                output[boundary + j] = prev * (1.0 - t) + curr * t;
            }
        }
        boundary += block_size;
    }

    output
}

fn correlate(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    let mut sum = 0.0f32;
    for i in 0..len {
        sum += a[i] * b[i];
    }
    sum
}
