/// Effect registry — generates ChainConfig + dispatch functions.
///
/// To add a new effect:
/// 1. Create `effects/your_effect.rs` with a process function
/// 2. Add config struct to `configs.rs`
/// 3. Add one entry to the registry macro below

use crate::effects;
use crate::configs::*;

// ============================================================
// ChainConfig — all effect config fields
// ============================================================

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ChainConfig {
    // Modifying effects
    pub beats: Option<BeatsConfig>,
    pub reshape: Option<ReshapeConfig>,
    pub reverb: Option<ReverbConfig>,
    pub warp: Option<WarpConfig>,
    pub ref_warp: Option<RefWarpConfig>,
    pub sub_bass: Option<SubBassConfig>,
    pub deepen: Option<DeepenConfig>,
    pub tape_flutter: Option<TapeFlutterConfig>,
    // Mastering effects
    pub saturate: Option<SaturateConfig>,
    pub excite: Option<ExciteConfig>,
    pub punch: Option<PunchConfig>,
    pub auto_eq: Option<AutoEqConfig>,
    pub fp_disrupt: Option<FpDisruptConfig>,
    // Stereo
    pub stereo_widen: Option<StereoWidenConfig>,
    // Flags
    pub seamless_loop: bool,
}

impl ChainConfig {
    pub fn default_config() -> Self {
        Self {
            beats: None, reshape: None, reverb: None, warp: None,
            ref_warp: None, sub_bass: None, deepen: None, tape_flutter: None,
            saturate: None, excite: None, punch: None,
            auto_eq: None, fp_disrupt: None, stereo_widen: None,
            seamless_loop: false,
        }
    }
}

// ============================================================
// process_mono_chain — direct inline calls, no function pointers
// ============================================================

#[inline(never)] // keep as a single function for profiling
pub fn process_mono_chain(
    samples: &[f32],
    reference: Option<&[f32]>,
    sr: u32,
    config: &ChainConfig,
) -> Vec<f32> {
    let mut buf = samples.to_vec();

    // --- Modifying effects ---

    if let Some(ref cfg) = config.beats {
        buf = effects::beats::process_beats(
            &buf, sr, cfg.reverse_prob, cfg.reorder, cfg.half_time, cfg.stutter, cfg.seed,
        );
    }

    if let Some(ref cfg) = config.reshape {
        buf = effects::reshape::process_reshape(&buf, sr, cfg.spread, cfg.center);
    }

    if let Some(ref cfg) = config.reverb {
        buf = effects::reverb::process_reverb(&buf, sr, cfg.size, cfg.damping, cfg.mix);
    }

    if let Some(ref cfg) = config.warp {
        buf = effects::warp::process_warp(&buf, sr, cfg.rate, cfg.grain_ms);
    }

    if let Some(ref cfg) = config.ref_warp {
        if let Some(ref_data) = reference {
            buf = effects::ref_warp::process_ref_warp(&buf, ref_data, sr, cfg.amount);
        }
        // No reference data → skip entirely (no copy)
    }

    if let Some(ref cfg) = config.sub_bass {
        buf = effects::sub_bass::process_sub_bass(&buf, sr, cfg.amount, cfg.freq);
    }

    if let Some(ref cfg) = config.deepen {
        buf = effects::deepen::process_deepen(&buf, sr, cfg.amount, cfg.freq);
    }

    if let Some(ref cfg) = config.tape_flutter {
        buf = effects::tape_flutter::process_tape_flutter(&buf, sr, cfg.rate, cfg.depth, cfg.mix);
    }

    // --- Mastering effects ---

    if let Some(ref cfg) = config.saturate {
        buf = effects::saturate::process_saturate(&buf, sr, cfg.drive, cfg.warmth);
    }

    if let Some(ref cfg) = config.excite {
        buf = effects::excite::process_excite(&buf, sr, cfg.freq, cfg.amount, cfg.tone);
    }

    if let Some(ref cfg) = config.punch {
        buf = effects::transient::process_transient(&buf, sr, cfg.attack, cfg.sustain);
    }

    if let Some(ref cfg) = config.auto_eq {
        buf = effects::auto_eq::process_auto_eq(&buf, sr, &cfg.preset, cfg.intensity);
    }

    if let Some(ref cfg) = config.fp_disrupt {
        buf = effects::fp_disrupt::process_fp_disrupt(&buf, sr, cfg.strength);
    }

    // --- Flags ---

    if config.seamless_loop {
        buf = effects::loop_maker::process_loop(&buf, sr);
    }

    // No normalization here — happens in interleave_and_normalize() after stereo
    buf
}

