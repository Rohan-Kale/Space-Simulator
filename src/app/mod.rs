use wgpu::{PipelineCompilationOptions, include_wgsl};

pub mod example_object;
use crate::{SpacePrograms, app::example_object::Object};

use crate::physics::{Body, GpuBody, SimulationParams};

pub mod camera;
use crate::app::camera::{Camera, CameraUniform};
use wgpu::util::DeviceExt;


pub mod trail;
use crate::app::trail::{Trail, TrailVertex};

pub mod starfield;
use crate::app::starfield::{Starfield, StarVertex};

pub struct AppGraphicsEngine {
    pipeline: wgpu::RenderPipeline,
    trail_pipeline: wgpu::RenderPipeline,
    example_object: Object,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    depth_texture: wgpu::TextureView,

    starfield: Starfield,
    star_pipeline: wgpu::RenderPipeline,

    gravity_pipeline: wgpu::ComputePipeline,
    gravity_bind_group: wgpu::BindGroup,
    body_buffer: wgpu::Buffer,
    body_read_buffer: wgpu::Buffer,
    simulation_buffer: wgpu::Buffer,

    velocity_pipeline: wgpu::RenderPipeline,
    velocity_layout: wgpu::BindGroupLayout,
    velocity_bind_group: wgpu::BindGroup,
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
            bind_group_layouts: &[Some(&camera_bind_group_layout)],
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
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
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

        let trail_shader =
            device.create_shader_module(
                include_wgsl!("../../resources/trail.wgsl")
            );

