use glam::Vec3;

/// Axis-Aligned Bounding Box defined by minimum and maximum corners.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// Create a new AABB from min and max corners.
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB from a center point and half-extents.
    pub fn from_center_half_extents(center: Vec3, half: Vec3) -> Self {
        Self {
            min: center - half,
            max: center + half,
        }
    }

    /// Return the center point of the AABB.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Return the half-extents (half the size along each axis).
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Check whether a point lies inside (or on the boundary of) this AABB.
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Check whether this AABB overlaps with another AABB.
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Return the union of two AABBs (the smallest AABB enclosing both).
    pub fn merge(&self, other: &Aabb) -> Aabb {
        Aabb {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Compute the surface area of the AABB.
    pub fn surface_area(&self) -> f32 {
        let d = self.max - self.min;
        2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let aabb = Aabb::new(Vec3::new(-1.0, -2.0, -3.0), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_from_center_half_extents() {
        let aabb =
            Aabb::from_center_half_extents(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.5, 1.0, 1.5));
        assert_eq!(aabb.min, Vec3::new(0.5, 1.0, 1.5));
        assert_eq!(aabb.max, Vec3::new(1.5, 3.0, 4.5));
    }

    #[test]
    fn test_center() {
        let aabb = Aabb::new(Vec3::new(-1.0, -2.0, -3.0), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.center(), Vec3::ZERO);
    }

    #[test]
    fn test_center_offset() {
        let aabb = Aabb::new(Vec3::new(2.0, 4.0, 6.0), Vec3::new(4.0, 8.0, 10.0));
        assert_eq!(aabb.center(), Vec3::new(3.0, 6.0, 8.0));
    }

    #[test]
    fn test_half_extents() {
        let aabb = Aabb::new(Vec3::new(-1.0, -2.0, -3.0), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.half_extents(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_contains_point_inside() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        assert!(aabb.contains_point(Vec3::new(1.0, 1.0, 1.0)));
    }

    #[test]
    fn test_contains_point_on_boundary() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        assert!(aabb.contains_point(Vec3::new(0.0, 0.0, 0.0)));
        assert!(aabb.contains_point(Vec3::new(2.0, 2.0, 2.0)));
    }

    #[test]
    fn test_contains_point_outside() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        assert!(!aabb.contains_point(Vec3::new(3.0, 1.0, 1.0)));
        assert!(!aabb.contains_point(Vec3::new(-0.1, 1.0, 1.0)));
        assert!(!aabb.contains_point(Vec3::new(1.0, 1.0, 2.1)));
    }

    #[test]
    fn test_intersects_overlap() {
        let a = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        let b = Aabb::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(3.0, 3.0, 3.0));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn test_intersects_touching() {
        let a = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        let b = Aabb::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(2.0, 1.0, 1.0));
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_intersects_separated() {
        let a = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        let b = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));
    }

    #[test]
    fn test_intersects_one_axis_separated() {
        let a = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        let b = Aabb::new(Vec3::new(0.0, 0.0, 1.5), Vec3::new(1.0, 1.0, 2.5));
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_merge() {
        let a = Aabb::new(Vec3::new(-1.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let b = Aabb::new(Vec3::new(0.0, -1.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
        let merged = a.merge(&b);
        assert_eq!(merged.min, Vec3::new(-1.0, -1.0, 0.0));
        assert_eq!(merged.max, Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_merge_contained() {
        let outer = Aabb::new(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
        let inner = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let merged = outer.merge(&inner);
        assert_eq!(merged.min, outer.min);
        assert_eq!(merged.max, outer.max);
    }

    #[test]
    fn test_surface_area_unit_cube() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        assert!((aabb.surface_area() - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_surface_area_rectangular() {
        // 2x3x4 box: 2*(2*3 + 3*4 + 4*2) = 2*(6+12+8) = 52
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0));
        assert!((aabb.surface_area() - 52.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_roundtrip_center_half_extents() {
        let center = Vec3::new(5.0, 10.0, -3.0);
        let half = Vec3::new(2.0, 3.0, 4.0);
        let aabb = Aabb::from_center_half_extents(center, half);
        let diff_center = (aabb.center() - center).length();
        let diff_half = (aabb.half_extents() - half).length();
        assert!(diff_center < f32::EPSILON);
        assert!(diff_half < f32::EPSILON);
    }
}
