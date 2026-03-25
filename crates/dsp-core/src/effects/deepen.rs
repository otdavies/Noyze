/// Deepen — low-shelf bass boost for a deeper, weightier sound.
/// Uses a cascaded biquad low-shelf filter for smooth bass enhancement.
/// amount: boost intensity (0.0-1.0, maps to 0-12 dB shelf gain)
/// freq: shelf frequency in Hz (40-500)

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type};

pub fn process_deepen(
    samples: &[f32],
    sample_rate: u32,
    amount: f32,
    freq: f32,
) -> Vec<f32> {
    let sr = sample_rate as f32;
    let gain_db = amount * 12.0;

    if gain_db < 0.1 {
        return samples.to_vec();
    }

    // Convert dB gain to linear for the shelf filter
    // Q of ~0.7 gives a smooth, musical shelf curve
    let shelf_q = 0.707;

    let coeffs = match Coefficients::<f32>::from_params(
        Type::LowShelf(gain_db),
        sr.hz(),
        freq.hz(),
        shelf_q,
    ) {
        Ok(c) => c,
        Err(_) => return samples.to_vec(),
    };

    let mut filter = DirectForm2Transposed::<f32>::new(coeffs);

    samples.iter().map(|&s| filter.run(s)).collect()
}
