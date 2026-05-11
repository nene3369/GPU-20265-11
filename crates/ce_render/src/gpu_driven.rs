/// A single indirect draw command (matches `wgpu::DrawIndirectArgs` layout).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndirectCommand {
    pub vertex_count: u32,
    pub instance_count: u32, // 0 = culled, 1 = visible
    pub first_vertex: u32,
    pub first_instance: u32,
}

/// A single indexed indirect draw command.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedIndirectCommand {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

/// Per-object data for GPU culling and drawing.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectData {
    /// Model matrix (4x4 f32, column-major).
    pub model: [[f32; 4]; 4],
    /// AABB min point (world space).
    pub aabb_min: [f32; 3],
    pub _pad0: f32,
    /// AABB max point (world space).
    pub aabb_max: [f32; 3],
    pub _pad1: f32,
    /// Index into the mesh/material table.
    pub mesh_id: u32,
    /// Vertex count for this object's mesh.
    pub vertex_count: u32,
    /// First vertex offset in the global vertex buffer.
    pub first_vertex: u32,
    pub _pad2: u32,
}

/// Manages the GPU-side draw list for indirect rendering.
///
/// Flow:
/// 1. CPU uploads `ObjectData[]` for all objects
/// 2. GPU compute shader culls -> writes `DrawIndirectCommand[]` (instance_count = 0 or 1)
/// 3. Render pass uses `multi_draw_indirect` to draw all visible objects
pub struct GpuDrawList {
    /// All object data (uploaded to GPU each frame).
    pub objects: Vec<ObjectData>,
    /// Pre-generated indirect commands (one per object).
    pub commands: Vec<DrawIndirectCommand>,
    /// Number of visible objects after culling.
    pub visible_count: u32,
}

impl GpuDrawList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            commands: Vec::new(),
            visible_count: 0,
        }
    }

    /// Create with pre-allocated capacity (avoids reallocation during frame).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            objects: Vec::with_capacity(capacity),
            commands: Vec::with_capacity(capacity),
            visible_count: 0,
        }
    }

    /// Add an object to the draw list. Returns its index.
    pub fn add_object(&mut self, data: ObjectData) -> u32 {
        let idx = self.objects.len() as u32;
        // Pre-populate the indirect command (visible by default until culling runs)
        self.commands.push(DrawIndirectCommand {
            vertex_count: data.vertex_count,
            instance_count: 1, // visible
            first_vertex: data.first_vertex,
            first_instance: idx,
        });
        self.objects.push(data);
        idx
    }

    /// Clear the draw list for a new frame.
    pub fn clear(&mut self) {
        self.objects.clear();
        self.commands.clear();
        self.visible_count = 0;
    }

    /// Total number of objects in the list.
    pub fn object_count(&self) -> u32 {
        self.objects.len() as u32
    }

    /// Get the indirect commands as raw bytes (for GPU upload).
    pub fn commands_as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.commands)
    }

    /// Get the object data as raw bytes (for GPU upload).
    pub fn objects_as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.objects)
    }

    /// After GPU culling, count visible (instance_count > 0) commands.
    pub fn count_visible(&self) -> u32 {
        self.commands
            .iter()
            .filter(|c| c.instance_count > 0)
            .count() as u32
    }

    /// CPU-side frustum cull (fallback when GPU cull is unavailable).
    /// Sets `instance_count = 0` for culled objects.
    pub fn cpu_frustum_cull(&mut self, frustum_planes: &[[f32; 4]; 6]) {
        self.visible_count = 0;
        for (i, obj) in self.objects.iter().enumerate() {
            let visible = aabb_in_frustum(obj.aabb_min, obj.aabb_max, frustum_planes);
            self.commands[i].instance_count = if visible { 1 } else { 0 };
            if visible {
                self.visible_count += 1;
            }
        }
    }
}

impl Default for GpuDrawList {
    fn default() -> Self {
        Self::new()
    }
}

