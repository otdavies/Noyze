/// Psychoacoustic sub-bass enhancement.
/// Generates a sub-octave harmonic below the fundamental bass frequency,
/// adding depth and warmth without muddiness.
/// amount: 0.0-1.0, how much sub content to add
/// freq: cutoff frequency below which we generate sub content (default ~100 Hz)
pub fn process_sub_bass(
    samples: &[f32],
    sample_rate: u32,
    amount: f32,
    freq: f32,
) -> Vec<f32> {
    let sr = sample_rate as f32;
    let n = samples.len();
    let mut out = vec![0.0f32; n];

    // Extract bass content using a 2nd-order Butterworth lowpass
    let w0 = 2.0 * std::f32::consts::PI * freq / sr;
    let cos_w = w0.cos();
    let sin_w = w0.sin();
    let alpha = sin_w / (2.0 * 0.707);

    let b0 = (1.0 - cos_w) / 2.0 / (1.0 + alpha);
    let b1 = (1.0 - cos_w) / (1.0 + alpha);
    let b2 = b0;
    let a1 = -2.0 * cos_w / (1.0 + alpha);
    let a2 = (1.0 - alpha) / (1.0 + alpha);

    let mut x1 = 0.0f32;
    let mut x2 = 0.0f32;
    let mut y1 = 0.0f32;
    let mut y2 = 0.0f32;
    let mut bass = vec![0.0f32; n];

    for i in 0..n {
        let x = samples[i];
        let y = b0 * x + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
        x2 = x1; x1 = x;
        y2 = y1; y1 = y;
        bass[i] = y;
    }

    // Generate sub-octave via half-wave rectification + filtering
    // This classic technique produces a signal one octave below the input
    // by tracking zero crossings and generating a square wave at half frequency
    let mut last_sign = false;
    let mut toggle = false;
    let mut sub_raw = vec![0.0f32; n];

    for i in 0..n {
        let positive = bass[i] >= 0.0;
        if positive != last_sign {
            toggle = !toggle;
            last_sign = positive;
        }
        sub_raw[i] = if toggle { bass[i].abs() } else { -bass[i].abs() };
    }

    // Smooth the sub signal with a lowpass at half the bass frequency
    let sub_freq = (freq * 0.5).max(20.0);
    let w0s = 2.0 * std::f32::consts::PI * sub_freq / sr;
    let cos_ws = w0s.cos();
    let sin_ws = w0s.sin();
    let alphas = sin_ws / (2.0 * 0.707);

    let sb0 = (1.0 - cos_ws) / 2.0 / (1.0 + alphas);
    let sb1 = (1.0 - cos_ws) / (1.0 + alphas);
    let sb2 = sb0;
    let sa1 = -2.0 * cos_ws / (1.0 + alphas);
    let sa2 = (1.0 - alphas) / (1.0 + alphas);

    let mut sx1 = 0.0f32;
    let mut sx2 = 0.0f32;
    let mut sy1 = 0.0f32;
    let mut sy2 = 0.0f32;
    let mut sub_smooth = vec![0.0f32; n];

    for i in 0..n {
        let x = sub_raw[i];
        let y = sb0 * x + sb1 * sx1 + sb2 * sx2 - sa1 * sy1 - sa2 * sy2;
        sx2 = sx1; sx1 = x;
        sy2 = sy1; sy1 = y;
        sub_smooth[i] = y;
    }

    // Mix: original + sub harmonics
    let sub_gain = amount * 0.5; // Keep subtle
    for i in 0..n {
        out[i] = samples[i] + sub_smooth[i] * sub_gain;
    }

    // Auto-normalize to prevent clipping
    let in_peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let out_peak = out.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if out_peak > 1e-8 && in_peak > 1e-8 {
        let ratio = in_peak / out_peak;
        for s in out.iter_mut() {
            *s *= ratio;
        }
    }

    out
}