// ============================================================
// Two-phase processing for chunked streaming
// ============================================================

/// Phase 1: Structural effects that change audio timing/length.
/// Must process the FULL buffer. These are O(n) and fast.
pub fn process_structural(
    samples: &[f32],
    reference: Option<&[f32]>,
    sr: u32,
    config: &ChainConfig,
) -> Vec<f32> {
    let mut buf = samples.to_vec();

    if let Some(ref cfg) = config.beats {
        buf = effects::beats::process_beats(
            &buf, sr, cfg.reverse_prob, cfg.reorder, cfg.half_time, cfg.stutter, cfg.seed,
        );
    }

    if let Some(ref cfg) = config.warp {
        buf = effects::warp::process_warp(&buf, sr, cfg.rate, cfg.grain_ms);
    }

    if let Some(ref cfg) = config.ref_warp {
        if let Some(ref_data) = reference {
            buf = effects::ref_warp::process_ref_warp(&buf, ref_data, sr, cfg.amount);
        }
    }

    if config.seamless_loop {
        buf = effects::loop_maker::process_loop(&buf, sr);
    }

    buf
}

/// Phase 2: FX effects that can be processed on arbitrary chunks.
/// These are the expensive STFT effects + cheap sample-by-sample effects.
/// Caller is responsible for providing overlap at chunk boundaries for
/// STFT effects (reshape, fp_disrupt) — typically fft_size (4096) samples.
pub fn process_fx_chunk(
    samples: &[f32],
    sr: u32,
    config: &ChainConfig,
) -> Vec<f32> {
    let mut buf = samples.to_vec();

    if let Some(ref cfg) = config.reshape {
        buf = effects::reshape::process_reshape(&buf, sr, cfg.spread, cfg.center);
    }

    if let Some(ref cfg) = config.reverb {
        buf = effects::reverb::process_reverb(&buf, sr, cfg.size, cfg.damping, cfg.mix);
    }

    if let Some(ref cfg) = config.sub_bass {
        buf = effects::sub_bass::process_sub_bass(&buf, sr, cfg.amount, cfg.freq);
    }

    if let Some(ref cfg) = config.deepen {
        buf = effects::deepen::process_deepen(&buf, sr, cfg.amount, cfg.freq);
    }

    if let Some(ref cfg) = config.tape_flutter {
        buf = effects::tape_flutter::process_tape_flutter(&buf, sr, cfg.rate, cfg.depth, cfg.mix);
    }

    if let Some(ref cfg) = config.saturate {
        buf = effects::saturate::process_saturate(&buf, sr, cfg.drive, cfg.warmth);
    }

    if let Some(ref cfg) = config.excite {
        buf = effects::excite::process_excite(&buf, sr, cfg.freq, cfg.amount, cfg.tone);
    }

    if let Some(ref cfg) = config.punch {
        buf = effects::transient::process_transient(&buf, sr, cfg.attack, cfg.sustain);
    }

    if let Some(ref cfg) = config.auto_eq {
        buf = effects::auto_eq::process_auto_eq(&buf, sr, &cfg.preset, cfg.intensity);
    }

    if let Some(ref cfg) = config.fp_disrupt {
        buf = effects::fp_disrupt::process_fp_disrupt(&buf, sr, cfg.strength);
    }

    buf
}

// ============================================================
// process_stereo_chain — stereo-aware effects
// ============================================================

pub fn process_stereo_chain(
    out_l: &mut Vec<f32>,
    out_r: &mut Vec<f32>,
    sr: u32,
    config: &ChainConfig,
) {
    if let Some(ref cfg) = config.stereo_widen {
        let (new_l, new_r) = effects::stereo_widen::process_stereo_widen(out_l, out_r, sr, cfg.width);
        *out_l = new_l;
        *out_r = new_r;
    }
}
