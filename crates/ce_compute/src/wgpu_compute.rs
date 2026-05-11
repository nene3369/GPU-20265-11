//! wgpu-based compute backend for GPU shader dispatch.

use crate::backend::{BackendKind, ComputeCapabilities, ComputeEngine};

/// Wraps a wgpu device and queue for compute shader dispatch.
pub struct WgpuComputeBackend {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl WgpuComputeBackend {
    /// Creates a new wgpu compute backend from an existing device and queue.
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self { device, queue }
    }

    /// Creates a GPU buffer initialized with the given data.
    pub fn create_buffer(&self, data: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ce_compute buffer"),
                contents: data,
                usage,
            })
    }

    /// Dispatches a compute pipeline with the given bind group and workgroup counts.
    pub fn dispatch_compute(
        &self,
        pipeline: &wgpu::ComputePipeline,
        bind_group: &wgpu::BindGroup,
        workgroups: (u32, u32, u32),
    ) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ce_compute dispatch encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ce_compute pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Reads the contents of a GPU buffer back to the CPU.
    ///
    /// Creates a staging buffer, copies the data, maps it, and returns the bytes.
    pub fn read_buffer(&self, buffer: &wgpu::Buffer, size: u64) -> Vec<u8> {
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ce_compute staging buffer"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ce_compute read encoder"),
            });
        encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, size);
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = staging.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        receiver.recv().unwrap().unwrap();

        let data = slice.get_mapped_range().to_vec();
        staging.unmap();
        data
    }
}

impl ComputeEngine for WgpuComputeBackend {
    fn capabilities(&self) -> ComputeCapabilities {
        let limits = self.device.limits();
        ComputeCapabilities {
            backend: BackendKind::Wgpu,
            max_workgroup_size: limits.max_compute_workgroup_size_x,
            max_buffer_size: limits.max_buffer_size,
        }
    }

    fn backend_name(&self) -> &str {
        "wgpu"
    }
}
