pub mod aabb;
pub mod ray;

// Re-export commonly used glam types for convenience.
pub use glam::{IVec2, IVec3, Mat3, Mat4, Quat, UVec2, UVec3, Vec2, Vec3, Vec4};

// Re-export game-engine-specific types.
pub use aabb::Aabb;
pub use ray::Ray;
