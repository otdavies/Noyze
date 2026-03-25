/// Psychoacoustic sub-bass enhancement.
/// Generates a sub-octave harmonic below the fundamental bass frequency,
/// adding depth and warmth without muddiness.
///
/// Uses the `biquad` crate for lowpass filter stages.

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type, Q_BUTTERWORTH_F32};

pub fn process_sub_bass(
    samples: &[f32],
    sample_rate: u32,
    amount: f32,
    freq: f32,
) -> Vec<f32> {
    let sr = sample_rate as f32;
    let n = samples.len();

    // Extract bass content using a 2nd-order Butterworth lowpass
    let bass_coeffs = match Coefficients::<f32>::from_params(
        Type::LowPass,
        sr.hz(),
        freq.hz(),
        Q_BUTTERWORTH_F32,
    ) {
        Ok(c) => c,
        Err(_) => return samples.to_vec(),
    };
    let mut bass_filter = DirectForm2Transposed::<f32>::new(bass_coeffs);

    let mut bass = Vec::with_capacity(n);
    for &s in samples {
        bass.push(bass_filter.run(s));
    }

    // Generate sub-octave via half-wave rectification + filtering
    let mut last_sign = false;
    let mut toggle = false;
    let mut sub_raw = Vec::with_capacity(n);

    for i in 0..n {
        let positive = bass[i] >= 0.0;
        if positive != last_sign {
            toggle = !toggle;
            last_sign = positive;
        }
        sub_raw.push(if toggle { bass[i].abs() } else { -bass[i].abs() });
    }

    // Smooth the sub signal with a lowpass at half the bass frequency
    let sub_freq = (freq * 0.5).max(20.0);
    let sub_coeffs = match Coefficients::<f32>::from_params(
        Type::LowPass,
        sr.hz(),
        sub_freq.hz(),
        Q_BUTTERWORTH_F32,
    ) {
        Ok(c) => c,
        Err(_) => return samples.to_vec(),
    };
    let mut sub_filter = DirectForm2Transposed::<f32>::new(sub_coeffs);

    let mut sub_smooth = Vec::with_capacity(n);
    for &s in &sub_raw {
        sub_smooth.push(sub_filter.run(s));
    }

    // Mix: original + sub harmonics
    let sub_gain = amount * 0.5;
    let out: Vec<f32> = samples.iter().enumerate()
        .map(|(i, &s)| s + sub_smooth[i] * sub_gain)
        .collect();

    out
}
