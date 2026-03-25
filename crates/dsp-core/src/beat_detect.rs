use crate::fft_utils::find_zero_crossing;

/// Estimate tempo (BPM) from energy onset signal using autocorrelation.
fn estimate_tempo(flux: &[f32], hop: usize, sample_rate: u32) -> (f32, usize) {
    let sr = sample_rate as f32;
    let min_lag = (60.0 * sr / (200.0 * hop as f32)) as usize;
    let max_lag = (60.0 * sr / (60.0 * hop as f32)) as usize;
    let max_lag = max_lag.min(flux.len() / 2);

    if min_lag >= max_lag || flux.len() < max_lag * 2 {
        let period = (sr * 60.0 / 120.0) as usize;
        return (120.0, period);
    }

    // Subsample the autocorrelation — check every 2nd lag, then refine
    let mut best_corr = 0.0f32;
    let mut best_lag = min_lag;
    let count_max = flux.len();

    for lag in (min_lag..max_lag).step_by(2) {
        let count = count_max - lag;
        let mut corr = 0.0f32;
        // Sample every 4th frame for speed
        let mut i = 0;
        while i < count {
            corr += flux[i] * flux[i + lag];
            i += 4;
        }
        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    // Fine search around best
    let fine_start = best_lag.saturating_sub(2);
    let fine_end = (best_lag + 2).min(max_lag);
    for lag in fine_start..=fine_end {
        let count = count_max - lag;
        let mut corr = 0.0f32;
        for i in 0..count {
            corr += flux[i] * flux[i + lag];
        }
        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    let beat_period_samples = best_lag * hop;
    let bpm = 60.0 * sr / beat_period_samples as f32;
    (bpm, beat_period_samples)
}

/// Build a beat grid from estimated tempo, aligned to the strongest onset.
fn build_beat_grid(
    flux: &[f32],
    hop: usize,
    beat_period_frames: usize,
    total_samples: usize,
) -> Vec<usize> {
    if beat_period_frames == 0 {
        return vec![0, total_samples];
    }

    let mut best_frame = 0;
    let mut best_val = 0.0f32;
    for (i, &v) in flux.iter().enumerate() {
        if v > best_val {
            best_val = v;
            best_frame = i;
        }
    }

    let beat_period_samples = beat_period_frames * hop;
    let anchor = best_frame * hop;

    let mut grid = Vec::new();
    let mut pos = anchor as isize;
    while pos > 0 {
        pos -= beat_period_samples as isize;
    }
    pos = pos.max(0);
    while (pos as usize) < total_samples {
        grid.push(pos as usize);
        pos += beat_period_samples as isize;
    }

    if grid.is_empty() || grid[0] != 0 {
        grid.insert(0, 0);
    }
    if let Some(&last) = grid.last() {
        if last < total_samples {
            grid.push(total_samples);
        }
    }

    grid
}

/// Snap detected onsets to the nearest beat grid position.
fn quantize_onsets_to_grid(
    raw_onsets: &[usize],
    beat_grid: &[usize],
    flux: &[f32],
    hop: usize,
    sample_rate: u32,
) -> Vec<usize> {
    if beat_grid.len() < 2 {
        return raw_onsets.to_vec();
    }

    let tolerance_samples = (sample_rate as f32 * 0.08) as usize;
    let global_mean: f32 = flux.iter().sum::<f32>() / flux.len().max(1) as f32;

    let mut result = Vec::new();
    result.push(0);

    for &grid_pos in &beat_grid[1..beat_grid.len() - 1] {
        let has_nearby_onset = raw_onsets.iter().any(|&o| {
            let diff = if o > grid_pos { o - grid_pos } else { grid_pos - o };
            diff < tolerance_samples
        });

        if has_nearby_onset {
            let frame_idx = (grid_pos / hop).min(flux.len().saturating_sub(1));
            let local_start = frame_idx.saturating_sub(3);
            let local_end = (frame_idx + 4).min(flux.len());
            let local_max = flux[local_start..local_end]
                .iter()
                .cloned()
                .fold(0.0f32, f32::max);

            if local_max > global_mean * 0.5 {
                if let Some(&last) = result.last() {
                    if grid_pos > last + hop * 4 {
                        result.push(grid_pos);
                    }
                }
            }
        }
    }

    if let Some(&last) = result.last() {
        let total = beat_grid.last().copied().unwrap_or(0);
        if last < total {
            result.push(total);
        }
    }

    result
}

/// Compute a compact spectral fingerprint for a section of audio.
pub fn spectral_fingerprint(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    let n = 2048.min(samples.len());
    if n < 64 {
        return vec![0.0; 8];
    }

    let fft = crate::fft_utils::with_planner(|p| p.plan_fft_forward(n));
    let window = crate::fft_utils::hann_window(n);
    let mut buf: Vec<rustfft::num_complex::Complex32> = (0..n)
        .map(|i| rustfft::num_complex::Complex32::new(
            if i < samples.len() { samples[i] * window[i] } else { 0.0 }, 0.0
        ))
        .collect();
    fft.process(&mut buf);

    let sr = sample_rate as f32;
    let band_edges = [0.0, 100.0, 300.0, 600.0, 1200.0, 2400.0, 5000.0, 10000.0, 20000.0];
    let mut energies = vec![0.0f32; 8];
    let freq_per_bin = sr / n as f32;

    for band in 0..8 {
        let lo_bin = (band_edges[band] / freq_per_bin) as usize;
        let hi_bin = ((band_edges[band + 1] / freq_per_bin) as usize).min(n / 2);
        let mut sum = 0.0f32;
        let mut count = 0;
        for i in lo_bin..hi_bin {
            sum += (buf[i].re * buf[i].re + buf[i].im * buf[i].im).sqrt();
            count += 1;
        }
        energies[band] = if count > 0 { sum / count as f32 } else { 0.0 };
    }

    let max_e = energies.iter().cloned().fold(0.001f32, f32::max);
    for e in energies.iter_mut() {
        *e /= max_e;
    }
    energies
}

/// Compute cosine similarity between two spectral fingerprints.
pub fn fingerprint_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = (na * nb).sqrt();
    if denom > 1e-8 { dot / denom } else { 0.0 }
}

/// Detect beat/section boundaries using energy-based onset detection
/// with tempo-aware beat grid quantization.
/// Uses RMS energy difference instead of STFT spectral flux for speed.
pub fn detect_onsets(samples: &[f32], sample_rate: u32) -> Vec<usize> {
    let hop = 512;
    let window = 1024;

    if samples.len() < window * 2 {
        return vec![0, samples.len()];
    }

    // Compute energy flux using RMS in short windows — no FFT needed
    let num_frames = (samples.len() - window) / hop + 1;
    let mut energy = Vec::with_capacity(num_frames);
    for f in 0..num_frames {
        let start = f * hop;
        let end = (start + window).min(samples.len());
        let mut sum = 0.0f32;
        for i in start..end {
            sum += samples[i] * samples[i];
        }
        energy.push((sum / (end - start) as f32).sqrt());
    }

    // Energy difference (half-wave rectified)
    let mut flux = Vec::with_capacity(energy.len());
    flux.push(0.0f32);
    for i in 1..energy.len() {
        let diff = energy[i] - energy[i - 1];
        flux.push(if diff > 0.0 { diff } else { 0.0 });
    }

    // Estimate tempo from onset strength
    let (_bpm, beat_period) = estimate_tempo(&flux, hop, sample_rate);

    // Build beat grid
    let beat_period_frames = beat_period / hop;
    let beat_grid = build_beat_grid(&flux, hop, beat_period_frames, samples.len());

    // Raw onset detection via adaptive threshold peak-picking
    let window_size = 10;
    let mut raw_onsets = Vec::new();
    raw_onsets.push(0usize);

    for i in 1..flux.len().saturating_sub(1) {
        let start = i.saturating_sub(window_size);
        let end = (i + window_size + 1).min(flux.len());
        let local_mean: f32 = flux[start..end].iter().sum::<f32>() / (end - start) as f32;
        let threshold = local_mean * 1.5;

        if flux[i] > threshold && flux[i] > flux[i - 1] && flux[i] > flux[i + 1] {
            let sample_pos = i * hop;
            if sample_pos < samples.len() {
                if let Some(&last) = raw_onsets.last() {
                    if sample_pos > last + hop * 4 {
                        raw_onsets.push(sample_pos);
                    }
                }
            }
        }
    }
    raw_onsets.push(samples.len());

    // Quantize to beat grid
    let mut quantized = quantize_onsets_to_grid(&raw_onsets, &beat_grid, &flux, hop, sample_rate);

    // Snap all positions to zero crossings
    let search_radius = (sample_rate as f32 * 0.005) as usize;
    for pos in quantized.iter_mut() {
        if *pos > 0 && *pos < samples.len() {
            *pos = find_zero_crossing(samples, *pos, search_radius);
        }
    }

    if quantized.is_empty() || quantized[0] != 0 {
        quantized.insert(0, 0);
    }
    if let Some(&last) = quantized.last() {
        if last < samples.len() {
            quantized.push(samples.len());
        }
    }

    quantized.dedup();

    let min_section = (sample_rate as f32 * 0.05) as usize;
    let mut filtered = vec![quantized[0]];
    for &pos in &quantized[1..] {
        if pos - *filtered.last().unwrap() >= min_section {
            filtered.push(pos);
        }
    }
    if let Some(&last) = filtered.last() {
        if last < samples.len() {
            filtered.push(samples.len());
        }
    }

    filtered
}
