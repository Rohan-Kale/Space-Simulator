use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct TrailVertex {
    pos: [f32;3],
}

pub struct Trail {
    pub vertex_buffer: wgpu:: Buffer,
    pub vertex_count: u32,
}
// impl Trail {
//     pub fn update(&mut self, queue: &wgpu::Queue, bodies: &Vec<Body>) {
//         todo!()
//     }

// }
