// TAAU — Temporal Anti-Aliasing Upscaling shader.
//
// Takes a jittered low-resolution render (color + depth + motion) and a
// previous-frame history buffer at output resolution, produces a new
// output-resolution frame that is temporally stable.
//
// Algorithm summary per output pixel:
//   1. Un-jitter sample current low-res color at (uv - jitter*inv_internal)
//   2. Find closest 3x3 depth neighbor -> pick its motion vector (edge fix)
//   3. Reproject: prev_uv = uv + motion (motion is "from prev to cur", sign flipped)
//   4. Sample previous history at prev_uv
//   5. Neighborhood-clamp previous color to YCoCg AABB of 3x3 current
//   6. Blend with motion-weighted alpha; first frame -> alpha=1
//   7. CAS-lite sharpen
//
// All work happens on the fragment shader of a single fullscreen triangle.

struct TaauParams {
    inv_internal_size: vec2<f32>,   // 1.0 / internal_extent
    inv_output_size:   vec2<f32>,   // 1.0 / output_extent
    jitter:            vec2<f32>,   // current frame jitter in low-res pixels
    sharpness:         f32,         // 0..1
    history_valid:     u32,         // 0 on first frame / after resize
};

@group(0) @binding(0) var lowres_color:  texture_2d<f32>;
@group(0) @binding(1) var lowres_depth:  texture_depth_2d;
@group(0) @binding(2) var lowres_motion: texture_2d<f32>;
@group(0) @binding(3) var prev_color:    texture_2d<f32>;
@group(0) @binding(4) var bilinear_sampler: sampler;
@group(0) @binding(5) var<uniform> params: TaauParams;

// ---------- Vertex: fullscreen triangle via vertex_index -------------------

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vid: u32) -> VsOut {
    // 3 vertices covering the screen: (-1,-1), (3,-1), (-1,3)
    var out: VsOut;
    let x = f32((vid << 1u) & 2u);   // 0, 2, 0
    let y = f32(vid & 2u);           // 0, 0, 2
    out.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv  = vec2<f32>(x, y);
    return out;
}

// ---------- Color space helpers --------------------------------------------

fn rgb_to_ycocg(c: vec3<f32>) -> vec3<f32> {
    let y  =  0.25 * c.r + 0.5 * c.g + 0.25 * c.b;
    let co =  0.5  * c.r              - 0.5  * c.b;
    let cg = -0.25 * c.r + 0.5 * c.g - 0.25 * c.b;
    return vec3<f32>(y, co, cg);
}

fn ycocg_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let r = c.x + c.y - c.z;
    let g = c.x        + c.z;
    let b = c.x - c.y - c.z;
    return vec3<f32>(r, g, b);
}

// ---------- Fragment -------------------------------------------------------

@fragment
fn fs_taau(in: VsOut) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // --- Un-jitter: subtract the sub-pixel offset used during scene render ---
    // jitter is in low-res pixels; inv_internal_size * jitter = UV offset.
    let unjittered_uv = uv - params.jitter * params.inv_internal_size;

    // --- 1. Current color (bilinear on low-res, un-jittered) -------------
    let current = textureSampleLevel(lowres_color, bilinear_sampler, unjittered_uv, 0.0).rgb;

    // --- 2. Closest-depth neighbor for motion vector selection -----------
    // Scan a 3x3 in low-res depth; pick the pixel with the nearest depth
    // (largest value for reverse-Z; here we assume standard [0,1] depth, so
    // nearest = smallest). Use its motion vector.
    var best_offset = vec2<i32>(0, 0);
    var best_depth = 2.0;
    let lr_tex_size = vec2<f32>(textureDimensions(lowres_depth, 0));
    let lr_coord = vec2<i32>(unjittered_uv * lr_tex_size);

    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let c = clamp(lr_coord + vec2<i32>(dx, dy),
                          vec2<i32>(0, 0),
                          vec2<i32>(lr_tex_size) - vec2<i32>(1, 1));
            let d = textureLoad(lowres_depth, c, 0);
            if (d < best_depth) {
                best_depth = d;
                best_offset = vec2<i32>(dx, dy);
            }
        }
    }

    let motion_coord = clamp(lr_coord + best_offset,
                             vec2<i32>(0, 0),
                             vec2<i32>(lr_tex_size) - vec2<i32>(1, 1));
    let motion = textureLoad(lowres_motion, motion_coord, 0).xy;
    // motion is (cur_ndc.xy - prev_ndc.xy) * 0.5; to get UV offset in [0,1]
    // space we must flip Y (NDC-Y points up, UV-Y points down).
    let motion_uv = vec2<f32>(motion.x, -motion.y);

    // --- 3. Reproject previous history ----------------------------------
    let prev_uv = uv - motion_uv;
    let prev_in_bounds = all(prev_uv >= vec2<f32>(0.0)) && all(prev_uv <= vec2<f32>(1.0));
    let prev_rgb = textureSampleLevel(prev_color, bilinear_sampler, prev_uv, 0.0).rgb;

    // --- 4. Neighborhood AABB (YCoCg, 3x3 of low-res current) ------------
    var nmin = vec3<f32>( 1e20);
    var nmax = vec3<f32>(-1e20);
    var nsum = vec3<f32>(0.0);
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let c = clamp(lr_coord + vec2<i32>(dx, dy),
                          vec2<i32>(0, 0),
                          vec2<i32>(lr_tex_size) - vec2<i32>(1, 1));
            let s_rgb = textureLoad(lowres_color, c, 0).rgb;
            let s = rgb_to_ycocg(s_rgb);
            nmin = min(nmin, s);
            nmax = max(nmax, s);
            nsum = nsum + s;
        }
    }
    let navg = nsum * (1.0 / 9.0);

    // Shrink the AABB slightly toward the average to reduce banding.
    let shrink = 0.25;
    nmin = mix(nmin, navg, vec3<f32>(shrink));
    nmax = mix(nmax, navg, vec3<f32>(shrink));

    let prev_ycocg = rgb_to_ycocg(prev_rgb);
    let clamped_prev_ycocg = clamp(prev_ycocg, nmin, nmax);

    // --- 5. Blend factor --------------------------------------------------
    // Base: 1/8 = 0.125. This keeps ~88% previous, producing long history
    // reuse for temporal stability.
    var alpha = 0.125;

    // Boost alpha (less history reuse) when motion is large.
    let motion_len = length(motion_uv);
    alpha = alpha + clamp(motion_len * 60.0, 0.0, 0.5);

    // Out-of-bounds: full current.
    if (!prev_in_bounds) {
        alpha = 1.0;
    }

    // First frame / after resize: no history exists.
    if (params.history_valid == 0u) {
        alpha = 1.0;
    }

    // --- 6. Mix -----------------------------------------------------------
    let current_ycocg = rgb_to_ycocg(current);
    let out_ycocg = mix(clamped_prev_ycocg, current_ycocg, vec3<f32>(alpha));
    var result = ycocg_to_rgb(out_ycocg);

    // --- 7. CAS-lite sharpening ------------------------------------------
    // result += sharpness * (result - box3x3_avg(result_neighborhood))
    // We approximate the neighborhood via the low-res box avg already computed.
    let navg_rgb = ycocg_to_rgb(navg);
    let high_freq = result - navg_rgb;
    result = result + params.sharpness * high_freq;

    result = max(result, vec3<f32>(0.0));
    return vec4<f32>(result, 1.0);
}
