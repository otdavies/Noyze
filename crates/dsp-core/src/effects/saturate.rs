/// Tape saturation — musical soft-clipping with warmth control.
///
/// Uses 2x oversampling to reduce aliasing from the nonlinear waveshaper.
/// Drive controls how hard the signal is pushed into the saturator.
/// Warmth applies a gentle high-shelf cut for analog character.

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type, Q_BUTTERWORTH_F32};

pub fn process_saturate(samples: &[f32], sample_rate: u32, drive_db: f32, warmth: f32) -> Vec<f32> {
    let sr = sample_rate as usize;
    let oversampled_sr = (sr * 2) as f32;

    // 2x upsample
    let upsampled = linear_upsample_2x(samples);

    // Gentle drive curve: 0dB → 1x, 6dB → ~1.4x, 12dB → ~2x, 24dB → ~4x
    // Using sqrt scaling so high drive values don't obliterate the signal
    let drive_linear = 10.0f32.powf(drive_db / 40.0); // /40 instead of /20 = sqrt of normal

    // Wet/dry mix scales with drive: low drive = mostly dry, high drive = mostly wet
    let mix = ((drive_db - 1.0) / 23.0).clamp(0.0, 1.0) * 0.8 + 0.2;

    // Anti-alias lowpass at Nyquist before downsampling
    let aa_coeffs = Coefficients::<f32>::from_params(
        Type::LowPass,
        oversampled_sr.hz(),
        (sr as f32 * 0.95).hz(), // just below original Nyquist
        Q_BUTTERWORTH_F32,
    );
    let mut aa_filter = aa_coeffs.ok().map(|c| DirectForm2Transposed::<f32>::new(c));

    // Warmth: gentle high-shelf cut. At warmth=1.0, -6dB above 4kHz.
    // Much gentler than a lowpass — preserves presence and air.
    let warmth_coeffs = if warmth > 0.01 {
        let shelf_db = -(warmth * 6.0);
        Coefficients::<f32>::from_params(
            Type::HighShelf(shelf_db),
            oversampled_sr.hz(),
            4000.0f32.hz(),
            0.707,
        ).ok()
    } else {
        None
    };
    let mut warmth_filter = warmth_coeffs.map(|c| DirectForm2Transposed::<f32>::new(c));

    // Process
    let processed: Vec<f32> = upsampled
        .iter()
        .map(|&s| {
            let dry = s;
            let driven = s * drive_linear;

            // Simple tanh soft-clip — clean, musical, no weird harmonics
            let wet = driven.tanh() / drive_linear.min(2.0).max(1.0);
            // The division by min(drive,2) compensates for tanh's gain reduction
            // at high drive, keeping perceived loudness closer to the original

            // Blend wet/dry
            let mut out = dry * (1.0 - mix) + wet * mix;

            // Apply warmth filter
            if let Some(ref mut f) = warmth_filter {
                out = f.run(out);
            }

            // Anti-alias filter before downsampling
            if let Some(ref mut f) = aa_filter {
                out = f.run(out);
            }

            out
        })
        .collect();

    // Downsample
    linear_downsample_2x(&processed)
}

/// Zero-latency linear interpolation upsample
fn linear_upsample_2x(samples: &[f32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for i in 0..samples.len() {
        out.push(samples[i]);
        let next = if i + 1 < samples.len() { samples[i + 1] } else { samples[i] };
        out.push((samples[i] + next) * 0.5);
    }
    out
}

/// Simple decimation downsample
fn linear_downsample_2x(samples: &[f32]) -> Vec<f32> {
    samples.chunks(2).map(|c| {
        if c.len() == 2 { (c[0] + c[1]) * 0.5 } else { c[0] }
    }).collect()
}
