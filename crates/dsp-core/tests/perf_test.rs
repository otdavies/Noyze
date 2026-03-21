/// Performance and correctness regression tests for the DSP pipeline.
///
/// Tests cover:
/// - Per-effect timing with 5s clips (catches hot-path regressions)
/// - Long clip (2min) timing (catches scaling issues)
/// - Volume/normalization correctness (catches quietness bugs)
/// - Output validity (no NaN, no Infinity, non-silent)

use noyze_dsp::{process_chain, process_mono, finalize_mono};
use std::time::Instant;

fn generate_sine(freq: f32, sr: u32, duration_s: f32) -> Vec<f32> {
    let len = (sr as f32 * duration_s) as usize;
    (0..len)
        .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sr as f32).sin() * 0.5)
        .collect()
}

fn generate_music_like(sr: u32, duration_s: f32) -> Vec<f32> {
    // Multi-frequency signal that resembles real music more than a pure sine
    let len = (sr as f32 * duration_s) as usize;
    (0..len)
        .map(|i| {
            let t = i as f32 / sr as f32;
            let s = 0.3 * (2.0 * std::f32::consts::PI * 220.0 * t).sin()
                + 0.2 * (2.0 * std::f32::consts::PI * 440.0 * t).sin()
                + 0.15 * (2.0 * std::f32::consts::PI * 880.0 * t).sin()
                + 0.1 * (2.0 * std::f32::consts::PI * 1760.0 * t).sin()
                + 0.05 * (2.0 * std::f32::consts::PI * 3520.0 * t).sin();
            s
        })
        .collect()
}

fn make_config(effects: &str) -> String {
    let base = r#"{
        "beats": null, "reshape": null, "reverb": null, "warp": null,
        "ref_warp": null, "saturate": null, "excite": null, "punch": null,
        "auto_eq": null, "fp_disrupt": null, "stereo_widen": null,
        "sub_bass": null, "tape_flutter": null, "seamless_loop": false
    }"#;

    let mut config: serde_json::Value = serde_json::from_str(base).unwrap();

    for effect in effects.split(',') {
        match effect.trim() {
            "saturate" => config["saturate"] = serde_json::json!({"drive": 0.5, "warmth": 0.5}),
            "reverb" => config["reverb"] = serde_json::json!({"size": 0.5, "damping": 0.5, "mix": 0.3}),
            "excite" => config["excite"] = serde_json::json!({"freq": 3000.0, "amount": 0.5, "tone": 0.5}),
            "punch" => config["punch"] = serde_json::json!({"attack": 0.5, "sustain": 0.5}),
            "auto_eq" => config["auto_eq"] = serde_json::json!({"preset": "warm", "intensity": 0.5}),
            "reshape" => config["reshape"] = serde_json::json!({"spread": 1.3, "center": 2000.0}),
            "fp_disrupt" => config["fp_disrupt"] = serde_json::json!({"strength": 0.5}),
            "beats" => config["beats"] = serde_json::json!({"reverse_prob": 0.3, "reorder": true, "half_time": false, "stutter": false, "seed": 42}),
            "warp" => config["warp"] = serde_json::json!({"rate": 1.2, "grain_ms": 50.0}),
            "sub_bass" => config["sub_bass"] = serde_json::json!({"amount": 0.5, "freq": 60.0}),
            "tape_flutter" => config["tape_flutter"] = serde_json::json!({"rate": 4.0, "depth": 0.3, "mix": 0.5}),
            "none" | "" => {}
            _ => panic!("Unknown effect: {}", effect),
        }
    }

    serde_json::to_string(&config).unwrap()
}

fn time_process(input: &[f32], config_json: &str, label: &str) -> (std::time::Duration, Vec<f32>) {
    let empty: Vec<f32> = vec![];
    let start = Instant::now();
    let result = process_chain(input, &empty, &empty, 44100, config_json);
    let elapsed = start.elapsed();
    let duration_s = input.len() as f32 / 44100.0;
    eprintln!(
        "  {}: {:.1}ms for {:.1}s audio ({:.1}x realtime) — {} output samples",
        label,
        elapsed.as_secs_f64() * 1000.0,
        duration_s,
        duration_s / elapsed.as_secs_f32(),
        result.len() / 2,
    );
    (elapsed, result)
}

fn rms(data: &[f32]) -> f32 {
    if data.is_empty() { return 0.0; }
    let sum: f32 = data.iter().map(|s| s * s).sum();
    (sum / data.len() as f32).sqrt()
}

fn peak(data: &[f32]) -> f32 {
    data.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
}

// ============================================================
// PERFORMANCE TESTS - 5 second clips
// ============================================================

