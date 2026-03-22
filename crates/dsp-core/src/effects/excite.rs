/// Harmonic exciter — isolates highs with a Butterworth highpass, saturates them,
/// and mixes the harmonics back into the dry signal.
///
/// Uses the `biquad` crate for the highpass filter.

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type, Q_BUTTERWORTH_F32};

pub fn process_excite(
    samples: &[f32],
    sample_rate: u32,
    freq: f32,
    amount: f32,
    tone: f32,
) -> Vec<f32> {
    let sr = sample_rate as f32;

    // 2nd-order Butterworth highpass via biquad crate
    let coeffs = match Coefficients::<f32>::from_params(
        Type::HighPass,
        sr.hz(),
        freq.hz(),
        Q_BUTTERWORTH_F32,
    ) {
        Ok(c) => c,
        Err(_) => return samples.to_vec(),
    };
    let mut hp = DirectForm2Transposed::<f32>::new(coeffs);

    // Saturation gain derived from tone parameter
    let gain = 1.0 + tone * 3.0;

    samples
        .iter()
        .map(|&s| {
            // Highpass filter
            let filtered = hp.run(s);

            // Mix: out = dry + tanh(hp * g) * amount
            let excited = (filtered * gain).tanh() * amount;
            s + excited
        })
        .collect()
}
