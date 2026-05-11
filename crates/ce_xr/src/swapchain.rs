/// Manages OpenXR swapchain for stereo rendering.
/// Full implementation in M2 with Vulkan graphics binding.
pub struct XrSwapchain {
    pub width: u32,
    pub height: u32,
    pub sample_count: u32,
}

impl XrSwapchain {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            sample_count: 1,
        }
    }
}
