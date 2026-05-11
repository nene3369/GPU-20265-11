use ce_math::{Aabb, Vec3};

/// Collider component. Defines the shape used for collision detection.
#[derive(Debug, Clone)]
pub struct Collider {
    pub shape: ColliderShape,
    pub is_trigger: bool, // If true, detects overlap but doesn't resolve
}

impl Collider {
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Sphere { radius },
            is_trigger: false,
        }
    }

    pub fn box_shape(half_extents: Vec3) -> Self {
        Self {
            shape: ColliderShape::Box { half_extents },
            is_trigger: false,
        }
    }

    pub fn capsule(radius: f32, half_height: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule {
                radius,
                half_height,
            },
            is_trigger: false,
        }
    }

    /// Compute AABB for this collider at a given position.
    pub fn aabb(&self, position: Vec3) -> Aabb {
        match &self.shape {
            ColliderShape::Sphere { radius } => {
                let r = Vec3::splat(*radius);
                Aabb::new(position - r, position + r)
            }
            ColliderShape::Box { half_extents } => {
                Aabb::new(position - *half_extents, position + *half_extents)
            }
            ColliderShape::Capsule {
                radius,
                half_height,
            } => {
                let r = Vec3::new(*radius, *half_height + *radius, *radius);
                Aabb::new(position - r, position + r)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColliderShape {
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
    Capsule { radius: f32, half_height: f32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_aabb_computation() {
        let collider = Collider::sphere(2.0);
        let aabb = collider.aabb(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.min, Vec3::new(-1.0, 0.0, 1.0));
        assert_eq!(aabb.max, Vec3::new(3.0, 4.0, 5.0));
    }

    #[test]
    fn box_aabb_computation() {
        let collider = Collider::box_shape(Vec3::new(1.0, 2.0, 3.0));
        let aabb = collider.aabb(Vec3::new(5.0, 5.0, 5.0));
        assert_eq!(aabb.min, Vec3::new(4.0, 3.0, 2.0));
        assert_eq!(aabb.max, Vec3::new(6.0, 7.0, 8.0));
    }

    #[test]
    fn capsule_aabb_computation() {
        let collider = Collider::capsule(1.0, 2.0);
        let aabb = collider.aabb(Vec3::ZERO);
        // half_height + radius = 3.0 for y, radius = 1.0 for x and z
        assert_eq!(aabb.min, Vec3::new(-1.0, -3.0, -1.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 3.0, 1.0));
    }
}
