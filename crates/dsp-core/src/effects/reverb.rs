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

    let mut output = Vec::with_capacity(samples.len());
    for &s in samples {
        // Feed mono to both channels of stereo reverb
        let out = reverb_node.tick(&Frame::from([s, s]));
        // Average both channels back to mono
        let wet = (out[0] + out[1]) * 0.5;
        output.push(s * (1.0 - mix) + wet * mix);
    }

    output
}
