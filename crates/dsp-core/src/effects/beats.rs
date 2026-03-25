/// Beats — musical beat manipulation effect (Gross Beat / HalfTime style).
///
/// Operates on detected beat boundaries with four modes:
///
/// - **Half-time**: Varispeed replay at 0.5x (pitch drops — this IS the desired
///   aesthetic, same as HalfTime by Cableguys / Gross Beat). Maintains original
///   duration by playing only the first half of each beat at half speed.
///
/// - **Stutter**: Beat-grid-aligned slice repetition using musical subdivisions
///   (1/2, 1/3, 1/4 beat). Per-repeat volume decay (-1dB) for natural feel.
///   Raised-cosine fades at slice boundaries eliminate clicks.
///
/// - **Reverse**: Raw time-reversal within beat boundaries. The reversed envelope
///   (decay→attack "swell") is the creative effect people want — it builds
///   tension into the next beat.
///
/// - **Reorder**: Shuffle beat positions with 15ms equal-power crossfades.

use crate::beat_detect::detect_onsets;

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self { Self(seed.wrapping_add(1)) }
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 & 0xFFFFFF) as f32 / 0xFFFFFF as f32
    }
}

/// Cubic Hermite interpolation for smooth reads at fractional buffer positions.
#[inline(always)]
fn hermite_interp(buf: &[f32], pos: f32) -> f32 {
    let len = buf.len();
    if len < 4 { return buf.get(pos as usize).copied().unwrap_or(0.0); }

    let idx = pos.floor() as isize;
    let frac = pos - pos.floor();

    let get = |i: isize| -> f32 {
        buf[i.rem_euclid(len as isize) as usize]
    };

    let y0 = get(idx - 1);
    let y1 = get(idx);
    let y2 = get(idx + 1);
    let y3 = get(idx + 2);

    let c0 = y1;
    let c1 = 0.5 * (y2 - y0);
    let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
    let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);

    ((c3 * frac + c2) * frac + c1) * frac + c0
}

/// Apply a smooth raised-cosine fade to a buffer
fn apply_fade(buf: &mut [f32], fade_in: usize, fade_out: usize) {
    let len = buf.len();
    let fi = fade_in.min(len / 2);
    let fo = fade_out.min(len / 2);

    for i in 0..fi {
        let t = i as f32 / fi as f32;
        buf[i] *= 0.5 * (1.0 - (std::f32::consts::PI * t).cos());
    }
    for i in 0..fo {
        let idx = len - 1 - i;
        let t = i as f32 / fo as f32;
        buf[idx] *= 0.5 * (1.0 - (std::f32::consts::PI * t).cos());
    }
}

/// Equal-power crossfade splice
fn crossfade_splice(output: &mut Vec<f32>, new_segment: &[f32], fade_samples: usize) {
    if output.is_empty() || fade_samples == 0 || new_segment.is_empty() {
        output.extend_from_slice(new_segment);
        return;
    }

    let overlap = fade_samples.min(output.len()).min(new_segment.len());
    let out_start = output.len() - overlap;

    for j in 0..overlap {
        let t = j as f32 / overlap as f32;
        let ga = (0.5 * std::f32::consts::PI * t).cos();
        let gb = (0.5 * std::f32::consts::PI * t).sin();
        output[out_start + j] = output[out_start + j] * ga + new_segment[j] * gb;
    }
    output.extend_from_slice(&new_segment[overlap..]);
}

/// Half-time a beat: play first half at 0.5x speed via varispeed.
/// Output length = input length (maintains original duration).
/// Pitch drops one octave — this is the desired aesthetic.
fn halftime_beat(beat: &[f32]) -> Vec<f32> {
    let out_len = beat.len();
    let mut output = Vec::with_capacity(out_len);

    for i in 0..out_len {
        // Read at half speed: output position i maps to input position i*0.5
        let read_pos = i as f32 * 0.5;
        output.push(hermite_interp(beat, read_pos));
    }
    output
}

/// Reverse a beat with fade-in at the start (the reversed "swell into beat" effect).
fn reverse_beat(beat: &[f32], fade_samples: usize) -> Vec<f32> {
    let mut reversed: Vec<f32> = beat.iter().rev().copied().collect();
    // Apply short fade-in to the start to avoid the abrupt reversed tail
    apply_fade(&mut reversed, fade_samples, 0);
    reversed
}