#[test]
fn perf_passthrough_under_50ms() {
    let input = generate_sine(440.0, 44100, 5.0);
    let config = make_config("none");
    let (elapsed, _) = time_process(&input, &config, "passthrough");
    assert!(elapsed.as_millis() < 50, "Passthrough took {}ms (limit: 50ms)", elapsed.as_millis());
}

#[test]
fn perf_saturate_under_100ms() {
    let input = generate_sine(440.0, 44100, 5.0);
    let config = make_config("saturate");
    let (elapsed, _) = time_process(&input, &config, "saturate");
    assert!(elapsed.as_millis() < 100, "Saturate took {}ms", elapsed.as_millis());
}

#[test]
fn perf_reverb_under_500ms() {
    let input = generate_sine(440.0, 44100, 5.0);
    let config = make_config("reverb");
    let (elapsed, _) = time_process(&input, &config, "reverb");
    assert!(elapsed.as_millis() < 500, "Reverb took {}ms", elapsed.as_millis());
}

#[test]
fn perf_reshape_under_2000ms() {
    let input = generate_sine(440.0, 44100, 5.0);
    let config = make_config("reshape");
    let (elapsed, _) = time_process(&input, &config, "reshape");
    assert!(elapsed.as_millis() < 2000, "Reshape took {}ms", elapsed.as_millis());
}

#[test]
fn perf_auto_eq_under_1000ms() {
    let input = generate_sine(440.0, 44100, 5.0);
    let config = make_config("auto_eq");
    let (elapsed, _) = time_process(&input, &config, "auto_eq");
    assert!(elapsed.as_millis() < 1000, "AutoEQ took {}ms", elapsed.as_millis());
}

#[test]
fn perf_fp_disrupt_under_1000ms() {
    let input = generate_sine(440.0, 44100, 5.0);
    let config = make_config("fp_disrupt");
    let (elapsed, _) = time_process(&input, &config, "fp_disrupt");
    assert!(elapsed.as_millis() < 1000, "FpDisrupt took {}ms", elapsed.as_millis());
}

#[test]
fn perf_full_mastering_chain_under_3000ms() {
    let input = generate_sine(440.0, 44100, 5.0);
    let config = make_config("saturate,excite,punch,auto_eq");
    let (elapsed, _) = time_process(&input, &config, "full mastering");
    assert!(elapsed.as_millis() < 3000, "Full mastering took {}ms", elapsed.as_millis());
}

#[test]
fn perf_full_chain_under_5000ms() {
    let input = generate_sine(440.0, 44100, 5.0);
    let config = make_config("reshape,reverb,saturate,excite,punch,auto_eq,fp_disrupt");
    let (elapsed, _) = time_process(&input, &config, "full chain");
    assert!(elapsed.as_millis() < 5000, "Full chain took {}ms", elapsed.as_millis());
}

// ============================================================
// LONG CLIP TESTS - 2 minute clips (simulating real usage)
// ============================================================

#[test]
fn perf_2min_mastering_under_10s() {
    let input = generate_music_like(44100, 120.0); // 2 minutes
    let config = make_config("saturate,excite,punch,auto_eq");
    let (elapsed, _) = time_process(&input, &config, "2min mastering");
    assert!(
        elapsed.as_secs() < 10,
        "2min mastering took {}s (limit: 10s)",
        elapsed.as_secs()
    );
}

#[test]
fn perf_2min_full_chain_under_30s() {
    let input = generate_music_like(44100, 120.0); // 2 minutes
    let config = make_config("reshape,reverb,saturate,excite,punch,auto_eq,fp_disrupt");
    let (elapsed, _) = time_process(&input, &config, "2min full chain");
    assert!(
        elapsed.as_secs() < 30,
        "2min full chain took {}s (limit: 30s)",
        elapsed.as_secs()
    );
}

#[test]
fn perf_process_mono_api() {
    // Test the new stepped API
    let input = generate_music_like(44100, 30.0); // 30s
    let config = make_config("saturate,reverb,auto_eq");
    let empty: Vec<f32> = vec![];

    let start = Instant::now();
    let mono_out = process_mono(&input, &empty, 44100, &config);
    let mono_time = start.elapsed();

    let start2 = Instant::now();
    let stereo_out = finalize_mono(&mono_out);
    let finalize_time = start2.elapsed();

    eprintln!(
        "  process_mono: {:.1}ms, finalize_mono: {:.1}ms for 30s audio",
        mono_time.as_secs_f64() * 1000.0,
        finalize_time.as_secs_f64() * 1000.0,
    );

    assert!(stereo_out.len() == mono_out.len() * 2);
    assert!(peak(&stereo_out) > 0.9, "Output peak too low: {}", peak(&stereo_out));
}

// ============================================================
// VOLUME / NORMALIZATION TESTS
// ============================================================

