// Scene MRT shader — draws a cube at low resolution, outputs:
//   location(0): color (Rgba16Float)
//   location(1): motion vector (Rg16Float) = (cur_ndc - prev_ndc) * 0.5
//   depth      : Depth32Float attachment
//
// Per-frame we upload two uniform matrices: the jittered-current MVP and
// the non-jittered previous MVP. Motion vectors are the difference of
// projected positions in NDC; the un-jittered versions are used so that
// per-frame jitter doesn't leak into the motion buffer.

struct Uniforms {
    cur_mvp_jittered:   mat4x4<f32>,   // used for rasterization position
    cur_mvp_unjittered: mat4x4<f32>,   // used for motion numerator
    prev_mvp_unjittered: mat4x4<f32>,  // used for motion denominator
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VsIn {
    @location(0) pos:   vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) cur_clip_unj:  vec4<f32>,
    @location(2) prev_clip_unj: vec4<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let p = vec4<f32>(in.pos, 1.0);

    out.clip_pos      = u.cur_mvp_jittered   * p;
    out.cur_clip_unj  = u.cur_mvp_unjittered * p;
    out.prev_clip_unj = u.prev_mvp_unjittered * p;
    out.color = in.color;
    return out;
}

struct FsOut {
    @location(0) color:  vec4<f32>,
    @location(1) motion: vec4<f32>,
};

@fragment
fn fs_main(in: VsOut) -> FsOut {
    var out: FsOut;
    out.color = vec4<f32>(in.color, 1.0);

    // Motion vector in NDC-XY space, scaled by 0.5 to match the [-0.5,0.5]
    // range that our shader convention expects (caller/TAAU shader is aware).
    let cur  = in.cur_clip_unj.xy  / max(in.cur_clip_unj.w,  1e-6);
    let prev = in.prev_clip_unj.xy / max(in.prev_clip_unj.w, 1e-6);
    let mv = (cur - prev) * 0.5;
    out.motion = vec4<f32>(mv, 0.0, 0.0);
    return out;
}
