/// Auto EQ — 5-band parametric equalizer using cascaded biquad filters.
/// Bands: sub(0-80Hz), low(80-300Hz), mid(300-2kHz), presence(2k-8kHz), air(8k-20kHz)
///
/// Uses O(n) biquad filters instead of STFT — processes millions of samples
/// in milliseconds with zero heap allocation beyond the output buffer.

use std::f32::consts::PI;

struct Biquad {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl Biquad {
    /// Create a peaking EQ filter (constant-Q)
    fn peaking(center_freq: f32, sample_rate: f32, gain_db: f32, q: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0); // sqrt of linear gain
        let w0 = 2.0 * PI * center_freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let a0 = 1.0 + alpha / a;
        Biquad {
            b0: (1.0 + alpha * a) / a0,
            b1: (-2.0 * cos_w0) / a0,
            b2: (1.0 - alpha * a) / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha / a) / a0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    /// Create a low-shelf filter
    fn low_shelf(freq: f32, sample_rate: f32, gain_db: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * 0.707); // Q = 1/sqrt(2)
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
        Biquad {
            b0: (a * ((a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha)) / a0,
            b1: (2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0)) / a0,
            b2: (a * ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha)) / a0,
            a1: (-2.0 * ((a - 1.0) + (a + 1.0) * cos_w0)) / a0,
            a2: ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha) / a0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    /// Create a high-shelf filter
    fn high_shelf(freq: f32, sample_rate: f32, gain_db: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * 0.707);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
        Biquad {
            b0: (a * ((a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha)) / a0,
            b1: (-2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0)) / a0,
            b2: (a * ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha)) / a0,
            a1: (2.0 * ((a - 1.0) - (a + 1.0) * cos_w0)) / a0,
            a2: ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha) / a0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    #[inline(always)]
    fn process_sample(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
              - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

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
    // Band centers: sub=40Hz, low=150Hz, mid=800Hz, presence=4kHz, air=12kHz
    let mut filters = [
        Biquad::low_shelf(80.0, sr, g[0]),                  // Sub band
        Biquad::peaking(150.0, sr, g[1], 0.8),              // Low band
        Biquad::peaking(800.0, sr, g[2], 0.8),              // Mid band
        Biquad::peaking(4000.0, sr, g[3], 0.8),             // Presence band
        Biquad::high_shelf(8000.0, sr, g[4]),                // Air band
    ];

    // Process all samples through the filter cascade — O(n), zero allocation
    let mut output = Vec::with_capacity(samples.len());
    for &s in samples {
        let mut x = s;
        for f in filters.iter_mut() {
            x = f.process_sample(x);
        }
        output.push(x);
    }

    output
}
