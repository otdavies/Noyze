/// High-quality reverb using FunDSP's feedback delay network.
///
/// Replaces the manual Schroeder reverb (8 comb + 4 allpass) with FunDSP's
/// 32-channel FDN reverb, which produces denser, more natural-sounding tails.

use fundsp::hacker32::*;

pub fn process_reverb(samples: &[f32], sample_rate: u32, size: f32, damping: f32, mix: f32) -> Vec<f32> {
    if samples.is_empty() || mix <= 0.0 {
        return samples.to_vec();
    }

    // Map our 0-1 params to FunDSP reverb ranges
    let room = (size * 30.0 + 10.0) as f32;   // 10-40m room size
    let time = (size * 4.0 + 0.5) as f32;     // 0.5-4.5s RT60
    let damp = damping;

    // Create FunDSP's FDN-based stereo reverb
    let mut reverb_node = Box::new(reverb_stereo(room, time, damp));
    reverb_node.set_sample_rate(sample_rate as f64);
    reverb_node.reset();

    // Tail = full RT60 so the reverb decays naturally before we cut
    let tail_samples = (time * sample_rate as f32) as usize;
    let total_len = samples.len() + tail_samples;
    let mut output = Vec::with_capacity(total_len);

    for i in 0..total_len {
        let dry = if i < samples.len() { samples[i] } else { 0.0 };
        let out = reverb_node.tick(&Frame::from([dry, dry]));
        let wet = (out[0] + out[1]) * 0.5;
        output.push(dry * (1.0 - mix) + wet * mix);
    }

    // Exponential fade over the last 60% of the tail for a natural decay envelope.
    // Real reverb energy decays exponentially, so a linear fade sounds abrupt.
    if tail_samples > 0 {
        let fade_len = (tail_samples as f32 * 0.6) as usize;
        let tail_start = output.len() - fade_len;
        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            // Exponential decay curve: fast initial drop, gentle final approach to zero
            let gain = (-4.0 * t).exp();
            output[tail_start + i] *= gain;
        }
    }

    // Trim trailing near-silence to avoid unnecessary length
    while output.len() > samples.len() {
        if output.last().map_or(false, |&s| s.abs() < 1e-6) {
            output.pop();
        } else {
            break;
        }
    }

    output
}