/// Generate a stutter pattern for a beat: repeat musical subdivisions with
/// per-repeat volume decay and smooth fades at boundaries.
fn stutter_beat(beat: &[f32], sample_rate: u32, rng: &mut Rng) -> Vec<f32> {
    let len = beat.len();
    if len < 128 { return beat.to_vec(); }

    // Fast musical subdivisions: 1/8, 1/12, 1/16 of the beat for rapid-fire stutter
    let subdivisions = [6, 8, 12, 16];
    let sub_idx = (rng.next() * subdivisions.len() as f32) as usize;
    let num_slices = subdivisions[sub_idx.min(subdivisions.len() - 1)];
    let slice_len = len / num_slices;

    if slice_len < 64 { return beat.to_vec(); }

    // Pick source slice (weighted toward beat start for musicality — transient is there)
    let source_slice = (rng.next() * rng.next() * num_slices as f32) as usize;
    let source_start = (source_slice * slice_len).min(len.saturating_sub(slice_len));

    // Fade must be shorter than the slice to avoid crossfade_splice consuming
    // the entire segment (which would prevent the output from growing → infinite loop)
    let fade_len = ((sample_rate as f32 * 0.003) as usize).min(slice_len / 3);
    let decay_per_repeat = 0.93f32;

    let mut output = Vec::with_capacity(len);
    let mut repeat_idx = 0u32;
    let max_repeats = (len / slice_len + 2) as u32;

    // Fill beat with repeats of the source slice
    while output.len() < len && repeat_idx < max_repeats {
        let remaining = len - output.len();
        let take = remaining.min(slice_len);
        let end = (source_start + take).min(len);
        let actual_take = end - source_start;
        if actual_take == 0 { break; }

        let mut slice = beat[source_start..end].to_vec();

        let gain = decay_per_repeat.powi(repeat_idx as i32);
        for s in slice.iter_mut() {
            *s *= gain;
        }

        apply_fade(&mut slice, fade_len, fade_len);
        crossfade_splice(&mut output, &slice, fade_len);
        repeat_idx += 1;
    }

    output.truncate(len);
    output
}

pub fn process_beats(
    samples: &[f32],
    sample_rate: u32,
    reverse_prob: f32,
    reorder: bool,
    half_time: bool,
    stutter: bool,
    seed: u64,
) -> Vec<f32> {
    let onsets = detect_onsets(samples, sample_rate);
    if onsets.len() < 3 {
        return samples.to_vec();
    }

    // Extract beat ranges
    let mut beat_ranges: Vec<(usize, usize)> = Vec::new();
    for i in 0..onsets.len() - 1 {
        let start = onsets[i];
        let end = onsets[i + 1].min(samples.len());
        if end > start {
            beat_ranges.push((start, end));
        }
    }

    if beat_ranges.is_empty() {
        return samples.to_vec();
    }

    let mut rng = Rng::new(seed);
    let num_beats = beat_ranges.len();

    // Determine playback order
    let mut order: Vec<usize> = (0..num_beats).collect();
    if reorder && num_beats > 3 {
        let max_swaps = (num_beats / 3).min(6);
        for _ in 0..max_swaps {
            let a = 1 + (rng.next() * (num_beats - 2) as f32) as usize;
            let b = 1 + (rng.next() * (num_beats - 2) as f32) as usize;
            if a != b {
                order.swap(a.min(num_beats - 1), b.min(num_beats - 1));
            }
        }
    }

    // Process each beat
    let fade_samples = (sample_rate as f32 * 0.015) as usize; // 15ms crossfade
    let mut output: Vec<f32> = Vec::with_capacity(samples.len());

    // Preserve samples before the first onset (intro/silence/pickup)
    let first_onset = onsets[0];
    if first_onset > 0 {
        output.extend_from_slice(&samples[..first_onset]);
    }

    for &beat_idx in &order {
        let (start, end) = beat_ranges[beat_idx];
        let beat = &samples[start..end];
        if beat.len() < 16 { continue; }

        let do_reverse = reverse_prob > 0.0 && rng.next() < reverse_prob;
        let do_stutter = stutter && rng.next() < 0.35;

        let beat_output = if do_stutter {
            stutter_beat(beat, sample_rate, &mut rng)
        } else if half_time {
            // Varispeed half-time: play first half at 0.5x, maintains duration
            let mut ht = halftime_beat(beat);
            if do_reverse {
                ht.reverse();
                apply_fade(&mut ht, fade_samples, 0);
            }
            ht
        } else if do_reverse {
            reverse_beat(beat, fade_samples)
        } else {
            // Normal playback — no processing needed
            beat.to_vec()
        };

        crossfade_splice(&mut output, &beat_output, fade_samples);
    }

    // Preserve samples after the last onset's beat range
    let last_end = beat_ranges[beat_ranges.len() - 1].1;
    if last_end < samples.len() {
        crossfade_splice(&mut output, &samples[last_end..], fade_samples);
    }

    output
}
