use ce_math::{Mat4, Quat, Vec3};

/// Which eye to render for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Eye {
    Left,
    Right,
}

/// Configuration for the XR session.
pub struct XrConfig {
    /// Application name shown in the VR runtime.
    pub app_name: String,
    /// Target render resolution per eye (width, height). 0 = use runtime recommended.
    pub render_resolution: (u32, u32),
    /// Enable foveated rendering if supported.
    pub foveated_rendering: bool,
    /// Blend mode (opaque for VR, alpha blend for AR).
    pub blend_mode: BlendMode,
}

impl Default for XrConfig {
    fn default() -> Self {
        Self {
            app_name: "ChemEngine".to_string(),
            render_resolution: (0, 0),
            foveated_rendering: true,
            blend_mode: BlendMode::Opaque,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Opaque,     // VR: fully immersive
    AlphaBlend, // AR: see-through
    Additive,   // AR: additive overlay
}

/// Pose of a single eye/view in world space.
#[derive(Debug, Clone, Copy)]
pub struct ViewPose {
    pub eye: Eye,
    pub position: Vec3,
    pub orientation: Quat,
    pub fov: Fov,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
}

/// Field of view angles in radians.
#[derive(Debug, Clone, Copy)]
pub struct Fov {
    pub angle_left: f32,
    pub angle_right: f32,
    pub angle_up: f32,
    pub angle_down: f32,
}

impl Fov {
    /// Create a symmetric FOV (typical for desktop fallback).
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        let hh = horizontal * 0.5;
        let hv = vertical * 0.5;
        Self {
            angle_left: -hh,
            angle_right: hh,
            angle_up: hv,
            angle_down: -hv,
        }
    }

    /// Convert asymmetric FOV to a projection matrix.
    pub fn to_projection_matrix(&self, near: f32, far: f32) -> Mat4 {
        let left = near * self.angle_left.tan();
        let right = near * self.angle_right.tan();
        let down = near * self.angle_down.tan();
        let up = near * self.angle_up.tan();

        let width = right - left;
        let height = up - down;

        Mat4::from_cols(
            ce_math::Vec4::new(2.0 * near / width, 0.0, 0.0, 0.0),
            ce_math::Vec4::new(0.0, 2.0 * near / height, 0.0, 0.0),
            ce_math::Vec4::new(
                (right + left) / width,
                (up + down) / height,
                -(far + near) / (far - near),
                -1.0,
            ),
            ce_math::Vec4::new(0.0, 0.0, -2.0 * far * near / (far - near), 0.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fov_symmetric() {
        let fov = Fov::symmetric(1.57, 1.57);
        assert!((fov.angle_left + fov.angle_right).abs() < 1e-6);
        assert!((fov.angle_up + fov.angle_down).abs() < 1e-6);
    }

    #[test]
    fn fov_to_projection_produces_valid_matrix() {
        let fov = Fov::symmetric(1.57, 1.57);
        let proj = fov.to_projection_matrix(0.01, 1000.0);
        // Projection matrix should have negative z-component for perspective
        assert!(proj.w_axis.z < 0.0);
    }

    #[test]
    fn xr_config_default() {
        let config = XrConfig::default();
        assert_eq!(config.app_name, "ChemEngine");
        assert_eq!(config.render_resolution, (0, 0));
        assert!(config.foveated_rendering);
    }
}
