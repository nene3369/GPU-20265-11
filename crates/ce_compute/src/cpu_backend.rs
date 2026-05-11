//! CPU fallback compute backend.

use crate::backend::{BackendKind, ComputeCapabilities, ComputeEngine};

/// A CPU-only fallback compute backend.
///
/// This backend does not dispatch to any GPU — it exists so that the
/// engine can function (albeit slowly) on machines without a suitable
/// GPU adapter.
pub struct CpuBackend;

impl CpuBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CpuBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeEngine for CpuBackend {
    fn capabilities(&self) -> ComputeCapabilities {
        ComputeCapabilities {
            backend: BackendKind::Cpu,
            max_workgroup_size: 1,
            max_buffer_size: u64::MAX,
        }
    }

    fn backend_name(&self) -> &str {
        "cpu"
    }
}
