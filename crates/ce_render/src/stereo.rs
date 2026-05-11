//! Stereo TAAU — two-eye upscaling for VR / windowed side-by-side preview.
//!
//! Thin orchestrator that owns two [`TaauPass`] instances (one per eye) and
//! runs them with decorrelated Halton jitter so that correlated sub-pixel
//! offsets do not produce a cross-eyed shimmer when the viewer fuses the
//! left/right images.
//!
//! This module does **not** talk to OpenXR. It operates on whatever
//! per-eye textures the caller provides; M2 will replace caller-owned
//! textures with OpenXR swapchain images.
//!
//! ## Decorrelation
//!
//! Left eye calls `TaauPass::execute(frame)`, right eye calls
//! `TaauPass::execute(frame + 8)`. Offsetting by 8 (half the 16-entry
//! Halton table) guarantees the two eyes sample different sub-pixel
//! positions at every frame while preserving frame-parity — which is
//! what [`TaauPass`] uses for history ping-pong. 8 is even, so history
//! ping-pong is unaffected.

use crate::taau::{TaauInputs, TaauPass, UpscaleSettings};

/// Eye-local inputs handed to [`StereoTaauPass::execute`].
pub struct EyeTarget<'a> {
    pub color_view: &'a wgpu::TextureView,
    pub depth_view: &'a wgpu::TextureView,
    pub motion_view: &'a wgpu::TextureView,
}

/// Amount to offset the right eye's Halton index. Must be even (to
/// preserve ping-pong parity) and non-zero (to decorrelate).
const RIGHT_EYE_JITTER_OFFSET: u64 = 8;

/// Two-eye TAAU orchestrator. Shares no wgpu state between eyes — each
/// eye has its own history textures and uniform buffer.
pub struct StereoTaauPass {
    left: TaauPass,
    right: TaauPass,
}

impl StereoTaauPass {
    /// Build a stereo pair. `per_eye_output` is the per-eye output
    /// extent; the pair conceptually produces a `2 * per_eye_output`
    /// wide image.
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        per_eye_output: wgpu::Extent3d,
        settings: UpscaleSettings,
    ) -> Self {
        let left = TaauPass::new(device, output_format, per_eye_output, settings);
        let right = TaauPass::new(device, output_format, per_eye_output, settings);
        Self { left, right }
    }

    pub fn resize(&mut self, device: &wgpu::Device, per_eye_output: wgpu::Extent3d) {
        self.left.resize(device, per_eye_output);
        self.right.resize(device, per_eye_output);
    }

    pub fn set_settings(&mut self, settings: UpscaleSettings) {
        self.left.set_settings(settings);
        self.right.set_settings(settings);
    }

    pub fn settings(&self) -> UpscaleSettings {
        // Both eyes share the same settings by construction.
        self.left.settings()
    }

    /// Shared internal extent (both eyes are identical).
    pub fn per_eye_internal_extent(&self) -> wgpu::Extent3d {
        self.left.internal_extent()
    }

    pub fn per_eye_output_extent(&self) -> wgpu::Extent3d {
        self.left.output_extent()
    }

    /// Jitter (in low-res pixel units, [-0.5, 0.5]) for each eye this
    /// frame. Returned as `[left, right]`. Feed these to your scene
    /// pass's projection matrix.
    pub fn jitter_for_frame(&self, frame_index: u64) -> [[f32; 2]; 2] {
        [
            self.left.jitter_for_frame(frame_index),
            self.right
                .jitter_for_frame(frame_index.wrapping_add(RIGHT_EYE_JITTER_OFFSET)),
        ]
    }

    /// View that receives **this** frame's left-eye upscale output. Sample
    /// this in your present pass.
    pub fn left_output(&self, frame_index: u64) -> &wgpu::TextureView {
        self.left.output_view(frame_index)
    }

    pub fn right_output(&self, frame_index: u64) -> &wgpu::TextureView {
        self.right
            .output_view(frame_index.wrapping_add(RIGHT_EYE_JITTER_OFFSET))
    }

    /// Encode both TAAU passes. Writes to
    /// `left_history[frame & 1]` and `right_history[(frame+8) & 1]`,
    /// which by construction has the same parity.
    pub fn execute(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        frame_index: u64,
        left: &EyeTarget<'_>,
        right: &EyeTarget<'_>,
    ) {
        let left_in = TaauInputs {
            lowres_color_view: left.color_view,
            lowres_depth_view: left.depth_view,
            lowres_motion_view: left.motion_view,
        };
        let right_in = TaauInputs {
            lowres_color_view: right.color_view,
            lowres_depth_view: right.depth_view,
            lowres_motion_view: right.motion_view,
        };
        self.left
            .execute(device, queue, encoder, frame_index, &left_in);
        self.right.execute(
            device,
            queue,
            encoder,
            frame_index.wrapping_add(RIGHT_EYE_JITTER_OFFSET),
            &right_in,
        );
    }
}

// ---------------------------------------------------------------------------
// Tests (CPU-only; no wgpu device required)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirrors the TaauPass Halton implementation so we can assert on
    /// the public surface without a device.
    fn halton(mut i: u32, base: u32) -> f32 {
        let mut f = 1.0f32;
        let mut r = 0.0f32;
        while i > 0 {
            f /= base as f32;
            r += f * (i % base) as f32;
            i /= base;
        }
        r
    }

    fn halton_pair(frame: u64) -> [f32; 2] {
        let i = (frame % 16) as u32 + 1;
        [halton(i, 2) - 0.5, halton(i, 3) - 0.5]
    }

    #[test]
    fn decorrelated_jitter() {
        // For every frame in a full Halton window, left and right must differ.
        for frame in 0u64..32 {
            let left = halton_pair(frame);
            let right = halton_pair(frame + RIGHT_EYE_JITTER_OFFSET);
            let d = ((left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2)).sqrt();
            assert!(
                d > 1e-3,
                "frame {}: left {:?} and right {:?} are too close",
                frame,
                left,
                right
            );
        }
    }

    #[test]
    fn jitter_offset_preserves_history_parity() {
        // Both eyes must land on the same history-texture slot each frame so
        // that the ping-pong invariant `write[frame & 1]` holds consistently.
        for frame in 0u64..32 {
            let left_parity = frame & 1;
            let right_parity = (frame + RIGHT_EYE_JITTER_OFFSET) & 1;
            assert_eq!(
                left_parity, right_parity,
                "frame {} eyes on different parity",
                frame
            );
        }
    }

    #[test]
    fn halton_spans_both_halves() {
        // Over 16 frames, Halton should produce samples in both
        // [-0.5, 0) and [0, 0.5) on each axis — otherwise the TAAU
        // coverage is one-sided.
        let mut seen_neg_x = false;
        let mut seen_pos_x = false;
        let mut seen_neg_y = false;
        let mut seen_pos_y = false;
        for frame in 0u64..16 {
            let v = halton_pair(frame);
            if v[0] < 0.0 {
                seen_neg_x = true;
            } else {
                seen_pos_x = true;
            }
            if v[1] < 0.0 {
                seen_neg_y = true;
            } else {
                seen_pos_y = true;
            }
        }
        assert!(seen_neg_x && seen_pos_x, "X not covering both halves");
        assert!(seen_neg_y && seen_pos_y, "Y not covering both halves");
    }

    #[test]
    fn right_eye_offset_constant_is_even_and_nonzero() {
        assert_ne!(RIGHT_EYE_JITTER_OFFSET, 0);
        assert_eq!(
            RIGHT_EYE_JITTER_OFFSET & 1,
            0,
            "offset must be even to preserve history parity"
        );
    }
}
