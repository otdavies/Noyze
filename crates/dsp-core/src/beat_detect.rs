use crate::fft_utils::{stft, find_zero_crossing};

/// Estimate tempo (BPM) from onset strength signal using autocorrelation.
/// Returns the most likely BPM and the corresponding period in samples.
fn estimate_tempo(flux: &[f32], hop: usize, sample_rate: u32) -> (f32, usize) {
    // Search BPM range 60-200
    let sr = sample_rate as f32;
    let min_lag = (60.0 * sr / (200.0 * hop as f32)) as usize; // ~200 BPM
    let max_lag = (60.0 * sr / (60.0 * hop as f32)) as usize;  // ~60 BPM
    let max_lag = max_lag.min(flux.len() / 2);

    if min_lag >= max_lag || flux.len() < max_lag * 2 {
        // Fallback: assume 120 BPM
        let period = (sr * 60.0 / 120.0) as usize;
        return (120.0, period);
    }

    // Autocorrelation of onset strength
    let mut best_corr = 0.0f32;
    let mut best_lag = min_lag;

    for lag in min_lag..max_lag {
        let mut corr = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;
        let count = flux.len() - lag;
        for i in 0..count {
            corr += flux[i] * flux[i + lag];
            norm_a += flux[i] * flux[i];
            norm_b += flux[i + lag] * flux[i + lag];
        }
        let denom = (norm_a * norm_b).sqrt();
        if denom > 1e-8 {
            corr /= denom;
        }
        // Prefer lags that also correlate at double rate (reinforces real tempo)
        let double_lag = lag * 2;
        if double_lag < flux.len() / 2 {
            let mut corr2 = 0.0f32;
            let mut n2a = 0.0f32;
            let mut n2b = 0.0f32;
            let count2 = flux.len() - double_lag;
            for i in 0..count2 {
                corr2 += flux[i] * flux[i + double_lag];
                n2a += flux[i] * flux[i];
                n2b += flux[i + double_lag] * flux[i + double_lag];
            }
            let d2 = (n2a * n2b).sqrt();
            if d2 > 1e-8 {
                corr2 /= d2;
            }
            corr += corr2 * 0.3; // Bonus for double-period reinforcement
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

    // Find the strongest onset to use as phase reference
    let mut best_frame = 0;
    let mut best_val = 0.0f32;
    for (i, &v) in flux.iter().enumerate() {
        if v > best_val {
            best_val = v;
            best_frame = i;
        }
    }

    // Build grid going backward from best_frame to 0, then forward to end
    let beat_period_samples = beat_period_frames * hop;
    let anchor = best_frame * hop;

    let mut grid = Vec::new();

    // Go backward
    let mut pos = anchor as isize;
    while pos > 0 {
        pos -= beat_period_samples as isize;
    }
    // Go forward from earliest
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
/// Returns positions that are both musically meaningful (on-beat) and
/// acoustically valid (near actual transients).
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

    // For each beat grid position, check if there's a nearby onset.
    // If so, use the grid position (snapped to zero crossing later).
    // This ensures cuts always land on-beat.
    let tolerance_samples = (sample_rate as f32 * 0.08) as usize; // 80ms tolerance

    let mut result = Vec::new();
    result.push(0);

    for &grid_pos in &beat_grid[1..beat_grid.len() - 1] {
        // Check if any raw onset is near this grid position
        let has_nearby_onset = raw_onsets.iter().any(|&o| {
            let diff = if o > grid_pos { o - grid_pos } else { grid_pos - o };
            diff < tolerance_samples
        });

        if has_nearby_onset {
            // Also verify there's actual energy change here
            let frame_idx = (grid_pos / hop).min(flux.len().saturating_sub(1));
            let local_start = frame_idx.saturating_sub(3);
            let local_end = (frame_idx + 4).min(flux.len());
            let local_max = flux[local_start..local_end]
                .iter()
                .cloned()
                .fold(0.0f32, f32::max);
            let global_mean: f32 = flux.iter().sum::<f32>() / flux.len() as f32;

            // Only include if there's meaningful energy at this beat
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
/// Returns a vector of band energies that can be compared for similarity.
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

    // 8 mel-inspired bands
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

    // Normalize
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

/// Detect beat/section boundaries using spectral flux onset detection
/// with tempo-aware beat grid quantization.
/// Returns sample positions of detected onsets, snapped to beat grid and zero crossings.
pub fn detect_onsets(samples: &[f32], sample_rate: u32) -> Vec<usize> {
    let fft_size = 1024;
    let hop = 512;
    let frames = stft(samples, fft_size, hop);
    if frames.len() < 2 {
        return vec![0, samples.len()];
    }

    // Compute spectral flux (half-wave rectified magnitude difference)
    let mut flux = Vec::with_capacity(frames.len());
    flux.push(0.0f32);
    for i in 1..frames.len() {
        let mut sum = 0.0f32;
        for j in 0..fft_size / 2 {
            let prev_mag = (frames[i - 1][j].re * frames[i - 1][j].re
                + frames[i - 1][j].im * frames[i - 1][j].im)
                .sqrt();
            let curr_mag = (frames[i][j].re * frames[i][j].re
                + frames[i][j].im * frames[i][j].im)
                .sqrt();
            let diff = curr_mag - prev_mag;
            if diff > 0.0 {
                sum += diff;
            }
        }
        flux.push(sum);
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

    for i in 1..flux.len() - 1 {
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

    // Ensure first and last
    if quantized.is_empty() || quantized[0] != 0 {
        quantized.insert(0, 0);
    }
    if let Some(&last) = quantized.last() {
        if last < samples.len() {
            quantized.push(samples.len());
        }
    }

    // Deduplicate
    quantized.dedup();

    // Remove sections that are too small (< 50ms)
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
