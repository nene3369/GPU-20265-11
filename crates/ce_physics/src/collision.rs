use ce_core::Entity;
use ce_math::Vec3;

/// A contact point between two colliders.
#[derive(Debug, Clone, Copy)]
pub struct Contact {
    pub point: Vec3,
    pub normal: Vec3,
    pub penetration: f32,
}

/// Collision event emitted when two entities collide.
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub contacts: Vec<Contact>,
    pub is_trigger: bool,
}

/// Test if two AABBs overlap and compute penetration.
pub fn aabb_overlap(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> Option<Contact> {
    // Check overlap on all 3 axes
    let overlap_x = (a_max.x.min(b_max.x)) - (a_min.x.max(b_min.x));
    let overlap_y = (a_max.y.min(b_max.y)) - (a_min.y.max(b_min.y));
    let overlap_z = (a_max.z.min(b_max.z)) - (a_min.z.max(b_min.z));

    if overlap_x <= 0.0 || overlap_y <= 0.0 || overlap_z <= 0.0 {
        return None;
    }

    // Find minimum penetration axis
    let (normal, penetration) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
        let center_a = (a_min.x + a_max.x) * 0.5;
        let center_b = (b_min.x + b_max.x) * 0.5;
        let dir = if center_a < center_b { -1.0 } else { 1.0 };
        (Vec3::new(dir, 0.0, 0.0), overlap_x)
    } else if overlap_y <= overlap_z {
        let center_a = (a_min.y + a_max.y) * 0.5;
        let center_b = (b_min.y + b_max.y) * 0.5;
        let dir = if center_a < center_b { -1.0 } else { 1.0 };
        (Vec3::new(0.0, dir, 0.0), overlap_y)
    } else {
        let center_a = (a_min.z + a_max.z) * 0.5;
        let center_b = (b_min.z + b_max.z) * 0.5;
        let dir = if center_a < center_b { -1.0 } else { 1.0 };
        (Vec3::new(0.0, 0.0, dir), overlap_z)
    };

    let point = Vec3::new(
        (a_min.x.max(b_min.x) + a_max.x.min(b_max.x)) * 0.5,
        (a_min.y.max(b_min.y) + a_max.y.min(b_max.y)) * 0.5,
        (a_min.z.max(b_min.z) + a_max.z.min(b_max.z)) * 0.5,
    );

    Some(Contact {
        point,
        normal,
        penetration,
    })
}

/// Test sphere-sphere collision.
pub fn sphere_overlap(pos_a: Vec3, radius_a: f32, pos_b: Vec3, radius_b: f32) -> Option<Contact> {
    let diff = pos_b - pos_a;
    let dist_sq = diff.length_squared();
    let combined_radius = radius_a + radius_b;

    if dist_sq >= combined_radius * combined_radius {
        return None;
    }

    let dist = dist_sq.sqrt();
    let normal = if dist > 1e-6 { diff / dist } else { Vec3::Y };
    let penetration = combined_radius - dist;
    let point = pos_a + normal * (radius_a - penetration * 0.5);

    Some(Contact {
        point,
        normal,
        penetration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_overlap_detects_overlap() {
        let result = aabb_overlap(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(3.0, 3.0, 3.0),
        );
        assert!(result.is_some());
    }

    #[test]
    fn aabb_overlap_returns_none_for_separated_boxes() {
        let result = aabb_overlap(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(2.0, 2.0, 2.0),
            Vec3::new(3.0, 3.0, 3.0),
        );
        assert!(result.is_none());
    }

    #[test]
    fn aabb_overlap_computes_correct_normal() {
        // Box A at origin, Box B shifted right so X overlap is smallest
        let result = aabb_overlap(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
            Vec3::new(1.5, 0.5, 0.5),
            Vec3::new(3.5, 2.5, 2.5),
        );
        let contact = result.unwrap();
        // X overlap = 0.5, Y overlap = 1.5, Z overlap = 1.5
        // Minimum penetration axis should be X
        assert!((contact.normal.x.abs() - 1.0).abs() < 1e-6);
        assert!(contact.normal.y.abs() < 1e-6);
        assert!(contact.normal.z.abs() < 1e-6);
        assert!((contact.penetration - 0.5).abs() < 1e-6);
    }

    #[test]
    fn sphere_overlap_detects_overlap() {
        let result = sphere_overlap(Vec3::new(0.0, 0.0, 0.0), 1.5, Vec3::new(2.0, 0.0, 0.0), 1.5);
        assert!(result.is_some());
        let contact = result.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn sphere_overlap_returns_none_for_separated_spheres() {
        let result = sphere_overlap(Vec3::new(0.0, 0.0, 0.0), 1.0, Vec3::new(5.0, 0.0, 0.0), 1.0);
        assert!(result.is_none());
    }
}
