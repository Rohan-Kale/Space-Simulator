use wgpu::{PipelineCompilationOptions, include_wgsl};

pub mod example_object;
use crate::{SpacePrograms, app::example_object::Object};

use crate::physics::Body;


pub struct AppGraphicsEngine {
    pipeline: wgpu::RenderPipeline,
    example_object: Object,
}

impl AppGraphicsEngine {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        example_program: &SpacePrograms,
        bodies: &Vec<Body>,
    ) -> Self {
        // pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, queue: &wgpu::Queue) -> Self { // add queue if you use write_buffer in create_triangle

        let shaders;
        let example_object;

        match example_program {
            SpacePrograms::CreateBodies => {
                shaders =
                    device.create_shader_module(include_wgsl!("../../resources/particle.wgsl"));
                example_object = Object::create_bodies(device, bodies);
            }
        }

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("triangle_pipeline_layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shaders,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &example_object.layouts, // Add vertex buffer layouts here
            },
            fragment: Some(wgpu::FragmentState {
                module: &shaders,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },

            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            example_object,
        }
    }

    pub fn update_instances(
        &self,
        queue: &wgpu::Queue,
        bodies: &Vec<Body>,
    ) {
        self.example_object.update_instances(queue, bodies);
    }

    pub fn render(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, view: &wgpu::TextureView) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        rpass.set_pipeline(&self.pipeline);

        // square mesh
        rpass.set_vertex_buffer(0, self.example_object.vertex_buffers[0].slice(..));

        // body positions + radius
        rpass.set_vertex_buffer(1, self.example_object.instance_buffer.slice(..));

        // If we have an index buffer, draw using indexing, if we don't, draw using vertices
        if self.example_object.index_buffer.is_some() {
            //println!("num to draw: {}, instances: {}", self.example_object.num_to_draw, self.example_object.instances);
            rpass.set_index_buffer(
                self.example_object.index_buffer.as_ref().unwrap().slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.draw_indexed(0..self.example_object.num_to_draw, 0, 0..self.example_object.instances);
        } else {
            println!(
                "draw {} vertices, {} instances",
                self.example_object.num_to_draw, self.example_object.instances
            );
            
            rpass.draw(
                0..self.example_object.num_to_draw,
                0..self.example_object.instances,
            );
        }

        drop(rpass);

        queue.submit(Some(encoder.finish()));
    }
}