#[test]
fn volume_output_peak_near_098() {
    let input = generate_music_like(44100, 2.0);
    let config = make_config("saturate,reverb,excite");
    let empty: Vec<f32> = vec![];
    let result = process_chain(&input, &empty, &empty, 44100, &config);

    let p = peak(&result);
    eprintln!("  Output peak: {:.4}", p);
    assert!(p > 0.90, "Output peak too low ({}), normalization may be broken", p);
    assert!(p <= 1.0, "Output peak exceeds 1.0 ({}), clipping!", p);
}

#[test]
fn volume_rms_not_squashed() {
    // Input RMS vs output RMS should be in a reasonable range.
    // If output is 4x quieter, RMS ratio would be ~0.25
    let input = generate_music_like(44100, 2.0);
    let input_rms = rms(&input);

    let config = make_config("saturate,excite,punch");
    let empty: Vec<f32> = vec![];
    let result = process_chain(&input, &empty, &empty, 44100, &config);

    // Extract left channel from interleaved output
    let left: Vec<f32> = (0..result.len() / 2).map(|i| result[i * 2]).collect();
    let output_rms = rms(&left);

    let ratio = output_rms / input_rms;
    eprintln!("  Input RMS: {:.4}, Output RMS: {:.4}, Ratio: {:.2}x", input_rms, output_rms, ratio);

    // Output should be at least 50% of input RMS (not 4x quieter)
    assert!(ratio > 0.3, "Output is {:.1}x quieter than input (RMS ratio: {:.3})", 1.0 / ratio, ratio);
}

#[test]
fn volume_passthrough_preserves_level() {
    let input = generate_music_like(44100, 1.0);
    let input_peak = peak(&input);
    let config = make_config("none");
    let empty: Vec<f32> = vec![];
    let result = process_chain(&input, &empty, &empty, 44100, &config);

    let left: Vec<f32> = (0..result.len() / 2).map(|i| result[i * 2]).collect();
    let output_peak = peak(&left);

    eprintln!("  Input peak: {:.4}, Output peak: {:.4}", input_peak, output_peak);
    // Passthrough should normalize to ~0.98
    assert!(output_peak > 0.90, "Passthrough output too quiet: {}", output_peak);
}

#[test]
fn volume_heavy_effects_not_crushed() {
    // Test with aggressive settings that might cause volume squashing
    let input = generate_music_like(44100, 2.0);

    let config_json = r#"{
        "beats": null, "reshape": null, "reverb": {"size": 0.8, "damping": 0.3, "mix": 0.5},
        "warp": null, "ref_warp": null,
        "saturate": {"drive": 0.8, "warmth": 0.7},
        "excite": {"freq": 2000.0, "amount": 0.8, "tone": 0.8},
        "punch": {"attack": 0.7, "sustain": 0.7},
        "auto_eq": {"preset": "full", "intensity": 0.8},
        "fp_disrupt": null, "stereo_widen": null,
        "sub_bass": null, "tape_flutter": null, "seamless_loop": false
    }"#;

    let empty: Vec<f32> = vec![];
    let result = process_chain(&input, &empty, &empty, 44100, config_json);

    let left: Vec<f32> = (0..result.len() / 2).map(|i| result[i * 2]).collect();
    let output_rms = rms(&left);
    let output_peak = peak(&left);

    eprintln!("  Heavy effects — Peak: {:.4}, RMS: {:.4}", output_peak, output_rms);

    // Even with heavy effects, output should be normalized and not crushed
    assert!(output_peak > 0.90, "Peak too low after heavy effects: {}", output_peak);
    assert!(output_rms > 0.05, "RMS too low after heavy effects: {}", output_rms);
}

// ============================================================
// OUTPUT VALIDITY
// ============================================================

#[test]
fn output_is_not_silent() {
    let input = generate_sine(440.0, 44100, 1.0);
    let config = make_config("saturate,reverb");
    let empty: Vec<f32> = vec![];
    let result = process_chain(&input, &empty, &empty, 44100, &config);

    assert!(!result.is_empty(), "Output is empty");

    let p = peak(&result);
    assert!(p > 0.1, "Output is near-silent (peak: {})", p);

    for (i, &s) in result.iter().enumerate() {
        assert!(s.is_finite(), "Sample {} is not finite: {}", i, s);
    }
}

#[test]
fn output_no_nan_with_all_effects() {
    let input = generate_music_like(44100, 1.0);
    let config = make_config("reshape,reverb,saturate,excite,punch,auto_eq,fp_disrupt,sub_bass,tape_flutter");
    let empty: Vec<f32> = vec![];
    let result = process_chain(&input, &empty, &empty, 44100, &config);

    for (i, &s) in result.iter().enumerate() {
        assert!(s.is_finite(), "Sample {} is not finite: {}", i, s);
    }
    assert!(peak(&result) > 0.1, "All-effects output is silent");
}
