/// AABB data for GPU culling (matches WGSL struct layout).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuAabb {
    pub min: [f32; 3],
    pub _pad0: f32,
    pub max: [f32; 3],
    pub _pad1: f32,
}

/// Frustum planes for GPU culling (6 planes, each [nx, ny, nz, d]).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuFrustum {
    pub planes: [[f32; 4]; 6],
}

/// Result of GPU culling: visibility flags (1 = visible, 0 = culled).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CullResult {
    pub visible: u32,
}

/// Manages GPU culling resources and pipeline.
pub struct GpuCullPipeline {
    /// The WGSL shader source (embedded).
    pub shader_source: &'static str,
    /// Number of objects to cull.
    pub object_count: u32,
}

impl GpuCullPipeline {
    pub fn new() -> Self {
        Self {
            shader_source: include_str!("shaders/gpu_cull_aabb.wgsl"),
            object_count: 0,
        }
    }

    /// Compute the frustum planes from a view-projection matrix.
    /// Extracts 6 clip planes from the VP matrix (Gribb/Hartmann method).
    pub fn extract_frustum_planes(vp: &[[f32; 4]; 4]) -> GpuFrustum {
        // VP matrix is column-major: vp[col][row]
        // Access as row: row_i = [vp[0][i], vp[1][i], vp[2][i], vp[3][i]]
        let row = |i: usize| -> [f32; 4] { [vp[0][i], vp[1][i], vp[2][i], vp[3][i]] };

        let r0 = row(0);
        let r1 = row(1);
        let r2 = row(2);
        let r3 = row(3);

        let mut planes = [[0.0f32; 4]; 6];

        // Left:   r3 + r0
        // Right:  r3 - r0
        // Bottom: r3 + r1
        // Top:    r3 - r1
        // Near:   r3 + r2
        // Far:    r3 - r2
        for i in 0..4 {
            planes[0][i] = r3[i] + r0[i]; // Left
            planes[1][i] = r3[i] - r0[i]; // Right
            planes[2][i] = r3[i] + r1[i]; // Bottom
            planes[3][i] = r3[i] - r1[i]; // Top
            planes[4][i] = r3[i] + r2[i]; // Near
            planes[5][i] = r3[i] - r2[i]; // Far
        }

        // Normalize each plane
        for plane in &mut planes {
            let len = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
            if len > 1e-6 {
                for v in plane.iter_mut() {
                    *v /= len;
                }
            }
        }

        GpuFrustum { planes }
    }

    /// Perform CPU-side AABB frustum culling (fallback).
    pub fn cull_cpu(frustum: &GpuFrustum, aabbs: &[GpuAabb]) -> Vec<CullResult> {
        aabbs
            .iter()
            .map(|aabb| {
                let mut visible = true;
                for plane in &frustum.planes {
                    let px = if plane[0] >= 0.0 {
                        aabb.max[0]
                    } else {
                        aabb.min[0]
                    };
                    let py = if plane[1] >= 0.0 {
                        aabb.max[1]
                    } else {
                        aabb.min[1]
                    };
                    let pz = if plane[2] >= 0.0 {
                        aabb.max[2]
                    } else {
                        aabb.min[2]
                    };
                    let dist = plane[0] * px + plane[1] * py + plane[2] * pz + plane[3];
                    if dist < 0.0 {
                        visible = false;
                        break;
                    }
                }
                CullResult {
                    visible: if visible { 1 } else { 0 },
                }
            })
            .collect()
    }
}

