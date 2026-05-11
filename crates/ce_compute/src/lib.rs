//! # ce_compute — ChemEngine Compute Shader Backend
//!
//! Provides a [`ComputeEngine`] trait with a wgpu implementation
//! ([`WgpuComputeBackend`]) and a CPU fallback ([`CpuBackend`]).
//! CUDA / HIP backends will be added in future releases.

pub mod backend;
pub mod cpu_backend;
pub mod wgpu_compute;

pub use backend::{BackendKind, ComputeCapabilities, ComputeEngine};
pub use cpu_backend::CpuBackend;
pub use wgpu_compute::WgpuComputeBackend;

use ce_app::Plugin;

/// Plugin that registers the compute subsystem with a ChemEngine [`App`].
pub struct ComputePlugin;

impl Plugin for ComputePlugin {
    fn build(&self, _app: &mut ce_app::App) {
        log::info!("ComputePlugin loaded");
    }
}
