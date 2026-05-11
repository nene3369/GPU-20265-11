// GPU frustum culling compute shader.
// Tests each AABB against 6 frustum planes.
// Writes 1 (visible) or 0 (culled) to result buffer.

struct Aabb {
    min: vec3<f32>,
    _pad0: f32,
    max: vec3<f32>,
    _pad1: f32,
};

struct Frustum {
    planes: array<vec4<f32>, 6>,
};

struct CullResult {
    visible: u32,
};

struct DrawIndirectCommand {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
};

@group(0) @binding(0) var<storage, read> aabbs: array<Aabb>;
@group(0) @binding(1) var<uniform> frustum: Frustum;
@group(0) @binding(2) var<storage, read_write> results: array<CullResult>;
@group(0) @binding(3) var<storage, read_write> draw_commands: array<DrawIndirectCommand>;

@compute @workgroup_size(256)
fn cs_cull(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&aabbs)) {
        return;
    }

    let aabb = aabbs[i];
    var visible = true;

    for (var p = 0u; p < 6u; p = p + 1u) {
        let plane = frustum.planes[p];

        // Positive vertex test
        var pv: vec3<f32>;
        pv.x = select(aabb.min.x, aabb.max.x, plane.x >= 0.0);
        pv.y = select(aabb.min.y, aabb.max.y, plane.y >= 0.0);
        pv.z = select(aabb.min.z, aabb.max.z, plane.z >= 0.0);

        let dist = dot(plane.xyz, pv) + plane.w;
        if (dist < 0.0) {
            visible = false;
            break;
        }
    }

    let v = select(0u, 1u, visible);
    results[i].visible = v;

    // Also update the indirect draw command's instance_count
    draw_commands[i].instance_count = v;
}