impl Default for GpuCullPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn gpu_aabb_size() {
        assert_eq!(mem::size_of::<GpuAabb>(), 32);
    }

    #[test]
    fn gpu_frustum_size() {
        assert_eq!(mem::size_of::<GpuFrustum>(), 96);
    }

    #[test]
    fn cull_result_size() {
        assert_eq!(mem::size_of::<CullResult>(), 4);
    }

    #[test]
    fn extract_frustum_from_identity() {
        let identity: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let frustum = GpuCullPipeline::extract_frustum_planes(&identity);
        // All 6 planes should have been extracted and normalized
        for plane in &frustum.planes {
            let len = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
            // Normal part should have non-zero length (plane is valid)
            assert!(len > 0.5, "Plane normal length too small: {len}");
        }
    }

    #[test]
    fn extract_frustum_planes_normalized() {
        let vp: [[f32; 4]; 4] = [
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 3.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let frustum = GpuCullPipeline::extract_frustum_planes(&vp);
        for plane in &frustum.planes {
            let len = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-5,
                "Plane normal not normalized: len={len}"
            );
        }
    }

    #[test]
    fn cull_cpu_object_at_origin_visible() {
        let identity: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let frustum = GpuCullPipeline::extract_frustum_planes(&identity);
        let aabb = GpuAabb {
            min: [-0.5, -0.5, -0.5],
            _pad0: 0.0,
            max: [0.5, 0.5, 0.5],
            _pad1: 0.0,
        };
        let results = GpuCullPipeline::cull_cpu(&frustum, &[aabb]);
        assert_eq!(results[0].visible, 1, "Object at origin should be visible");
    }

    #[test]
    fn cull_cpu_object_far_away_culled() {
        let vp = make_perspective_vp();
        let frustum = GpuCullPipeline::extract_frustum_planes(&vp);
        let aabb = GpuAabb {
            min: [-0.5, -0.5, -1000.5],
            _pad0: 0.0,
            max: [0.5, 0.5, -999.5],
            _pad1: 0.0,
        };
        let results = GpuCullPipeline::cull_cpu(&frustum, &[aabb]);
        assert_eq!(
            results[0].visible, 0,
            "Object at z=-1000 should be culled by far plane"
        );
    }

    #[test]
    fn cull_cpu_object_behind_camera_culled() {
        let vp = make_perspective_vp();
        let frustum = GpuCullPipeline::extract_frustum_planes(&vp);
        let aabb = GpuAabb {
            min: [-0.5, -0.5, 9.5],
            _pad0: 0.0,
            max: [0.5, 0.5, 10.5],
            _pad1: 0.0,
        };
        let results = GpuCullPipeline::cull_cpu(&frustum, &[aabb]);
        assert_eq!(
            results[0].visible, 0,
            "Object behind camera (z=+10) should be culled"
        );
    }

    #[test]
    fn cull_cpu_many_objects() {
        let vp = make_perspective_vp();
        let frustum = GpuCullPipeline::extract_frustum_planes(&vp);

        let mut aabbs = Vec::with_capacity(1000);
        for i in 0..1000 {
            let z = -0.5 - (i as f32) * 0.1; // Objects stretching in -Z direction
            aabbs.push(GpuAabb {
                min: [-0.1, -0.1, z - 0.05],
                _pad0: 0.0,
                max: [0.1, 0.1, z + 0.05],
                _pad1: 0.0,
            });
        }

        let results = GpuCullPipeline::cull_cpu(&frustum, &aabbs);
        assert_eq!(results.len(), 1000);

        let visible_count = results.iter().filter(|r| r.visible == 1).count();
        let culled_count = results.iter().filter(|r| r.visible == 0).count();

        // Some should be visible (near the camera), some should be culled (beyond far plane)
        assert!(visible_count > 0, "Some objects should be visible");
        assert!(
            culled_count > 0,
            "Some objects should be culled beyond far plane"
        );
        assert_eq!(visible_count + culled_count, 1000);
    }

    #[test]
    fn extract_frustum_from_perspective() {
        let vp = make_perspective_vp();
        let frustum = GpuCullPipeline::extract_frustum_planes(&vp);

        // Object inside the frustum (in front of camera, within bounds)
        let inside = GpuAabb {
            min: [-0.5, -0.5, -5.5],
            _pad0: 0.0,
            max: [0.5, 0.5, -4.5],
            _pad1: 0.0,
        };
        let results = GpuCullPipeline::cull_cpu(&frustum, &[inside]);
        assert_eq!(
            results[0].visible, 1,
            "Object inside perspective frustum should be visible"
        );

        // Object far outside to the right
        let outside_right = GpuAabb {
            min: [500.0, -0.5, -5.5],
            _pad0: 0.0,
            max: [501.0, 0.5, -4.5],
            _pad1: 0.0,
        };
        let results = GpuCullPipeline::cull_cpu(&frustum, &[outside_right]);
        assert_eq!(
            results[0].visible, 0,
            "Object far to the right should be culled"
        );
    }

    /// Simple perspective: fov=90, aspect=1, near=0.1, far=100
    fn make_perspective_vp() -> [[f32; 4]; 4] {
        let f = 1.0; // 1/tan(fov/2) for 90 degrees
        [
            [f, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, -100.1 / 99.9, -1.0],
            [0.0, 0.0, -2.0 * 100.0 * 0.1 / 99.9, 0.0],
        ]
    }
}
