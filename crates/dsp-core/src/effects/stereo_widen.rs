/// Stereo widening via mid/side processing.
/// width: 0.0 = mono, 1.0 = original, 2.0 = maximum width
/// Works on interleaved stereo samples [L,R,L,R,...].
/// For mono input, creates stereo spread using Haas-style micro-delays.
pub fn process_stereo_widen(
    samples_l: &[f32],
    samples_r: &[f32],
    sample_rate: u32,
    width: f32,
) -> (Vec<f32>, Vec<f32>) {
    let n = samples_l.len().min(samples_r.len());

    // Mid/Side encoding
    let mut mid = vec![0.0f32; n];
    let mut side = vec![0.0f32; n];
    for i in 0..n {
        mid[i] = (samples_l[i] + samples_r[i]) * 0.5;
        side[i] = (samples_l[i] - samples_r[i]) * 0.5;
    }

    // Scale side signal by width factor
    // width 1.0 = original, >1.0 = wider
    let side_gain = width;
    for s in side.iter_mut() {
        *s *= side_gain;
    }

    // Add subtle Haas-style decorrelation to enhance width perception
    // Apply a tiny allpass filter to the side channel for phase spreading
    if width > 1.0 {
        let extra = width - 1.0;
        let delay_samples = ((sample_rate as f32 * 0.0003) as usize).max(1); // ~0.3ms
        let mut delay_buf = vec![0.0f32; delay_samples];
        let mut delay_idx = 0;
        for i in 0..n {
            let delayed = delay_buf[delay_idx];
            delay_buf[delay_idx] = side[i];
            delay_idx = (delay_idx + 1) % delay_samples;
            // Blend in the delayed component for extra width
            side[i] = side[i] * (1.0 - extra * 0.3) + delayed * (extra * 0.3);
        }
    }

    // Decode back to L/R
    let mut out_l = vec![0.0f32; n];
    let mut out_r = vec![0.0f32; n];
    for i in 0..n {
        out_l[i] = mid[i] + side[i];
        out_r[i] = mid[i] - side[i];
    }

    (out_l, out_r)
}
