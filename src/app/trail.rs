use glam::Vec3;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]

pub struct TrailVertex {
    pub pos: [f32; 3],
    pub alpha: f32,
}

pub struct Trail {
    pub points: Vec<Vec<(Vec3, f32)>>,
    pub vertex_buffer: wgpu::Buffer,
    pub ranges: Vec<std::ops::Range<u32>>,
    pub num_vertices: u32,
}

impl Trail {
    pub fn new(device: &wgpu::Device, num_bodies: usize) -> Self {
        let vertex_buffer =
            device.create_buffer(
                &wgpu::BufferDescriptor {
                    label: Some("Trail Buffer"),
                    size: 10 * 1024 * 1024,
                    usage:
                        wgpu::BufferUsages::VERTEX |
                        wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation:false,
                }
            );
        Self {
            points: vec![Vec::new(); num_bodies],
            vertex_buffer,
            ranges: Vec::new(),
            num_vertices:0,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, positions: &Vec<Vec3>) {
        let trail_length = 25;

        for (i, pos) in positions.iter().enumerate() {
            if i >= self.points.len() {
                self.points.push(Vec::new());
            }
            self.points[i].push((*pos, 1.0));

            // limit trail size
            if self.points[i].len() > trail_length {
                self.points[i].remove(0);
            }
        }
        let mut vertices = Vec::new();
        self.ranges.clear();

        let mut offset = 0;

        for body_trail in &self.points {
            for (index, (pos, _)) in body_trail.iter().enumerate() {
                let alpha = index as f32 / body_trail.len() as f32;
                vertices.push(TrailVertex {pos: pos.to_array(), alpha});
            }

            self.ranges.push(offset..(offset + body_trail.len() as u32));

            offset += body_trail.len() as u32;
        }
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        self.num_vertices = vertices.len() as u32;
    }
}
