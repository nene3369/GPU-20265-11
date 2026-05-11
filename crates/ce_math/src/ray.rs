use glam::Vec3;

use crate::aabb::Aabb;

/// A ray defined by an origin point and a normalized direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3, // should be normalized
}

impl Ray {
    /// Create a new ray. The direction is normalized internally.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Return the point along the ray at parameter `t`.
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Test intersection with an AABB using the slab method.
    ///
    /// Returns `Some(t)` with the nearest positive intersection distance,
    /// or `None` if the ray does not hit the AABB (or hits behind the origin).
    pub fn intersects_aabb(&self, aabb: &Aabb) -> Option<f32> {
        let inv_dir = Vec3::new(
            1.0 / self.direction.x,
            1.0 / self.direction.y,
            1.0 / self.direction.z,
        );

        let t1 = (aabb.min.x - self.origin.x) * inv_dir.x;
        let t2 = (aabb.max.x - self.origin.x) * inv_dir.x;
        let t3 = (aabb.min.y - self.origin.y) * inv_dir.y;
        let t4 = (aabb.max.y - self.origin.y) * inv_dir.y;
        let t5 = (aabb.min.z - self.origin.z) * inv_dir.z;
        let t6 = (aabb.max.z - self.origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        // If tmax < 0, the AABB is behind the ray origin.
        if tmax < 0.0 {
            return None;
        }

        // If tmin > tmax, the ray misses the AABB.
        if tmin > tmax {
            return None;
        }

        // If tmin < 0, the ray origin is inside the AABB; return tmax.
        if tmin < 0.0 {
            Some(tmax)
        } else {
            Some(tmin)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_normalizes_direction() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0));
        let diff = (ray.direction.length() - 1.0).abs();
        assert!(diff < 1e-6);
        assert!((ray.direction - Vec3::X).length() < 1e-6);
    }

    #[test]
    fn test_at() {
        let ray = Ray::new(Vec3::new(1.0, 2.0, 3.0), Vec3::X);
        let point = ray.at(5.0);
        assert!((point - Vec3::new(6.0, 2.0, 3.0)).length() < 1e-6);
    }

    #[test]
    fn test_at_zero() {
        let ray = Ray::new(Vec3::new(1.0, 2.0, 3.0), Vec3::Y);
        let point = ray.at(0.0);
        assert!((point - ray.origin).length() < 1e-6);
    }

    #[test]
    fn test_intersects_aabb_hit_from_front() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::X);
        let t = ray.intersects_aabb(&aabb);
        assert!(t.is_some());
        let t = t.unwrap();
        assert!((t - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_intersects_aabb_hit_from_y() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::NEG_Y);
        let t = ray.intersects_aabb(&aabb);
        assert!(t.is_some());
        let t = t.unwrap();
        assert!((t - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_intersects_aabb_miss() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3::new(-5.0, 5.0, 0.0), Vec3::X);
        let t = ray.intersects_aabb(&aabb);
        assert!(t.is_none());
    }

    #[test]
    fn test_intersects_aabb_behind_origin() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::X);
        let t = ray.intersects_aabb(&aabb);
        assert!(t.is_none());
    }

    #[test]
    fn test_intersects_aabb_origin_inside() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let t = ray.intersects_aabb(&aabb);
        assert!(t.is_some());
        let t = t.unwrap();
        // Origin at center, shooting +X, exits at x=1.0
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_intersects_aabb_diagonal() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        let ray = Ray::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let t = ray.intersects_aabb(&aabb);
        assert!(t.is_some());
        // The ray hits the corner at the origin; t = distance from (-1,-1,-1) to (0,0,0)
        let expected = Vec3::new(1.0, 1.0, 1.0).length();
        assert!((t.unwrap() - expected).abs() < 1e-4);
    }

    #[test]
    fn test_hit_point_lies_on_surface() {
        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::X);
        let t = ray.intersects_aabb(&aabb).unwrap();
        let hit = ray.at(t);
        // Hit point should be on the x=-1 face
        assert!((hit.x - (-1.0)).abs() < 1e-5);
        assert!(aabb.contains_point(hit));
    }
}
