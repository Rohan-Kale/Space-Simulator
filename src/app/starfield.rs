use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use rand::Rng;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct StarVertex {
    pub pos: [f32;3],
}


pub struct Starfield {
    pub vertex_buffer: wgpu::Buffer,
    pub num_stars: u32,
}


impl Starfield {

    pub fn new(device: &wgpu::Device) -> Self {          
        let mut rng = rand::thread_rng();
        let mut stars = Vec::new();

        for _ in 0..5000 {

            let x = rng.gen_range(-100.0..100.0);
            let y = rng.gen_range(-100.0..100.0);
            let z = rng.gen_range(-100.0..100.0);

            stars.push(
                StarVertex {
                    pos:[x,y,z]
                }
            );
        }


        let vertex_buffer =
            device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Star Buffer"),
                    contents: bytemuck::cast_slice(&stars),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );


        Self {
            vertex_buffer,
            num_stars: stars.len() as u32,
        }
    }
}
