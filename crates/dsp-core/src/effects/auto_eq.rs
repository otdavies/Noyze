/// Auto EQ — 5-band parametric equalizer using cascaded biquad filters.
/// Bands: sub(0-80Hz), low(80-300Hz), mid(300-2kHz), presence(2k-8kHz), air(8k-20kHz)
///
/// Uses the `biquad` crate for optimized filter coefficient calculation
/// and Direct Form 2 Transposed processing.

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type};

pub fn process_auto_eq(
    samples: &[f32],
    sample_rate: u32,
    preset: &str,
    intensity: f32,
) -> Vec<f32> {
    let sr = sample_rate as f32;

    // Preset gains in dB, scaled by intensity
    let gains_db: [f32; 5] = match preset.to_lowercase().as_str() {
        "warm"   => [0.0, 2.0, -1.0, 0.0, 0.5],
        "bright" => [-1.0, -0.5, 0.0, 1.5, 2.0],
        "full"   => [0.0, 1.5, -2.0, 1.0, 1.5],
        "dark"   => [0.0, 2.0, 0.0, -1.0, -2.0],
        _        => [-2.0, 0.0, -1.5, 0.0, 1.5], // "clean" default
    };

    // Scale by intensity and clamp
    let g: Vec<f32> = gains_db.iter()
        .map(|&db| (db * intensity).clamp(-6.0, 6.0))
        .collect();

    // Build 5-band EQ: low shelf + 3 peaking + high shelf
    let filters: Vec<Option<DirectForm2Transposed<f32>>> = vec![
        make_filter(Type::LowShelf(g[0]), sr, 80.0, 0.707),
        make_filter(Type::PeakingEQ(g[1]), sr, 150.0, 0.8),
        make_filter(Type::PeakingEQ(g[2]), sr, 800.0, 0.8),
        make_filter(Type::PeakingEQ(g[3]), sr, 4000.0, 0.8),
        make_filter(Type::HighShelf(g[4]), sr, 8000.0, 0.707),
    ];

    let mut active: Vec<DirectForm2Transposed<f32>> = filters.into_iter().flatten().collect();

    // Process all samples through the filter cascade
    let mut output = Vec::with_capacity(samples.len());
    for &s in samples {
        let mut x = s;
        for f in active.iter_mut() {
            x = f.run(x);
        }
        output.push(x);
    }

    output
}

fn make_filter(
    filter_type: Type<f32>,
    sample_rate: f32,
    freq: f32,
    q: f32,
) -> Option<DirectForm2Transposed<f32>> {
    Coefficients::<f32>::from_params(filter_type, sample_rate.hz(), freq.hz(), q)
        .ok()
        .map(|c| DirectForm2Transposed::<f32>::new(c))
}
