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

    // Add tail for reverb decay — use RT60 scaled by mix to capture meaningful tail
    let tail_samples = (time * sample_rate as f32 * mix) as usize;
    let total_len = samples.len() + tail_samples;
    let mut output = Vec::with_capacity(total_len);

    for i in 0..total_len {
        let dry = if i < samples.len() { samples[i] } else { 0.0 };
        let out = reverb_node.tick(&Frame::from([dry, dry]));
        let wet = (out[0] + out[1]) * 0.5;
        output.push(dry * (1.0 - mix) + wet * mix);
    }

    // Fade out the tail to avoid abrupt cutoff
    if tail_samples > 0 {
        let fade_len = std::cmp::min(tail_samples, sample_rate as usize / 4); // 250ms fade max
        let tail_start = output.len() - fade_len;
        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            output[tail_start + i] *= 1.0 - t;
        }
    }

    output
}
