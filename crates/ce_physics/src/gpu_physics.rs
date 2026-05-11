//! GPU-accelerated physics integration.
//!
//! This module provides data structures that mirror the WGSL shader layout
//! for GPU compute-based physics integration. It also includes a CPU fallback
//! that performs the same semi-implicit Euler integration for testing and
//! platforms without GPU support.

/// GPU-side body data (matches WGSL `Body` struct layout).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuBody {
    pub pos: [f32; 3],
    pub mass: f32,
    pub vel: [f32; 3],
    pub damping: f32,
    pub body_type: u32, // 0=Static, 1=Dynamic, 2=Kinematic
    pub _pad: [u32; 3],
}

/// GPU physics parameters (matches WGSL `PhysicsParams` struct).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuPhysicsParams {
    pub gravity: [f32; 3],
    pub dt: f32,
}

/// Manages GPU-side physics computation.
pub struct GpuPhysics {
    /// The WGSL shader source.
    pub shader_source: &'static str,
    /// Number of bodies.
    pub body_count: u32,
}

impl GpuPhysics {
    pub fn new() -> Self {
        Self {
            shader_source: include_str!("shaders/physics_integrate.wgsl"),
            body_count: 0,
        }
    }

    /// Convert CPU-side physics data to GPU format.
    pub fn bodies_to_gpu(
        positions: &[[f32; 3]],
        velocities: &[[f32; 3]],
        masses: &[f32],
        dampings: &[f32],
        body_types: &[u32],
    ) -> Vec<GpuBody> {
        let len = positions.len();
        let mut bodies = Vec::with_capacity(len);
        for i in 0..len {
            bodies.push(GpuBody {
                pos: positions[i],
                vel: velocities[i],
                mass: masses[i],
                damping: dampings[i],
                body_type: body_types[i],
                _pad: [0; 3],
            });
        }
        bodies
    }

    /// Convert GPU bodies back to separate position arrays.
    pub fn gpu_to_positions(bodies: &[GpuBody]) -> Vec<[f32; 3]> {
        bodies.iter().map(|b| b.pos).collect()
    }

    /// Convert GPU bodies back to separate velocity arrays.
    pub fn gpu_to_velocities(bodies: &[GpuBody]) -> Vec<[f32; 3]> {
        bodies.iter().map(|b| b.vel).collect()
    }

    /// Threshold above which parallel execution is faster than serial.
    const PARALLEL_THRESHOLD: usize = 50_000;

    /// Smart integration: auto-selects serial or parallel based on body count.
    pub fn integrate_auto(bodies: &mut [GpuBody], params: &GpuPhysicsParams) {
        if bodies.len() >= Self::PARALLEL_THRESHOLD {
            Self::integrate_cpu_parallel(bodies, params);
        } else {
            Self::integrate_cpu(bodies, params);
        }
    }

    /// CPU fallback: run the same integration on CPU (single-threaded).
    pub fn integrate_cpu(bodies: &mut [GpuBody], params: &GpuPhysicsParams) {
        for b in bodies.iter_mut() {
            Self::integrate_one(b, params);
        }
    }

    /// CPU parallel: multi-threaded via rayon. Use for 10K+ bodies.
    pub fn integrate_cpu_parallel(bodies: &mut [GpuBody], params: &GpuPhysicsParams) {
        use rayon::prelude::*;
        bodies.par_iter_mut().for_each(|b| {
            Self::integrate_one(b, params);
        });
    }

    /// Integrate a single body (shared by serial and parallel paths).
    #[inline(always)]
    fn integrate_one(b: &mut GpuBody, params: &GpuPhysicsParams) {
        if b.body_type != 1 {
            return;
        }

        b.vel[0] += params.gravity[0] * params.dt;
        b.vel[1] += params.gravity[1] * params.dt;
        b.vel[2] += params.gravity[2] * params.dt;

        let damp = 1.0 - b.damping;
        b.vel[0] *= damp;
        b.vel[1] *= damp;
        b.vel[2] *= damp;

        b.pos[0] += b.vel[0] * params.dt;
        b.pos[1] += b.vel[1] * params.dt;
        b.pos[2] += b.vel[2] * params.dt;
    }

    /// Compute workgroup count for N bodies.
    pub fn workgroup_count(body_count: u32) -> u32 {
        body_count.div_ceil(256)
    }
}

