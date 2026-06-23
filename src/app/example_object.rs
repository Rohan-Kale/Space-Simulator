use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use wgpu::{VertexBufferLayout, util::{BufferInitDescriptor, DeviceExt}};

use crate::physics::Body;

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]

struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
    uv: [f32; 2],
}

pub struct Object {
    pub vertex_buffers: Vec<wgpu::Buffer>, 
    pub layouts: Vec<VertexBufferLayout<'static>>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub num_to_draw: u32, // number of vertices or indices to pass to draw()
    pub instances: u32,
    pub instance_buffer: wgpu::Buffer,
}

impl Object {
    pub fn update_instances(
        &self,
        queue: &wgpu::Queue,
        bodies: &Vec<Body>,
    ) {
        let mut instance_data = Vec::new();

        for instance in bodies {
            instance_data.push(instance.position[0]);
            instance_data.push(instance.position[1]);
            instance_data.push(instance.position[2]);
            instance_data.push(instance.radius);
        }

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );
    }

    pub fn create_sphere(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        let stacks = 32;
        let sectors = 32;

        for i in 0..=stacks {

            let phi = PI * i as f32 / stacks as f32;

            for j in 0..=sectors {

                let theta = 2.0 * PI * j as f32 / sectors as f32;


                let x = theta.cos() * phi.sin();
                let y = phi.cos();
                let z = theta.sin() * phi.sin();

                vertices.push(Vertex {
                    pos: [x,y,z],
                    color: [1.0,1.0,1.0],
                    uv: [
                        j as f32 / sectors as f32,
                        i as f32 / stacks as f32
                    ],
                });
            }
        }


        for i in 0..stacks {
            for j in 0..sectors {
                let a = i * (sectors+1) + j;
                let b = a + sectors + 1;

                indices.push(a as u32);
                indices.push(b as u32);
                indices.push((a+1) as u32);

                indices.push((a+1) as u32);
                indices.push(b as u32);
                indices.push((b+1) as u32);
            }
        }
    }

    pub fn create_bodies(device: &wgpu::Device, bodies: &Vec<Body>) -> Self {
        let mut vertex_data = Vec::new();
        let mut indices = Vec::new();

        Object::create_sphere(&mut vertex_data, &mut indices);

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Spiral Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        // we need an indices buffer because order matters now
        let index_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Sphere Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress, 
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        };

        
        // Instance Buffer
        let mut body_data = Vec::new();

        for body in bodies {
            body_data.push(body.position[0]);
            body_data.push(body.position[1]);
            body_data.push(body.position[2]);
            body_data.push(body.radius);
        }

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Body Instance Buffer"),
            contents: bytemuck::cast_slice(&body_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,   // COPY_DST if we want to update instance data every frame using write_buffer()
        });


        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<f32>() * 4) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3, // position
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;3]>() as u64,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32, // radius
                },
            ],
        };

        let mut vertex_buffers = Vec::new();
        vertex_buffers.push(vertex_buffer);

        let mut layouts = Vec::new();
        layouts.push(vertex_layout);
        layouts.push(instance_layout);

        Self { 
            vertex_buffers,
            layouts,
            index_buffer: Some(index_buffer),
            num_to_draw: indices.len() as u32, // draw the number of of isntances
            instances: bodies.len() as u32, // number of bodies to draw
            instance_buffer,
        }
    }

}