        let trail_pipeline =
            device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
            label: Some("Trail Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &trail_shader,
                entry_point: Some("vs_main"),
                compilation_options:
                    PipelineCompilationOptions::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride:
                            std::mem::size_of::<TrailVertex>()
                            as u64,
                        step_mode:
                            wgpu::VertexStepMode::Vertex,
                        attributes:&[
                            wgpu::VertexAttribute {
                                offset:0,
                                shader_location:0,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32;3]>() as u64,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32,
                            }
                        ],
                    }
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module:&trail_shader,
                entry_point:Some("fs_main"),
                compilation_options:
                    PipelineCompilationOptions::default(),
                targets:&[
                    Some(wgpu::ColorTargetState {
                        format:config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask:wgpu::ColorWrites::ALL,
                    })
                ],
            }),
            primitive:wgpu::PrimitiveState {
                topology:
                wgpu::PrimitiveTopology::LineStrip,
                ..Default::default()
            },
            depth_stencil: Some(
                wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: Default::default(),
                    bias: Default::default(),
                }
            ),
            multisample:
            wgpu::MultisampleState::default(),
            multiview_mask:None,
            cache:None,
            });

            let starfield = Starfield::new(device);

            let star_shader =
            device.create_shader_module(include_wgsl!("../../resources/star.wgsl"));

            let star_pipeline =
                device.create_render_pipeline(
                    &wgpu::RenderPipelineDescriptor {
                        label: Some("Star Pipeline"),

                        layout: Some(&pipeline_layout),

                        vertex: wgpu::VertexState {
                            module: &star_shader,
                            entry_point: Some("vs_main"),
                            compilation_options:
                                PipelineCompilationOptions::default(),

                            buffers: &[
                                wgpu::VertexBufferLayout {
                                    array_stride:
                                        std::mem::size_of::<StarVertex>() as u64,
                                    step_mode:
                                        wgpu::VertexStepMode::Vertex,
                                    attributes:&[
                                        wgpu::VertexAttribute {
                                            offset:0,
                                            shader_location:0,
                                            format:
                                            wgpu::VertexFormat::Float32x3,
                                        }
                                    ],
                                }
                            ],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module:&star_shader,
                            entry_point:Some("fs_main"),
                            compilation_options:
                                PipelineCompilationOptions::default(),
                            targets:&[
                                Some(wgpu::ColorTargetState {
                                    format:config.format,
                                    blend:Some(wgpu::BlendState::REPLACE),
                                    write_mask:
                                    wgpu::ColorWrites::ALL,
                                })
                            ],
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology:
                                wgpu::PrimitiveTopology::PointList,

                            ..Default::default()
                        },
                        depth_stencil: Some(
                            wgpu::DepthStencilState {
                                format: wgpu::TextureFormat::Depth32Float,
                                depth_write_enabled: Some(false),
                                depth_compare: Some(wgpu::CompareFunction::Always),
                                stencil: Default::default(),
                                bias: Default::default(),
                            }
                        ),
                        multisample: wgpu::MultisampleState::default(),
                        multiview_mask:None,
                        cache:None,
                    }
                );

                let gpu_bodies: Vec<GpuBody> = bodies
                    .iter()
                    .map(|b| GpuBody::from(b))
                    .collect();

                let body_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Body Storage Buffer"),
                        contents: bytemuck::cast_slice(&gpu_bodies),
                        usage:
                            wgpu::BufferUsages::STORAGE |
                            wgpu::BufferUsages::VERTEX |
                            wgpu::BufferUsages::COPY_DST |
                            wgpu::BufferUsages::COPY_SRC,
                    }
                );

                let simulation_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Simulation Buffer"),
                        contents: bytemuck::cast_slice(&[SimulationParams {
                            dt: 0.001,
                            _padding: [0.0; 7],
                        }]),
                        usage:
                            wgpu::BufferUsages::UNIFORM |
                            wgpu::BufferUsages::COPY_DST,
                    },
                );

                let gravity_shader = device.create_shader_module(include_wgsl!("../../resources/gravity.wgsl"));
                let gravity_layout = device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: Some("gravity layout"),
                        entries: &[
                            //bodies buffer
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::COMPUTE,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            // simulation params
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::COMPUTE,
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

                let gravity_bind_group = device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        label: Some("gravity bind group"),
                        layout: &gravity_layout,
                        entries: &[
                            // bodies
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: body_buffer.as_entire_binding(),
                            },
                            // simulation params
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: simulation_buffer.as_entire_binding(),
                            },
                        ],
                    }
                );

                let gravity_pipeline = device.create_compute_pipeline(
                &wgpu::ComputePipelineDescriptor {
                    label: Some("Gravity Compute"),
                    layout: Some(
                        &device.create_pipeline_layout(
                            &wgpu::PipelineLayoutDescriptor {
                                label: None,
                                bind_group_layouts: &[Some(&gravity_layout)],
                                immediate_size: 0,
                            }
                        )
                    ),
                    module: &gravity_shader,
                    entry_point: Some("main"),
                    compilation_options:
                        PipelineCompilationOptions::default(),
                    cache: None,
                }
            );

            let body_read_buffer = device.create_buffer(
                &wgpu::BufferDescriptor {
                    label: Some("Body Read Buffer"),
                    size: (std::mem::size_of::<GpuBody>() * bodies.len()) as u64,
                    usage: 
                        wgpu::BufferUsages::COPY_DST |
                        wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }
            );

            let velocity_shader =
                device.create_shader_module(include_wgsl!("../../resources/velocity.wgsl"));

            let velocity_layout = device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("velocity layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }
                    ],
                }
            );

            let velocity_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("velocity pipeline layout"),
                    bind_group_layouts: &[
                        Some(&velocity_layout),
                        Some(&camera_bind_group_layout),
                    ],
                    immediate_size: 0,
                });

            let velocity_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Velocity Pipeline"),
                layout: Some(&velocity_pipeline_layout), // same camera layout is fine
                vertex: wgpu::VertexState {
                    module: &velocity_shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[
                        // wgpu::VertexBufferLayout {
                        //     array_stride: std::mem::size_of::<GpuBody>() as u64,
                        //     step_mode: wgpu::VertexStepMode::Instance,
                        //     attributes: &[],
                        // }
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &velocity_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            });

            let velocity_bind_group = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("velocity bind group"),
                    layout: &velocity_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: body_buffer.as_entire_binding(),
                        }
                    ],
                }
            );

        Self {
            pipeline,
            trail_pipeline,
            example_object,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            starfield,
            star_pipeline,
            gravity_pipeline,
            gravity_bind_group,
            body_buffer,
            simulation_buffer,
            body_read_buffer,
            velocity_pipeline,
            velocity_layout,
            velocity_bind_group,
        }
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

    pub fn get_positions(&self, device: &wgpu::Device) -> Vec<glam::Vec3> {
        let slice = self.body_read_buffer.slice(..);

        slice.map_async(wgpu::MapMode::Read, |_| {});

        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None});

        let data = slice.get_mapped_range();

        let bodies: &[GpuBody] =
            bytemuck::cast_slice(&data);

        let positions =
            bodies.iter()
            .map(|b| {
                glam::Vec3::new(
                    b.position[0],
                    b.position[1],
                    b.position[2],
                )
            })
            .collect();

        drop(data);

        self.body_read_buffer.unmap();

        positions
    }

    pub fn render(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, view: &wgpu::TextureView, trail: Option<&Trail>, simulation_dt: f32, show_velocity_vectors: bool) {
        queue.write_buffer(
            &self.simulation_buffer,
            0,
            bytemuck::cast_slice(&[SimulationParams {
                dt: simulation_dt,
                _padding: [0.0; 7],
            }]),
        );

        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }
        );

        //run the gravity compute shader
        {
            let mut cpass = encoder.begin_compute_pass(
                &wgpu::ComputePassDescriptor {
                    label: Some("Gravity Compute Pass"),
                    timestamp_writes: None,
                }
            );

            cpass.set_pipeline(&self.gravity_pipeline);
            cpass.set_bind_group(0, &self.gravity_bind_group, &[]);

            let workgroups = (self.example_object.instances + 63) / 64;
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }

        encoder.copy_buffer_to_buffer(&self.body_buffer, 0, &self.body_read_buffer, 0, self.body_buffer.size());

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

       if show_velocity_vectors {
            rpass.set_pipeline(&self.velocity_pipeline);

            rpass.set_bind_group(0, &self.velocity_bind_group, &[]);
            rpass.set_bind_group(1, &self.camera_bind_group, &[]);

            rpass.draw(0..2, 0..self.example_object.instances);
        }

        rpass.set_bind_group(
            0,
            &self.camera_bind_group,
            &[],
        );

        // create starfield pipeline and draw starfield
        rpass.set_pipeline(&self.star_pipeline);
        rpass.set_vertex_buffer(0, self.starfield.vertex_buffer.slice(..));
        rpass.draw(0..self.starfield.num_stars, 0..1);

        // draw planets first
        rpass.set_pipeline(&self.pipeline);

        rpass.set_bind_group(0, &self.camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.example_object.vertex_buffers[0].slice(..));
        rpass.set_vertex_buffer(1, self.body_buffer.slice(..));

        if let Some(index_buffer) = &self.example_object.index_buffer {
            rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(
                0..self.example_object.num_to_draw,
                0,
                0..self.example_object.instances,
            );
        } else {
            rpass.draw(
                0..self.example_object.num_to_draw,
                0..self.example_object.instances,
            );
        }

        if let Some(trail) = trail {
            rpass.set_pipeline(&self.trail_pipeline);
            rpass.set_vertex_buffer(0, trail.vertex_buffer.slice(..));

            for range in &trail.ranges {
                if range.end > range.start {
                    rpass.draw(range.clone(), 0..1);
                }
            }
        }

        drop(rpass);

        queue.submit(Some(encoder.finish()));
    }

}
