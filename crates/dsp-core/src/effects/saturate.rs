/// Tape saturation — asymmetric soft-clipping with warmth-dependent treble rolloff.

pub fn process_saturate(samples: &[f32], drive_db: f32, warmth: f32) -> Vec<f32> {
    // Convert drive_db to linear gain
    let drive = 10.0f32.powf(drive_db / 20.0);

    // One-pole LPF for treble rolloff — cutoff decreases with warmth
    let fc = 20000.0 * (1.0 - warmth * 0.7);
    let rc = 1.0 / (2.0 * std::f32::consts::PI * fc);
    let dt = 1.0 / 44100.0; // approximate; good enough for one-pole
    let alpha = dt / (rc + dt);

    // Measure input peak for auto-normalization
    let input_peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

    let mut prev = 0.0f32;
    let mut output: Vec<f32> = samples
        .iter()
        .map(|&s| {
            let driven = s * drive;

            // Asymmetric soft clip
            let clipped = if driven >= 0.0 {
                driven.tanh() + 0.05 * (2.0 * driven).tanh()
            } else {
                driven.tanh() + 0.03 * (2.0 * driven).tanh()
            };

            // One-pole LP filter
            let filtered = prev + alpha * (clipped - prev);
            prev = filtered;
            filtered
        })
        .collect();

    // Auto-normalize output to match input peak level
    let output_peak = output.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if output_peak > 1e-8 && input_peak > 1e-8 {
        let ratio = input_peak / output_peak;
        for s in output.iter_mut() {
            *s *= ratio;
        }
    }

    output
}
