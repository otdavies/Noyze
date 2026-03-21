//! Effect config structs.
//! Each struct is used by the registry macro to build ChainConfig.
//! Adding a new effect config: define the struct here, then reference it in registry.rs.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BeatsConfig {
    pub reverse_prob: f32,
    pub reorder: bool,
    pub half_time: bool,
    pub stutter: bool,
    pub seed: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReshapeConfig {
    pub spread: f32,
    pub center: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReverbConfig {
    pub size: f32,
    pub damping: f32,
    pub mix: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WarpConfig {
    pub rate: f32,
    pub grain_ms: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RefWarpConfig {
    pub amount: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaturateConfig {
    pub drive: f32,
    pub warmth: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExciteConfig {
    pub freq: f32,
    pub amount: f32,
    pub tone: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PunchConfig {
    pub attack: f32,
    pub sustain: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AutoEqConfig {
    pub preset: String,
    pub intensity: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FpDisruptConfig {
    pub strength: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StereoWidenConfig {
    pub width: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubBassConfig {
    pub amount: f32,
    pub freq: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TapeFlutterConfig {
    pub rate: f32,
    pub depth: f32,
    pub mix: f32,
}