impl Default for GpuPhysics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn gpu_body_size() {
        assert_eq!(
            std::mem::size_of::<GpuBody>(),
            48,
            "GpuBody must be 48 bytes for GPU alignment"
        );
    }

    #[test]
    fn gpu_params_size() {
        assert_eq!(
            std::mem::size_of::<GpuPhysicsParams>(),
            16,
            "GpuPhysicsParams must be 16 bytes"
        );
    }

    #[test]
    fn bodies_to_gpu_roundtrip() {
        let positions = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let velocities = vec![[0.1, 0.2, 0.3], [0.4, 0.5, 0.6]];
        let masses = vec![1.0, 2.0];
        let dampings = vec![0.01, 0.02];
        let body_types = vec![1, 0];

        let gpu_bodies =
            GpuPhysics::bodies_to_gpu(&positions, &velocities, &masses, &dampings, &body_types);

        let result_positions = GpuPhysics::gpu_to_positions(&gpu_bodies);
        let result_velocities = GpuPhysics::gpu_to_velocities(&gpu_bodies);

        assert_eq!(result_positions, positions);
        assert_eq!(result_velocities, velocities);
        assert_eq!(gpu_bodies[0].mass, 1.0);
        assert_eq!(gpu_bodies[1].mass, 2.0);
        assert_eq!(gpu_bodies[0].body_type, 1);
        assert_eq!(gpu_bodies[1].body_type, 0);
    }

    #[test]
    fn integrate_cpu_applies_gravity() {
        let params = GpuPhysicsParams {
            gravity: [0.0, -9.81, 0.0],
            dt: 1.0 / 60.0,
        };

        let mut bodies = vec![GpuBody {
            pos: [0.0, 100.0, 0.0],
            mass: 1.0,
            vel: [0.0, 0.0, 0.0],
            damping: 0.0,
            body_type: 1, // Dynamic
            _pad: [0; 3],
        }];

        GpuPhysics::integrate_cpu(&mut bodies, &params);

        // Velocity should be negative (gravity pulling down).
        assert!(
            bodies[0].vel[1] < 0.0,
            "velocity Y should be negative after gravity: {}",
            bodies[0].vel[1]
        );

        // Position should have moved down.
        assert!(
            bodies[0].pos[1] < 100.0,
            "position Y should be below 100: {}",
            bodies[0].pos[1]
        );
    }

    #[test]
    fn integrate_cpu_static_not_moved() {
        let params = GpuPhysicsParams {
            gravity: [0.0, -9.81, 0.0],
            dt: 1.0 / 60.0,
        };

        let mut bodies = vec![GpuBody {
            pos: [5.0, 10.0, 0.0],
            mass: 1.0,
            vel: [0.0, 0.0, 0.0],
            damping: 0.0,
            body_type: 0, // Static
            _pad: [0; 3],
        }];

        GpuPhysics::integrate_cpu(&mut bodies, &params);

        assert_eq!(bodies[0].pos, [5.0, 10.0, 0.0], "static body must not move");
        assert_eq!(
            bodies[0].vel,
            [0.0, 0.0, 0.0],
            "static body velocity must remain zero"
        );
    }

    #[test]
    fn integrate_cpu_100k_bodies() {
        let params = GpuPhysicsParams {
            gravity: [0.0, -9.81, 0.0],
            dt: 1.0 / 60.0,
        };

        let count = 100_000;
        let mut bodies: Vec<GpuBody> = (0..count)
            .map(|i| GpuBody {
                pos: [i as f32 * 0.1, 100.0, 0.0],
                mass: 1.0,
                vel: [0.0; 3],
                damping: 0.01,
                body_type: 1, // Dynamic
                _pad: [0; 3],
            })
            .collect();

        let start = Instant::now();
        GpuPhysics::integrate_cpu(&mut bodies, &params);
        let elapsed = start.elapsed();

        println!("CPU fallback: {} bodies integrated in {:?}", count, elapsed);

        // Sanity check: all bodies should have fallen.
        for b in &bodies {
            assert!(b.pos[1] < 100.0, "body should have moved down");
        }
    }

    #[test]
    fn workgroup_count_correct() {
        assert_eq!(GpuPhysics::workgroup_count(1000), 4);
        assert_eq!(GpuPhysics::workgroup_count(256), 1);
        assert_eq!(GpuPhysics::workgroup_count(257), 2);
        assert_eq!(GpuPhysics::workgroup_count(1), 1);
        assert_eq!(GpuPhysics::workgroup_count(0), 0);
    }
}
