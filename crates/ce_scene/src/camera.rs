use ce_math::{Mat4, Vec3};

/// Camera projection type.
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Perspective {
        fov: f32,    // vertical field of view in radians
        aspect: f32, // width / height
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

impl Projection {
    /// Standard perspective camera.
    pub fn perspective(fov_degrees: f32, aspect: f32, near: f32, far: f32) -> Self {
        Projection::Perspective {
            fov: fov_degrees.to_radians(),
            aspect,
            near,
            far,
        }
    }

    /// Compute the projection matrix.
    pub fn matrix(&self) -> Mat4 {
        match self {
            Projection::Perspective {
                fov,
                aspect,
                near,
                far,
            } => Mat4::perspective_rh(*fov, *aspect, *near, *far),
            Projection::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => Mat4::orthographic_rh(*left, *right, *bottom, *top, *near, *far),
        }
    }
}

/// Camera component. Attach to an entity with a Transform.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub projection: Projection,
    pub clear_color: [f32; 4],
    /// Render order (lower = renders first). For multi-camera setups.
    pub order: i32,
    pub active: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: Projection::perspective(60.0, 16.0 / 9.0, 0.1, 1000.0),
            clear_color: [0.1, 0.1, 0.1, 1.0],
            order: 0,
            active: true,
        }
    }
}

impl Camera {
    /// Compute the view-projection matrix from this camera and its transform.
    pub fn view_projection(&self, transform: &super::transform::Transform) -> Mat4 {
        let view = transform.matrix().inverse();
        self.projection.matrix() * view
    }
}

/// Convenience: a camera with a transform.
pub struct CameraBundle {
    pub camera: Camera,
    pub transform: super::transform::Transform,
    pub global_transform: super::transform::GlobalTransform,
}

impl Default for CameraBundle {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            transform: super::transform::Transform::from_translation(Vec3::new(0.0, 5.0, 10.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            global_transform: super::transform::GlobalTransform::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::Transform;

    #[test]
    fn default_camera_has_perspective_projection() {
        let cam = Camera::default();
        match cam.projection {
            Projection::Perspective {
                fov,
                aspect,
                near,
                far,
            } => {
                // 60 degrees in radians
                let expected_fov = 60.0_f32.to_radians();
                assert!((fov - expected_fov).abs() < 1e-5);
                assert!((aspect - 16.0 / 9.0).abs() < 1e-5);
                assert!((near - 0.1).abs() < 1e-5);
                assert!((far - 1000.0).abs() < 1e-3);
            }
            _ => panic!("Expected Perspective projection"),
        }
        assert!(cam.active);
        assert_eq!(cam.order, 0);
    }

    #[test]
    fn perspective_matrix_has_correct_near_plane() {
        let proj = Projection::perspective(60.0, 1.0, 0.1, 100.0);
        let m = proj.matrix();
        // For a right-handed perspective matrix, the [2][2] element should be negative
        // and the [2][3] element should be negative (related to near/far).
        assert!(
            m.z_axis.z < 0.0,
            "z_axis.z should be negative for RH perspective"
        );
        assert!(m.z_axis.w < 0.0, "z_axis.w should be -1 for RH perspective");
    }

    #[test]
    fn perspective_matrix_is_not_identity() {
        let proj = Projection::perspective(90.0, 1.0, 0.1, 100.0);
        let m = proj.matrix();
        assert!(!m.abs_diff_eq(Mat4::IDENTITY, 1e-6));
    }

    #[test]
    fn orthographic_projection_creates_valid_matrix() {
        let proj = Projection::Orthographic {
            left: -10.0,
            right: 10.0,
            bottom: -10.0,
            top: 10.0,
            near: 0.1,
            far: 100.0,
        };
        let m = proj.matrix();
        // Orthographic matrix should have 1/(right-left) type scaling in x_axis.x
        let expected_x = 2.0 / 20.0; // 2 / (right - left)
        assert!((m.x_axis.x - expected_x).abs() < 1e-5);
        let expected_y = 2.0 / 20.0; // 2 / (top - bottom)
        assert!((m.y_axis.y - expected_y).abs() < 1e-5);
        // w_axis.w should be 1.0 for orthographic
        assert!((m.w_axis.w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn view_projection_combines_view_and_projection() {
        let cam = Camera::default();
        let t = Transform::from_translation(Vec3::new(0.0, 5.0, 10.0));
        let vp = cam.view_projection(&t);

        // The VP matrix should not be identity (camera is offset from origin)
        assert!(!vp.abs_diff_eq(Mat4::IDENTITY, 1e-6));

        // Check that VP is indeed proj * view
        let view = t.matrix().inverse();
        let proj = cam.projection.matrix();
        let expected = proj * view;
        assert!(
            vp.abs_diff_eq(expected, 1e-4),
            "view_projection should equal projection * inverse(transform)"
        );
    }

    #[test]
    fn camera_bundle_default() {
        let bundle = CameraBundle::default();
        assert!(bundle.camera.active);
        // Camera should be positioned at (0, 5, 10)
        assert!((bundle.transform.translation.x).abs() < 1e-5);
        assert!((bundle.transform.translation.y - 5.0).abs() < 1e-5);
        assert!((bundle.transform.translation.z - 10.0).abs() < 1e-5);
    }

    #[test]
    fn camera_clear_color_default() {
        let cam = Camera::default();
        assert!((cam.clear_color[0] - 0.1).abs() < 1e-5);
        assert!((cam.clear_color[1] - 0.1).abs() < 1e-5);
        assert!((cam.clear_color[2] - 0.1).abs() < 1e-5);
        assert!((cam.clear_color[3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn projection_perspective_constructor() {
        let proj = Projection::perspective(45.0, 1.5, 0.5, 500.0);
        match proj {
            Projection::Perspective {
                fov,
                aspect,
                near,
                far,
            } => {
                assert!((fov - 45.0_f32.to_radians()).abs() < 1e-5);
                assert!((aspect - 1.5).abs() < 1e-5);
                assert!((near - 0.5).abs() < 1e-5);
                assert!((far - 500.0).abs() < 1e-3);
            }
            _ => panic!("Expected Perspective"),
        }
    }
}
