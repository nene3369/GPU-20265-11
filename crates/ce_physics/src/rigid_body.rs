use ce_math::Vec3;

/// Rigid body component. Attach to an entity with a Transform.
#[derive(Debug, Clone, Copy)]
pub struct RigidBody {
    pub body_type: BodyType,
    pub mass: f32,
    pub restitution: f32, // Bounciness (0.0 - 1.0)
    pub friction: f32,    // Surface friction (0.0 - 1.0)
    pub linear_damping: f32,
    pub angular_damping: f32,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: BodyType::Dynamic,
            mass: 1.0,
            restitution: 0.3,
            friction: 0.5,
            linear_damping: 0.01,
            angular_damping: 0.01,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    Static,    // Never moves (ground, walls)
    Dynamic,   // Affected by forces and gravity
    Kinematic, // Moves but not affected by forces (platforms)
}

/// Velocity component (linear + angular).
#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

/// Physics material properties.
#[derive(Debug, Clone, Copy)]
pub struct PhysicsMaterial {
    pub restitution: f32,
    pub friction: f32,
    pub density: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            restitution: 0.3,
            friction: 0.5,
            density: 1.0,
        }
    }
}
