use wgpu::{include_wgsl, PipelineCompilationOptions};


pub struct AppData {
    pipeline: wgpu::RenderPipeline,
}

impl AppData {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let triangle_shader = device.create_shader_module(include_wgsl!("../../resources/triangle_shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("triangle_pipeline_layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {    
                label: Some("triangle_render_pipeline"), 
                layout: Some(&pipeline_layout), 
                vertex: wgpu::VertexState { 
                    module: &triangle_shader, // point to vertex shader
                    entry_point: Some("vs_main"),  // declare function marked with @vertex attribute
                    compilation_options: PipelineCompilationOptions::default(), 
                    buffers: &[],
                }, 
                fragment: Some(wgpu::FragmentState {
                    module: &triangle_shader, // point to fragment shader
                    entry_point: Some("fs_main"), // declare function marked with @fragment attribute
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
                    cull_mode: None, // Some(wgpu::Face::Back),
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
                cache: None, 
                multiview_mask: None,
            });


        Self {
            pipeline,
        }
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView) {
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
                        r: 0.3,
                        g: 0.7,
                        b: 0.9,
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
        rpass.draw(0..3, 0..4);

        drop(rpass);

        queue.submit(Some(encoder.finish()));
    }

}
