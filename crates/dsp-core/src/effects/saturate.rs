/// Tape saturation — asymmetric soft-clipping with warmth-dependent treble rolloff.
///
/// Uses the `biquad` crate for the lowpass filter (fixes hardcoded 44100 Hz).
/// Uses `rubato` for 2x oversampling to reduce aliasing from nonlinear processing.

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type, Q_BUTTERWORTH_F32};
use rubato::{FftFixedIn, Resampler};

pub fn process_saturate(samples: &[f32], sample_rate: u32, drive_db: f32, warmth: f32) -> Vec<f32> {
    let sr = sample_rate as usize;
    let oversampled_sr = sr * 2;

    // --- 2x Oversample via Rubato ---
    let upsampled = upsample_2x(samples, sr);

    // Convert drive_db to linear gain
    let drive = 10.0f32.powf(drive_db / 20.0);

    // Lowpass filter for treble rolloff — cutoff decreases with warmth
    // Now sample-rate aware via biquad crate
    let fc = 20000.0 * (1.0 - warmth * 0.7);
    let lpf_coeffs = Coefficients::<f32>::from_params(
        Type::LowPass,
        (oversampled_sr as f32).hz(),
        fc.hz(),
        Q_BUTTERWORTH_F32,
    );
    let mut lpf = lpf_coeffs.ok().map(|c| DirectForm2Transposed::<f32>::new(c));

    // Process at 2x sample rate (reduces aliasing from tanh nonlinearity)
    let processed: Vec<f32> = upsampled
        .iter()
        .map(|&s| {
            let driven = s * drive;

            // Asymmetric soft clip
            let clipped = if driven >= 0.0 {
                driven.tanh() + 0.05 * (2.0 * driven).tanh()
            } else {
                driven.tanh() + 0.03 * (2.0 * driven).tanh()
            };

            // Apply lowpass if coefficients were valid
            match lpf.as_mut() {
                Some(f) => f.run(clipped),
                None => clipped,
            }
        })
        .collect();

    // --- Downsample back to original rate ---
    downsample_2x(&processed, sr)
}

/// 2x upsample using Rubato's FFT-based resampler
fn upsample_2x(samples: &[f32], sr: usize) -> Vec<f32> {
    let chunk_size = samples.len();
    if chunk_size == 0 {
        return vec![];
    }

    let mut resampler = match FftFixedIn::<f32>::new(sr, sr * 2, chunk_size, 1, 1) {
        Ok(r) => r,
        Err(_) => {
            // Fallback: simple linear interpolation if Rubato fails
            return linear_upsample_2x(samples);
        }
    };

    let input = vec![samples.to_vec()];
    match resampler.process(&input, None) {
        Ok(output) => output.into_iter().next().unwrap_or_else(|| linear_upsample_2x(samples)),
        Err(_) => linear_upsample_2x(samples),
    }
}

/// 2x downsample using Rubato's FFT-based resampler
fn downsample_2x(samples: &[f32], target_sr: usize) -> Vec<f32> {
    let chunk_size = samples.len();
    if chunk_size == 0 {
        return vec![];
    }

    let mut resampler = match FftFixedIn::<f32>::new(target_sr * 2, target_sr, chunk_size, 1, 1) {
        Ok(r) => r,
        Err(_) => {
            return linear_downsample_2x(samples);
        }
    };

    let input = vec![samples.to_vec()];
    match resampler.process(&input, None) {
        Ok(output) => output.into_iter().next().unwrap_or_else(|| linear_downsample_2x(samples)),
        Err(_) => linear_downsample_2x(samples),
    }
}

/// Fallback: simple linear interpolation upsample
fn linear_upsample_2x(samples: &[f32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for i in 0..samples.len() {
        out.push(samples[i]);
        let next = if i + 1 < samples.len() { samples[i + 1] } else { samples[i] };
        out.push((samples[i] + next) * 0.5);
    }
    out
}

/// Fallback: simple decimation downsample
fn linear_downsample_2x(samples: &[f32]) -> Vec<f32> {
    samples.chunks(2).map(|c| {
        if c.len() == 2 { (c[0] + c[1]) * 0.5 } else { c[0] }
    }).collect()
}
