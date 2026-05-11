// GPU Physics Integration — Semi-Implicit Euler
// Processes all rigid bodies in parallel on the GPU.

struct Body {
    // Position (xyz) + padding
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    mass: f32,

    // Velocity (xyz) + damping
    vel_x: f32,
    vel_y: f32,
    vel_z: f32,
    damping: f32,

    // Body type: 0=Static, 1=Dynamic, 2=Kinematic
    body_type: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct PhysicsParams {
    gravity_x: f32,
    gravity_y: f32,
    gravity_z: f32,
    dt: f32,
};

@group(0) @binding(0) var<storage, read_write> bodies: array<Body>;
@group(0) @binding(1) var<uniform> params: PhysicsParams;

@compute @workgroup_size(256)
fn cs_integrate(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&bodies)) {
        return;
    }

    var b = bodies[i];

    // Only Dynamic bodies are affected
    if (b.body_type != 1u) {
        return;
    }

    // Semi-implicit Euler:
    // 1. velocity += gravity * dt
    b.vel_x += params.gravity_x * params.dt;
    b.vel_y += params.gravity_y * params.dt;
    b.vel_z += params.gravity_z * params.dt;

    // 2. Apply damping
    let damp = 1.0 - b.damping;
    b.vel_x *= damp;
    b.vel_y *= damp;
    b.vel_z *= damp;

    // 3. position += velocity * dt
    b.pos_x += b.vel_x * params.dt;
    b.pos_y += b.vel_y * params.dt;
    b.pos_z += b.vel_z * params.dt;

    bodies[i] = b;
}
