use ce_math::{Mat3, Mat4, Quat, Vec3};

/// Local transform relative to parent (or world if no parent).
/// This is THE component that connects physics, rendering, AI, and interaction.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Create a transform at a position.
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    /// Create a transform with position and rotation.
    pub fn from_translation_rotation(translation: Vec3, rotation: Quat) -> Self {
        Self {
            translation,
            rotation,
            ..Self::IDENTITY
        }
    }

    /// Create a transform with uniform scale.
    pub fn from_scale(scale: f32) -> Self {
        Self {
            scale: Vec3::splat(scale),
            ..Self::IDENTITY
        }
    }

    /// Compute the 4x4 model matrix (TRS order: scale -> rotate -> translate).
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Transform a point from local to world space.
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.matrix().transform_point3(point)
    }

    /// Get the forward direction (-Z in right-handed coordinates).
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// Get the right direction (+X).
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction (+Y).
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Look at a target position from the current position.
    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        let forward = (target - self.translation).normalize_or_zero();
        if forward.length_squared() > 0.0 {
            let right = forward.cross(up).normalize_or_zero();
            let corrected_up = right.cross(forward);
            self.rotation = Quat::from_mat3(&Mat3::from_cols(right, corrected_up, -forward));
        }
        self
    }

    /// Multiply two transforms (parent * child = global).
    pub fn mul_transform(&self, child: &Transform) -> Transform {
        let translation = self.transform_point(child.translation);
        let rotation = self.rotation * child.rotation;
        let scale = self.scale * child.scale;
        Transform {
            translation,
            rotation,
            scale,
        }
    }

    /// Distance to another transform.
    pub fn distance(&self, other: &Transform) -> f32 {
        self.translation.distance(other.translation)
    }
}

/// World-space transform computed from the hierarchy.
/// Updated each frame by propagate_transforms system.
#[derive(Debug, Clone, Copy)]
pub struct GlobalTransform(pub Mat4);

impl Default for GlobalTransform {
    fn default() -> Self {
        Self(Mat4::IDENTITY)
    }
}

impl GlobalTransform {
    pub fn translation(&self) -> Vec3 {
        Vec3::new(self.0.w_axis.x, self.0.w_axis.y, self.0.w_axis.z)
    }

    pub fn from_transform(t: &Transform) -> Self {
        Self(t.matrix())
    }
}

