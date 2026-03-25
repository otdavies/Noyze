use rustfft::num_complex::Complex32;
use rustfft::FftPlanner;
use std::cell::RefCell;
use std::f32::consts::PI;

// Thread-local cached planner (WASM is single-threaded)
thread_local! {
    static PLANNER: RefCell<FftPlanner<f32>> = RefCell::new(FftPlanner::new());
}

pub fn with_planner<R>(f: impl FnOnce(&mut FftPlanner<f32>) -> R) -> R {
    PLANNER.with(|p| f(&mut p.borrow_mut()))
}

/// Hann window via apodize crate
pub fn hann_window(size: usize) -> Vec<f32> {
    apodize::hanning_iter(size).map(|x| x as f32).collect()
}

/// Hamming window — better sidelobe suppression than Hann
pub fn hamming_window(size: usize) -> Vec<f32> {
    apodize::hamming_iter(size).map(|x| x as f32).collect()
}

/// Blackman window — excellent sidelobe rejection, wider main lobe
pub fn blackman_window(size: usize) -> Vec<f32> {
    apodize::blackman_iter(size).map(|x| x as f32).collect()
}

/// Nuttall window — near-optimal sidelobe suppression
pub fn nuttall_window(size: usize) -> Vec<f32> {
    apodize::nuttall_iter(size).map(|x| x as f32).collect()
}

pub fn find_zero_crossing(samples: &[f32], pos: usize, search_radius: usize) -> usize {
    let start = pos.saturating_sub(search_radius);
    let end = (pos + search_radius).min(samples.len().saturating_sub(1));
    let mut best = pos.min(samples.len().saturating_sub(1));
    let mut best_val = f32::MAX;
    for i in start..=end {
        let v = samples[i].abs();
        if v < best_val {
            best_val = v;
            best = i;
        }
    }
    best
}

pub fn stft(samples: &[f32], fft_size: usize, hop: usize) -> Vec<Vec<Complex32>> {
    let window = hann_window(fft_size);
    let fft = with_planner(|p| p.plan_fft_forward(fft_size));
    let num_frames = if samples.len() >= fft_size {
        (samples.len() - fft_size) / hop + 1
    } else {
        0
    };
    // Pre-allocate all frames at once to reduce allocator pressure
    let mut frames: Vec<Vec<Complex32>> = (0..num_frames)
        .map(|_| vec![Complex32::new(0.0, 0.0); fft_size])
        .collect();
    let mut pos = 0usize;
    for frame in frames.iter_mut() {
        for i in 0..fft_size {
            frame[i] = Complex32::new(samples[pos + i] * window[i], 0.0);
        }
        fft.process(frame);
        pos += hop;
    }
    frames
}

pub fn istft(frames: &[Vec<Complex32>], fft_size: usize, hop: usize, output_len: usize) -> Vec<f32> {
    let window = hann_window(fft_size);
    let ifft = with_planner(|p| p.plan_fft_inverse(fft_size));
    let total = if output_len > 0 {
        output_len
    } else {
        (frames.len() - 1) * hop + fft_size
    };
    let mut output = vec![0.0f32; total];
    let mut norm = vec![0.0f32; total];
    let scale = 1.0 / fft_size as f32;
    // Reusable buffer for IFFT
    let mut buf = vec![Complex32::new(0.0, 0.0); fft_size];
    for (idx, frame) in frames.iter().enumerate() {
        let offset = idx * hop;
        buf.copy_from_slice(frame);
        ifft.process(&mut buf);
        for i in 0..fft_size {
            if offset + i < total {
                output[offset + i] += buf[i].re * scale * window[i];
                norm[offset + i] += window[i] * window[i];
            }
        }
    }
    for i in 0..total {
        if norm[i] > 1e-8 {
            output[i] /= norm[i];
        }
    }
    output
}

pub fn crossfade(a: &[f32], b: &[f32], len: usize) -> Vec<f32> {
    let n = len.min(a.len()).min(b.len());
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            // Equal-power crossfade using cos/sin
            let gain_a = (0.5 * PI * t).cos();
            let gain_b = (0.5 * PI * t).sin();
            a[i] * gain_a + b[i] * gain_b
        })
        .collect()
}

/// Apply a window function to a frame of samples in-place
pub fn apply_window(frame: &mut [f32], window: &[f32]) {
    let len = frame.len().min(window.len());
    for i in 0..len {
        frame[i] *= window[i];
    }
}

/// Normalize samples so the peak absolute value equals the given peak level
pub fn normalize(samples: &mut [f32], peak: f32) {
    let max_val = samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
    if max_val > 1e-8 {
        let scale = peak / max_val;
        for s in samples.iter_mut() {
            *s *= scale;
        }
    }
}
