/// SPL-style differential transient shaper — two parallel envelope followers
/// (fast / slow) whose difference drives level-independent transient / sustain gain.

pub fn process_transient(
    samples: &[f32],
    sample_rate: u32,
    attack: f32,
    sustain: f32,
) -> Vec<f32> {
    // Envelope follower coefficients
    let fast_attack = (-1.0 / (0.001 * sample_rate as f32)).exp(); // 1ms attack
    let slow_attack = (-1.0 / (0.015 * sample_rate as f32)).exp(); // 15ms attack
    let release = (-1.0 / (0.02 * sample_rate as f32)).exp(); // 20ms shared release

    let mut fast_env = 0.0f32;
    let mut slow_env = 0.0f32;

    samples
        .iter()
        .map(|&s| {
            let abs_s = s.abs();

            // Fast envelope follower (1ms attack)
            if abs_s > fast_env {
                fast_env = fast_attack * fast_env + (1.0 - fast_attack) * abs_s;
            } else {
                fast_env = release * fast_env + (1.0 - release) * abs_s;
            }

            // Slow envelope follower (15ms attack)
            if abs_s > slow_env {
                slow_env = slow_attack * slow_env + (1.0 - slow_attack) * abs_s;
            } else {
                slow_env = release * slow_env + (1.0 - release) * abs_s;
            }

            // Differential: positive during transients, negative during sustain
            let diff = fast_env - slow_env;

            // Gain as function of differential — level-independent (no threshold)
            let gain_db = if diff > 0.0 {
                diff * attack * 12.0
            } else {
                diff * sustain * 12.0
            };

            // Clamp to +/-12dB and convert to linear
            let gain = 10.0f32.powf(gain_db.clamp(-12.0, 12.0) / 20.0);
            s * gain
        })
        .collect()
}
