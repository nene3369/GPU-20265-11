pub mod collider;
pub mod collision;
pub mod gpu_physics;
pub mod rigid_body;
pub mod spatial;

pub use collider::{Collider, ColliderShape};
pub use collision::{CollisionEvent, Contact};
pub use gpu_physics::{GpuBody, GpuPhysics, GpuPhysicsParams};
pub use rigid_body::{BodyType, PhysicsMaterial, RigidBody, Velocity};
pub use spatial::SpatialGrid;

use ce_app::{App, Plugin};
use ce_ecs::{CoreStage, World};
use ce_math::Vec3;
use ce_scene::Transform;

pub struct PhysicsPlugin {
    pub gravity: Vec3,
    pub fixed_timestep: f32,
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            fixed_timestep: 1.0 / 60.0,
        }
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicsConfig {
            gravity: self.gravity,
            fixed_timestep: self.fixed_timestep,
        });
        app.add_system(CoreStage::FixedUpdate, physics_step);
        log::info!("PhysicsPlugin loaded: gravity={}", self.gravity);
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    pub gravity: Vec3,
    pub fixed_timestep: f32,
}

/// Main physics step system.
/// Semi-implicit Euler: velocity += acceleration * dt, then position += velocity * dt.
fn physics_step(world: &mut World) {
    let config = match world.get_resource::<PhysicsConfig>() {
        Some(c) => c.clone(),
        None => return,
    };
    let dt = config.fixed_timestep;
    let gravity = config.gravity;

    // Phase 1: Collect entity + component data (immutable borrow ends with block).
    let physics_bodies: Vec<(ce_core::Entity, RigidBody, Velocity)> = {
        let results = world.query2::<RigidBody, Velocity>();
        results.iter().map(|&(e, rb, vel)| (e, *rb, *vel)).collect()
    };

    // Phase 2: Mutate world (no outstanding borrows).
    for (entity, rb, mut vel) in physics_bodies {
        if rb.body_type != BodyType::Dynamic {
            continue;
        }

        // 1. Apply gravity.
        vel.linear += gravity * dt;

        // 2. Apply linear damping.
        vel.linear *= 1.0 - rb.linear_damping;

        // 3. Integrate position (semi-implicit Euler).
        if let Some(tf) = world.get_component::<Transform>(entity).copied() {
            let mut new_tf = tf;
            new_tf.translation += vel.linear * dt;
            world.insert_component(entity, new_tf);
        }

        // 4. Write back updated velocity.
        world.insert_component(entity, vel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_step_applies_gravity() {
        let mut world = World::new();
        world.insert_resource(PhysicsConfig {
            gravity: Vec3::new(0.0, -10.0, 0.0),
            fixed_timestep: 1.0 / 60.0,
        });

        let entity = world.spawn();
        world.insert_component(
            entity,
            Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
        );
        world.insert_component(entity, RigidBody::default());
        world.insert_component(entity, Velocity::default());

        // Run one physics step.
        physics_step(&mut world);

        let vel = world.get_component::<Velocity>(entity).unwrap();
        assert!(
            vel.linear.y < 0.0,
            "velocity should be negative after gravity"
        );

        let tf = world.get_component::<Transform>(entity).unwrap();
        assert!(tf.translation.y < 10.0, "position should have moved down");
    }

    #[test]
    fn static_body_does_not_move() {
        let mut world = World::new();
        world.insert_resource(PhysicsConfig {
            gravity: Vec3::new(0.0, -10.0, 0.0),
            fixed_timestep: 1.0 / 60.0,
        });

        let entity = world.spawn();
        world.insert_component(
            entity,
            Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
        );
        world.insert_component(
            entity,
            RigidBody {
                body_type: BodyType::Static,
                ..Default::default()
            },
        );
        world.insert_component(entity, Velocity::default());

        physics_step(&mut world);

        let tf = world.get_component::<Transform>(entity).unwrap();
        assert!(
            (tf.translation.y - 5.0).abs() < 1e-6,
            "static body should not move"
        );
    }

    #[test]
    fn physics_step_with_initial_velocity() {
        let mut world = World::new();
        world.insert_resource(PhysicsConfig {
            gravity: Vec3::ZERO,
            fixed_timestep: 1.0,
        });

        let entity = world.spawn();
        world.insert_component(entity, Transform::from_translation(Vec3::ZERO));
        world.insert_component(entity, RigidBody::default());
        world.insert_component(
            entity,
            Velocity {
                linear: Vec3::new(5.0, 0.0, 0.0),
                angular: Vec3::ZERO,
            },
        );

        physics_step(&mut world);

        let tf = world.get_component::<Transform>(entity).unwrap();
        // With damping 0.01: vel = 5.0 * (1 - 0.01) = 4.95, pos = 4.95 * 1.0
        assert!(
            tf.translation.x > 4.0,
            "should have moved right: {}",
            tf.translation.x
        );
    }

    #[test]
    fn multiple_bodies_simulated() {
        let mut world = World::new();
        world.insert_resource(PhysicsConfig {
            gravity: Vec3::new(0.0, -10.0, 0.0),
            fixed_timestep: 0.1,
        });

        let mut entities = Vec::new();
        for i in 0..10 {
            let e = world.spawn();
            world.insert_component(
                e,
                Transform::from_translation(Vec3::new(i as f32, 100.0, 0.0)),
            );
            world.insert_component(e, RigidBody::default());
            world.insert_component(e, Velocity::default());
            entities.push(e);
        }

        // Run 10 steps.
        for _ in 0..10 {
            physics_step(&mut world);
        }

        for e in &entities {
            let tf = world.get_component::<Transform>(*e).unwrap();
            assert!(tf.translation.y < 100.0, "all bodies should have fallen");
        }
    }
}