/// Test if an AABB is inside a frustum (6 planes, each `[nx, ny, nz, d]`).
/// Uses the "positive vertex" test: if the most-positive vertex is outside
/// any plane, the AABB is fully outside.
pub fn aabb_in_frustum(aabb_min: [f32; 3], aabb_max: [f32; 3], planes: &[[f32; 4]; 6]) -> bool {
    for plane in planes {
        // Find the positive vertex (the corner most aligned with the plane normal)
        let px = if plane[0] >= 0.0 {
            aabb_max[0]
        } else {
            aabb_min[0]
        };
        let py = if plane[1] >= 0.0 {
            aabb_max[1]
        } else {
            aabb_min[1]
        };
        let pz = if plane[2] >= 0.0 {
            aabb_max[2]
        } else {
            aabb_min[2]
        };

        // Dot product with plane
        let dist = plane[0] * px + plane[1] * py + plane[2] * pz + plane[3];
        if dist < 0.0 {
            return false; // Fully outside this plane
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    /// Build a standard perspective frustum (6 planes) looking down -Z.
    ///
    /// Planes: Left, Right, Bottom, Top, Near, Far.
    /// Each plane is `[nx, ny, nz, d]` with the convention that
    /// `nx*x + ny*y + nz*z + d >= 0` means "inside".
    fn standard_frustum() -> [[f32; 4]; 6] {
        // A simple symmetric frustum:
        //   near = 0.1, far = 100.0
        //   half-angle ~ 45 deg => normal components = 1/sqrt(2)
        let s = 1.0_f32 / 2.0_f32.sqrt(); // ~0.7071

        [
            // Left plane: normal points right-inward
            [s, 0.0, -s, 0.0],
            // Right plane: normal points left-inward
            [-s, 0.0, -s, 0.0],
            // Bottom plane: normal points up-inward
            [0.0, s, -s, 0.0],
            // Top plane: normal points down-inward
            [0.0, -s, -s, 0.0],
            // Near plane: z >= -near  =>  normal = (0,0,-1), d = -0.1
            [0.0, 0.0, -1.0, -0.1],
            // Far plane: z <= -far   =>  normal = (0,0,1), d = 100.0
            [0.0, 0.0, 1.0, 100.0],
        ]
    }

    fn make_object(aabb_min: [f32; 3], aabb_max: [f32; 3]) -> ObjectData {
        ObjectData {
            aabb_min,
            aabb_max,
            vertex_count: 3,
            first_vertex: 0,
            ..Default::default()
        }
    }

    // 1. draw_list_add_and_count
    #[test]
    fn draw_list_add_and_count() {
        let mut list = GpuDrawList::new();
        assert_eq!(list.object_count(), 0);

        list.add_object(ObjectData::default());
        list.add_object(ObjectData::default());
        list.add_object(ObjectData::default());

        assert_eq!(list.object_count(), 3);
        assert_eq!(list.commands.len(), 3);
    }

    // 2. draw_list_clear
    #[test]
    fn draw_list_clear() {
        let mut list = GpuDrawList::new();
        list.add_object(ObjectData::default());
        list.add_object(ObjectData::default());
        list.visible_count = 2;

        list.clear();

        assert_eq!(list.object_count(), 0);
        assert_eq!(list.commands.len(), 0);
        assert_eq!(list.visible_count, 0);
    }

    // 3. commands_as_bytes_layout -- 16 bytes per DrawIndirectCommand
    #[test]
    fn commands_as_bytes_layout() {
        assert_eq!(mem::size_of::<DrawIndirectCommand>(), 16);

        let mut list = GpuDrawList::new();
        let obj = ObjectData {
            vertex_count: 36,
            first_vertex: 100,
            ..Default::default()
        };
        list.add_object(obj);

        let bytes = list.commands_as_bytes();
        assert_eq!(bytes.len(), 16); // one command = 16 bytes

        // Verify the raw u32 values via bytemuck
        let cmd: &DrawIndirectCommand = bytemuck::from_bytes(&bytes[..16]);
        assert_eq!(cmd.vertex_count, 36);
        assert_eq!(cmd.instance_count, 1);
        assert_eq!(cmd.first_vertex, 100);
        assert_eq!(cmd.first_instance, 0);
    }

    // 4. object_data_layout -- 96 bytes
    #[test]
    fn object_data_layout() {
        // 4x4 matrix = 64 bytes
        // aabb_min (12) + pad (4) = 16
        // aabb_max (12) + pad (4) = 16
        // mesh_id + vertex_count + first_vertex + pad = 16
        // Total = 64 + 16 + 16 + 16 = 112
        //
        // Wait -- let's just assert whatever the actual repr(C) size is,
        // since the layout is dictated by the struct definition.
        let expected = 64 + 16 + 16 + 16; // 112
        assert_eq!(mem::size_of::<ObjectData>(), expected);
    }

    // 5. cpu_frustum_cull_visible -- object inside frustum stays visible
    #[test]
    fn cpu_frustum_cull_visible() {
        let frustum = standard_frustum();
        let mut list = GpuDrawList::new();

        // Object at z=-5, well inside the frustum
        let obj = make_object([-1.0, -1.0, -6.0], [1.0, 1.0, -4.0]);
        list.add_object(obj);

        list.cpu_frustum_cull(&frustum);

        assert_eq!(list.commands[0].instance_count, 1);
        assert_eq!(list.visible_count, 1);
    }

    // 6. cpu_frustum_cull_outside -- object fully outside is culled
    #[test]
    fn cpu_frustum_cull_outside() {
        let frustum = standard_frustum();
        let mut list = GpuDrawList::new();

        // Object at z=+10, behind the camera (in front of near plane, wrong side)
        let obj = make_object([-1.0, -1.0, 9.0], [1.0, 1.0, 11.0]);
        list.add_object(obj);

        list.cpu_frustum_cull(&frustum);

        assert_eq!(list.commands[0].instance_count, 0);
        assert_eq!(list.visible_count, 0);
    }

    // 7. cpu_frustum_cull_partial -- object partially inside stays visible
    #[test]
    fn cpu_frustum_cull_partial() {
        let frustum = standard_frustum();
        let mut list = GpuDrawList::new();

        // Object straddles the near plane (z from +0.05 to -0.5)
        // The positive-vertex test with the near plane [0,0,-1,-0.1]:
        // p-vertex z = max(0.05, -0.5) when normal_z < 0 => aabb_min_z = -0.5 (wait, let's think)
        // near plane normal = [0,0,-1], so plane[2] = -1.0 < 0 => pz = aabb_min[2] = -0.5
        // dist = 0*px + 0*py + (-1)*(-0.5) + (-0.1) = 0.5 - 0.1 = 0.4 >= 0 => inside
        //
        // Object spans from z=-0.5 to z=0.05, partially behind near plane.
        // The positive-vertex test will keep it visible (conservative).
        let obj = make_object([-0.1, -0.1, -0.5], [0.1, 0.1, 0.05]);
        list.add_object(obj);

        list.cpu_frustum_cull(&frustum);

        assert_eq!(list.commands[0].instance_count, 1);
        assert_eq!(list.visible_count, 1);
    }

    // 8. aabb_in_frustum_all_inside -- centered object visible in standard frustum
    #[test]
    fn aabb_in_frustum_all_inside() {
        let frustum = standard_frustum();
        // Small box centered at (0, 0, -5) -- deep inside frustum
        assert!(aabb_in_frustum(
            [-0.5, -0.5, -5.5],
            [0.5, 0.5, -4.5],
            &frustum,
        ));
    }

    // 9. aabb_in_frustum_behind_camera -- object behind camera is culled
    #[test]
    fn aabb_in_frustum_behind_camera() {
        let frustum = standard_frustum();
        // Object at z=+5, completely behind the camera
        assert!(!aabb_in_frustum(
            [-1.0, -1.0, 4.0],
            [1.0, 1.0, 6.0],
            &frustum,
        ));
    }

    // 10. count_visible_after_cull
    #[test]
    fn count_visible_after_cull() {
        let frustum = standard_frustum();
        let mut list = GpuDrawList::new();

        // 3 objects inside, 2 outside
        list.add_object(make_object([-1.0, -1.0, -3.0], [1.0, 1.0, -1.0])); // inside
        list.add_object(make_object([-1.0, -1.0, -10.0], [1.0, 1.0, -8.0])); // inside
        list.add_object(make_object([50.0, 50.0, 4.0], [52.0, 52.0, 6.0])); // behind camera
        list.add_object(make_object([-1.0, -1.0, -50.0], [1.0, 1.0, -48.0])); // inside
        list.add_object(make_object([200.0, 0.0, -5.0], [202.0, 2.0, -3.0])); // far right, outside

        list.cpu_frustum_cull(&frustum);

        assert_eq!(list.visible_count, 3);
        assert_eq!(list.count_visible(), 3);
    }
}
