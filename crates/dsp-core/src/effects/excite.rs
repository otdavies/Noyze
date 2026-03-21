/// Harmonic exciter — isolates highs with a Butterworth highpass, saturates them,
/// and mixes the harmonics back into the dry signal.

use std::f32::consts::PI;

pub fn process_excite(
    samples: &[f32],
    sample_rate: u32,
    freq: f32,
    amount: f32,
    tone: f32,
) -> Vec<f32> {
    // 2nd-order Butterworth highpass coefficients
    let omega = 2.0 * PI * freq / sample_rate as f32;
    let cos_w = omega.cos();
    let sin_w = omega.sin();
    let alpha = sin_w / (2.0 * 2.0f32.sqrt()); // Q = sqrt(2) for Butterworth

    let a0 = 1.0 + alpha;
    let b0 = ((1.0 + cos_w) / 2.0) / a0;
    let b1 = (-(1.0 + cos_w)) / a0;
    let b2 = ((1.0 + cos_w) / 2.0) / a0;
    let a1 = (-2.0 * cos_w) / a0;
    let a2 = (1.0 - alpha) / a0;

    // Saturation gain derived from tone parameter
    let gain = 1.0 + tone * 3.0;

    let mut x1 = 0.0f32;
    let mut x2 = 0.0f32;
    let mut y1 = 0.0f32;
    let mut y2 = 0.0f32;

    samples
        .iter()
        .map(|&s| {
            // Highpass filter
            let hp = b0 * s + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
            x2 = x1;
            x1 = s;
            y2 = y1;
            y1 = hp;

            // Mix: out = dry + tanh(hp * g) * amount
            let excited = (hp * gain).tanh() * amount;
            s + excited
        })
        .collect()
}
