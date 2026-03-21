struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
    damp1: f32,
    damp2: f32,
    filter_store: f32,
}

impl CombFilter {
    fn new(size: usize, feedback: f32, damp: f32) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
            feedback,
            damp1: damp,
            damp2: 1.0 - damp,
            filter_store: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.index];
        self.filter_store = output * self.damp2 + self.filter_store * self.damp1;
        self.buffer[self.index] = input + self.filter_store * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

struct AllpassFilter {
    buffer: Vec<f32>,
    index: usize,
}

impl AllpassFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buffered = self.buffer[self.index];
        let output = buffered - input;
        self.buffer[self.index] = input + buffered * 0.5;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

pub fn process_reverb(samples: &[f32], sample_rate: u32, size: f32, damping: f32, mix: f32) -> Vec<f32> {
    let sr = sample_rate;
    let scale = sr as f32 / 44100.0;
    let comb_delays = [1557, 1617, 1491, 1422, 1277, 1356, 1188, 1116];
    let allpass_delays = [225, 556, 441, 341];

    let feedback = size * 0.85 + 0.1;

    let mut combs: Vec<CombFilter> = comb_delays.iter()
        .map(|&d| CombFilter::new((d as f32 * scale) as usize, feedback, damping))
        .collect();

    let mut allpasses: Vec<AllpassFilter> = allpass_delays.iter()
        .map(|&d| AllpassFilter::new((d as f32 * scale) as usize))
        .collect();

    let mut output = Vec::with_capacity(samples.len());

    for &sample in samples {
        // Parallel comb filters
        let mut wet = 0.0f32;
        for comb in combs.iter_mut() {
            wet += comb.process(sample);
        }
        wet /= combs.len() as f32;

        // Series allpass filters
        for ap in allpasses.iter_mut() {
            wet = ap.process(wet);
        }

        output.push(sample * (1.0 - mix) + wet * mix);
    }

    output
}
