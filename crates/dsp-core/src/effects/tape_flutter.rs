/// Tape flutter / wow simulation.
/// Applies subtle pitch and timing modulation that mimics analog tape machines.
/// This adds pleasing organic character and "warmth" without ever degrading quality.
/// rate: modulation rate in Hz (0.5-6.0, lower = wow, higher = flutter)
/// depth: modulation depth (0.0-1.0)
/// mix: wet/dry mix (0.0-1.0)
pub fn process_tape_flutter(
    samples: &[f32],
    sample_rate: u32,
    rate: f32,
    depth: f32,
    mix: f32,
) -> Vec<f32> {
    let sr = sample_rate as f32;
    let n = samples.len();
    let mut out = vec![0.0f32; n];

    // Maximum delay modulation in samples (~1.5ms at full depth)
    let max_delay = (sr * 0.0015 * depth) as f32;

    // Need a buffer for fractional delay interpolation
    // Use a fixed delay buffer of max_delay * 2 + some extra
    let buf_size = (max_delay as usize + 4).max(8);
    let mut delay_buf = vec![0.0f32; buf_size * 2];
    let mut write_pos = 0usize;

    // Use multiple LFO rates for natural-sounding flutter
    // Real tape has both wow (slow, ~0.5-2Hz) and flutter (fast, ~4-10Hz)
    let lfo1_rate = rate;
    let lfo2_rate = rate * 2.37; // Irrational ratio for less repetition
    let lfo3_rate = rate * 0.41; // Slow wow component

    let lfo1_depth = max_delay * 0.5;
    let lfo2_depth = max_delay * 0.3;
    let lfo3_depth = max_delay * 0.2;

    let base_delay = max_delay + 2.0; // Ensure we always read from valid positions

    for i in 0..n {
        let t = i as f32 / sr;

        // Multi-rate LFO for natural flutter
        let lfo = lfo1_depth * (2.0 * std::f32::consts::PI * lfo1_rate * t).sin()
                + lfo2_depth * (2.0 * std::f32::consts::PI * lfo2_rate * t + 0.3).sin()
                + lfo3_depth * (2.0 * std::f32::consts::PI * lfo3_rate * t + 1.7).sin();

        let delay = base_delay + lfo;

        // Write to circular buffer
        let buf_len = delay_buf.len();
        delay_buf[write_pos % buf_len] = samples[i];

        // Read with fractional delay using cubic interpolation
        let read_pos_f = write_pos as f32 - delay;
        let read_pos_i = read_pos_f.floor() as isize;
        let frac = read_pos_f - read_pos_f.floor();

        let bl = buf_len as isize;
        let get = |offset: isize| -> f32 {
            let p = ((read_pos_i + offset) % bl + bl) % bl;
            delay_buf[p as usize]
        };

        // Cubic Hermite interpolation for smooth modulated delay
        let y0 = get(-1);
        let y1 = get(0);
        let y2 = get(1);
        let y3 = get(2);

        let c0 = y1;
        let c1 = 0.5 * (y2 - y0);
        let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
        let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);

        let wet = ((c3 * frac + c2) * frac + c1) * frac + c0;

        // Mix dry and wet
        out[i] = samples[i] * (1.0 - mix) + wet * mix;

        write_pos += 1;
    }

    // Subtle soft saturation for tape warmth (very light)
    if depth > 0.3 {
        let sat_amount = (depth - 0.3) * 0.15;
        for s in out.iter_mut() {
            let driven = *s * (1.0 + sat_amount);
            *s = driven.tanh() / (1.0 + sat_amount).tanh();
        }
    }

    // Auto-normalize
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
