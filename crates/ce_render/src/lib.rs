pub mod color;
pub mod gpu;
pub mod gpu_cull;
pub mod gpu_driven;
pub mod mesh;
pub mod render_graph;
pub mod stereo;
pub mod taau;

pub use color::Color;
pub use gpu::GpuContext;
pub use gpu_cull::{CullResult, GpuAabb, GpuCullPipeline, GpuFrustum};
pub use gpu_driven::{DrawIndirectCommand, GpuDrawList, ObjectData};
pub use mesh::{Mesh, Vertex};
pub use render_graph::{GraphError, PassContext, PassId, RenderGraph, ResourceId};
pub use stereo::{EyeTarget, StereoTaauPass};
pub use taau::{add_upscale_pass, TaauInputs, TaauPass, UpscaleSettings};

use ce_app::Plugin;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, _app: &mut ce_app::App) {
        // GPU initialization happens in the example, not here.
        // This is a placeholder for future render systems.
        log::info!("RenderPlugin loaded");
    }
}
