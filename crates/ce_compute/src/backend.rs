//! Backend trait and capability descriptors for compute engines.

/// Identifies which compute backend is in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Wgpu,
    Cuda,
    Hip,
    Cpu,
}

/// Describes the capabilities of a compute backend.
pub struct ComputeCapabilities {
    pub backend: BackendKind,
    pub max_workgroup_size: u32,
    pub max_buffer_size: u64,
}

/// Trait for GPU/CPU compute backends.
pub trait ComputeEngine: Send + Sync {
    /// Returns the capabilities of this backend.
    fn capabilities(&self) -> ComputeCapabilities;

    /// Returns a human-readable name for this backend.
    fn backend_name(&self) -> &str;
}
