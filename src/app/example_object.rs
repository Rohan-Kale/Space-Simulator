use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use wgpu::{BufferDescriptor, VertexBufferLayout, naga::DerivativeAxis::Y, util::{BufferInitDescriptor, DeviceExt}, wgc::device::queue};
use winit::dpi::Position;

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

        for body in bodies {
            instance_data.push(body.position[0]);
            instance_data.push(body.position[1]);
            instance_data.push(body.radius);
        }

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );
    }

    pub fn create_bodies(device: &wgpu::Device, bodies: &Vec<Body>) -> Self {
        let mut vertex_data = Vec::new();
        // IN ORDER TO DRAW SQUARE WE USE TWO TRIANGLES
        
        vertex_data.push(Vertex { pos: [-0.5,  0.5, 0.0], color: [1.0, 0.0, 0.0], uv: [0.0, 1.0] }); // top left

        vertex_data.push(Vertex { pos: [ 0.5,  0.5, 0.0],  color: [0.0, 1.0, 0.0], uv: [1.0, 1.0] }); // top right

        vertex_data.push(Vertex { pos: [-0.5, -0.5, 0.0],  color: [0.0, 0.0, 1.0], uv: [0.0, 0.0]   }); // bottom left

        vertex_data.push(Vertex { pos: [ 0.5, -0.5, 0.0], color: [1.0, 1.0, 1.0], uv: [1.0, 0.0] }); // bottom right


        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Spiral Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        // we need an indices buffer because order matters now
        let indices: &[u32] = &[ 0, 1, 2, 2, 1, 3];

        let index_buffer = device.create_buffer_init( 
            &BufferInitDescriptor {
                label: Some("Square Index Buffer"),
                contents: bytemuck::cast_slice(indices),
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
            body_data.push(body.radius);
        }

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Body Instance Buffer"),
            contents: bytemuck::cast_slice(&body_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,   // COPY_DST if we want to update instance data every frame using write_buffer()
        });


        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<f32>() * 3) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2, // x,y
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<f32>() as u64 * 2,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32, // r
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
            num_to_draw: 6, // each object drawn is a triangle made of three vertices
            instances: bodies.len() as u32, // number of instances to draw
            instance_buffer,
        }
    }

}