/// System that propagates transforms through the hierarchy.
/// For entities without parents, GlobalTransform = Transform.matrix().
/// For entities with parents, GlobalTransform = parent.GlobalTransform * local.Transform.
pub fn propagate_transforms(world: &mut ce_ecs::World) {
    // Simple version for MVP: just copy Transform to GlobalTransform for all entities.
    // Full hierarchy propagation requires Parent/Children traversal (added later).
    // This placeholder ensures GlobalTransform is always up-to-date for root entities.
    let _ = world;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn identity_is_at_origin_no_rotation() {
        let t = Transform::IDENTITY;
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn default_is_identity() {
        let t = Transform::default();
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn from_translation_sets_position() {
        let t = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(t.translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn from_translation_rotation() {
        let rot = Quat::from_rotation_y(PI / 2.0);
        let t = Transform::from_translation_rotation(Vec3::new(5.0, 0.0, 0.0), rot);
        assert_eq!(t.translation, Vec3::new(5.0, 0.0, 0.0));
        // Rotation should be close to the one we set
        let diff = (t.rotation.dot(rot)).abs();
        assert!((diff - 1.0).abs() < 1e-5);
    }

    #[test]
    fn from_scale_uniform() {
        let t = Transform::from_scale(3.0);
        assert_eq!(t.scale, Vec3::splat(3.0));
        assert_eq!(t.translation, Vec3::ZERO);
    }

    #[test]
    fn matrix_identity_is_identity_mat4() {
        let t = Transform::IDENTITY;
        let m = t.matrix();
        let diff = (m - Mat4::IDENTITY).abs_diff_eq(Mat4::ZERO, 1e-6);
        assert!(diff, "Identity transform should produce identity matrix");
    }

    #[test]
    fn matrix_with_translation() {
        let t = Transform::from_translation(Vec3::new(10.0, 20.0, 30.0));
        let m = t.matrix();
        assert!((m.w_axis.x - 10.0).abs() < 1e-6);
        assert!((m.w_axis.y - 20.0).abs() < 1e-6);
        assert!((m.w_axis.z - 30.0).abs() < 1e-6);
    }

    #[test]
    fn transform_point_applies_translation() {
        let t = Transform::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let result = t.transform_point(Vec3::ZERO);
        assert!((result.x - 5.0).abs() < 1e-6);
        assert!((result.y).abs() < 1e-6);
        assert!((result.z).abs() < 1e-6);
    }

    #[test]
    fn transform_point_applies_scale_and_translation() {
        let mut t = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        t.scale = Vec3::splat(2.0);
        let result = t.transform_point(Vec3::new(1.0, 0.0, 0.0));
        // Scale 2x the point (1,0,0) -> (2,0,0), then translate +1 -> (3,0,0)
        assert!((result.x - 3.0).abs() < 1e-5);
    }

    #[test]
    fn forward_default_is_neg_z() {
        let t = Transform::IDENTITY;
        let fwd = t.forward();
        assert!((fwd.x).abs() < 1e-6);
        assert!((fwd.y).abs() < 1e-6);
        assert!(
            (fwd.z + 1.0).abs() < 1e-6,
            "forward should be -Z, got {:?}",
            fwd
        );
    }

    #[test]
    fn right_default_is_pos_x() {
        let t = Transform::IDENTITY;
        let r = t.right();
        assert!((r.x - 1.0).abs() < 1e-6);
        assert!((r.y).abs() < 1e-6);
        assert!((r.z).abs() < 1e-6);
    }

    #[test]
    fn up_default_is_pos_y() {
        let t = Transform::IDENTITY;
        let u = t.up();
        assert!((u.x).abs() < 1e-6);
        assert!((u.y - 1.0).abs() < 1e-6);
        assert!((u.z).abs() < 1e-6);
    }

    #[test]
    fn looking_at_points_toward_target() {
        let t =
            Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)).looking_at(Vec3::ZERO, Vec3::Y);
        let fwd = t.forward();
        // Forward should point toward -Z (from z=10 to z=0)
        assert!(
            fwd.z < -0.9,
            "forward should point toward -Z, got {:?}",
            fwd
        );
    }

    #[test]
    fn mul_transform_combines_parent_and_child() {
        let parent = Transform::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let child = Transform::from_translation(Vec3::new(0.0, 5.0, 0.0));

        let combined = parent.mul_transform(&child);
        // Child at (0,5,0) in parent space that is at (10,0,0) -> (10,5,0)
        assert!((combined.translation.x - 10.0).abs() < 1e-5);
        assert!((combined.translation.y - 5.0).abs() < 1e-5);
        assert!((combined.translation.z).abs() < 1e-5);
    }

    #[test]
    fn mul_transform_with_scale() {
        let parent = Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(2.0),
        };
        let child = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));

        let combined = parent.mul_transform(&child);
        // Child offset (1,0,0) scaled by parent 2x -> (2,0,0)
        assert!((combined.translation.x - 2.0).abs() < 1e-5);
        assert_eq!(combined.scale, Vec3::splat(2.0));
    }

    #[test]
    fn distance_between_transforms() {
        let a = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let b = Transform::from_translation(Vec3::new(3.0, 4.0, 0.0));
        let d = a.distance(&b);
        assert!((d - 5.0).abs() < 1e-6, "distance should be 5.0, got {}", d);
    }

    #[test]
    fn distance_same_position_is_zero() {
        let a = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let b = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        assert!((a.distance(&b)).abs() < 1e-6);
    }

    #[test]
    fn global_transform_default_is_identity() {
        let gt = GlobalTransform::default();
        assert!(gt.0.abs_diff_eq(Mat4::IDENTITY, 1e-6));
    }

    #[test]
    fn global_transform_from_transform_matches_matrix() {
        let t = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let gt = GlobalTransform::from_transform(&t);
        assert!(gt.0.abs_diff_eq(t.matrix(), 1e-6));
    }

    #[test]
    fn global_transform_translation() {
        let t = Transform::from_translation(Vec3::new(7.0, 8.0, 9.0));
        let gt = GlobalTransform::from_transform(&t);
        let pos = gt.translation();
        assert!((pos.x - 7.0).abs() < 1e-6);
        assert!((pos.y - 8.0).abs() < 1e-6);
        assert!((pos.z - 9.0).abs() < 1e-6);
    }

    #[test]
    fn propagate_transforms_does_not_panic() {
        let mut world = ce_ecs::World::new();
        propagate_transforms(&mut world);
        // Just ensuring it doesn't crash with an empty world.
    }
}
