use wgpu::{PipelineCompilationOptions, include_wgsl};

pub mod example_object;
use crate::{SpacePrograms, app::example_object::Object};

use crate::physics::Body;

pub mod trail;
use crate::trail::Trail;

pub mod camera;
use crate::app::camera::{Camera, CameraUniform};
use wgpu::util::DeviceExt;



pub struct AppGraphicsEngine {
    pipeline: wgpu::RenderPipeline,
    // trail_pipeline: wgpu::RenderPipeline,
    example_object: Object,
    // trail_object: Trail,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    depth_texture: wgpu::TextureView,
}

impl AppGraphicsEngine {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        example_program: &SpacePrograms,
        bodies: &Vec<Body>,
        camera: &Camera,
    ) -> Self {

        let camera_uniform = CameraUniform {
            view_proj: camera
                .build_view_projection_matrix()
                .to_cols_array_2d(),
        };


        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM |
                    wgpu::BufferUsages::COPY_DST,
            }
        );

        let shaders;
        let example_object;

        match example_program {
            SpacePrograms::CreateBodies => {
                shaders =
                    device.create_shader_module(include_wgsl!("../../resources/particle.wgsl"));
                example_object = Object::create_bodies(device, bodies);
            }
        }

        let camera_bind_group_layout =
            device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("camera_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }
                    ],
                }
            );


        let camera_bind_group =
            device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("camera_bind_group"),
                    layout: &camera_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: camera_buffer.as_entire_binding(),
                        }
                    ],
                }
            );


        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("triangle_pipeline_layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
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
            depth_stencil: Some(
                wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }
            ),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },

            multiview_mask: None,
            cache: None,
        });

        let depth_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            }
        ).create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            pipeline,
            example_object,
            camera_buffer,
            camera_bind_group,
            depth_texture,
        }
    }

    pub fn update_instances(
        &self,
        queue: &wgpu::Queue,
        bodies: &Vec<Body>,
    ) {
        self.example_object.update_instances(queue, bodies);
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera,) {
        let uniform = CameraUniform {
            view_proj: camera
                .build_view_projection_matrix()
                .to_cols_array_2d(),
        };

        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );
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
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(
                        wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }
                    ),
                    stencil_ops: None,
                }
            ),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        rpass.set_pipeline(&self.pipeline);

        rpass.set_bind_group(
            0,
            &self.camera_bind_group,
            &[],
        );

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
