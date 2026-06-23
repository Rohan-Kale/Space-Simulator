use glam::Vec3;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]

pub struct TrailVertex {
    pub pos: [f32; 3],
    pub alpha: f32,
}

pub struct Trail {
    pub points: Vec<(Vec3, f32)>,
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
}

impl Trail {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Trail Buffer"),
            size: 10000,
            usage: wgpu::BufferUsages::VERTEX |
                   wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            points: Vec::new(),
            vertex_buffer,
            num_vertices: 0,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, position: Vec3) {
        // add new point
        self.points.push((position, 1.0));

        // fade old points
        for point in &mut self.points {
            point.1 -= 0.003;
        }

        // remove invisible points
        self.points.retain(|p| p.1 > 0.0);

        // convert to GPU vertices
        let vertices: Vec<TrailVertex> =
            self.points
            .iter()
            .map(|(pos, alpha)| {
                TrailVertex {
                    pos: [pos.x, pos.y, pos.z],
                    alpha: *alpha,
                }
            })
            .collect();

        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices)
        );

        self.num_vertices = vertices.len() as u32;
    }
}
