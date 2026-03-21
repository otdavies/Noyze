// This module is kept for backwards compatibility but all processing
// is now handled by the registry macro in registry.rs.
// See registry.rs for the effect chain dispatch.

pub use crate::registry::{process_mono_chain as process_mono, process_stereo_chain as process_stereo_widen_chain};
