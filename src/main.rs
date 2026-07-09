
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowId};
use winit::keyboard::KeyCode;

use std::time::Instant;
use std::collections::HashSet;

mod app_environment;
use crate::app_environment::AppEnvironment;

mod app;
use crate::app::*;
use crate::app::camera::Camera;

mod physics;
use physics::Body;
use crate::physics::octree::build_gpu_octree;

mod ui;
use crate::ui::{UiState, UiRenderer};

enum SpacePrograms {
    CreateBodies,
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let example_program = SpacePrograms::CreateBodies;

    let mut app = App::new("Vertex Buffer Example".to_string(), (1280, 720), example_program);
    let _ = event_loop.run_app(&mut app);
}


struct App {
    window_name: String,
    window_size: (i32, i32),
    environment: Option<AppEnvironment>,
    engine: Option<AppGraphicsEngine>,

    bodies: Vec<Body>,

    camera: Camera,

    example_program: SpacePrograms,

    keys: HashSet<KeyCode>,

    last_fps_update: Instant,
    frame_count: u32,
    fps: u32,
    last_frame: Instant,

    ui: UiState,
    ui_renderer: Option<UiRenderer>,

    camera_rotating: bool,

    gpu_nodes: Vec<physics::gpu_octree::GpuOctreeNode>,
    octree_timer: f32,
}

impl App {
    pub fn new(window_name: String, window_size: (i32, i32), example_program: SpacePrograms) -> Self {

        let mut bodies = Vec::new();

        let pi = std::f32::consts::PI;

        // Sun
        bodies.push(Body {
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            acceleration: [0.0, 0.0, 0.0],
            mass: 1.0,
            radius: 0.06,
        });

        // Earth
        bodies.push(Body {
            position: [1.0, 0.0, 0.0],
            velocity: [
                0.0,
                2.0 * pi,
                0.0
            ],
            acceleration: [0.0, 0.0, 0.0],
            mass: 3.003e-6,
            radius: 0.06,
        });

        // for i in 0..5000 {
        //     let angle = i as f32 * 0.2;
        //     let radius = 10.0 + (i as f32 * 0.05);

        //     bodies.push(Body {
        //         position: [
        //             radius * angle.cos(),
        //             radius * angle.sin(),
        //             0.0,
        //         ],

        //        velocity: [
        //             -angle.sin() * 3.1,
        //             angle.cos() * 3.1,
        //             0.0,
        //         ],

        //         acceleration: [0.0,0.0,0.0],

        //         mass: 0.01,

        //         radius: 0.5,
        //     });
        // }

        let camera = Camera {
            position: glam::vec3(0.0, 3.0, 8.0),

            up: glam::Vec3::Y,

            aspect: window_size.0 as f32 / window_size.1 as f32,

            fovy: 45.0_f32.to_radians(),

            // AU scale
            znear: 0.001,
            zfar: 100.0,

            yaw: -90.0_f32.to_radians(),
            pitch: -15.0_f32.to_radians(),
        };


        Self {
            window_name,
            window_size,
            environment: None,
            engine: None,
            bodies,
            camera,
            example_program,
            last_fps_update: Instant::now(),
            frame_count: 0,
            fps: 0,
            keys: HashSet::new(),
            last_frame: Instant::now(),
            ui: UiState::new(),
            ui_renderer: None,
            camera_rotating: false, 
            gpu_nodes: Vec::new(),
            octree_timer: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.octree_timer += dt;

        if self.ui.paused {
            return;
        }

        if self.octree_timer > 0.05 {
            self.gpu_nodes = build_gpu_octree(&self.bodies);
            self.octree_timer = 0.0;
        }

        self.frame_count += 1;

        if self.last_fps_update.elapsed().as_secs_f32() >= 1.0 {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.last_fps_update = Instant::now();
        }

        let speed = 2.5;
        let right = self.camera.direction().cross(self.camera.up).normalize();
        let up = self.camera.up.normalize();

        if self.keys.contains(&KeyCode::KeyW) {
            self.camera.position += self.camera.direction() * speed * dt;
        }

        if self.keys.contains(&KeyCode::KeyS) {
            self.camera.position -= self.camera.direction() * speed * dt;
        }

        if self.keys.contains(&KeyCode::KeyA) {
            self.camera.position -= right * speed * dt;
        }

        if self.keys.contains(&KeyCode::KeyD) {
            self.camera.position += right * speed * dt;
        }

        if self.keys.contains(&KeyCode::Space) {
            self.camera.position += up * speed * dt;
        }

        if self.keys.contains(&KeyCode::ShiftLeft)
            || self.keys.contains(&KeyCode::ShiftRight)
        {
            self.camera.position -= up * speed * dt;
        }
    }

    fn render(&mut self) {
        let full_output = self.update_ui();

        let app_window = self.environment.as_ref().unwrap();

        self.engine
            .as_ref()
            .unwrap()
            .update_camera(
                &app_window.queue,
                &self.camera,
            );

        let frame = match app_window.surface.get_current_texture() {
            frame => frame,
        };

        let surface_texture = match frame {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout => {
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                app_window.surface.configure(
                    &app_window.device,
                    &app_window.surface_desc,
                );
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                app_window.surface.configure(
                    &app_window.device,
                    &app_window.surface_desc,
                );
                return;
            }

            wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return;
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        //println!("GPU octree nodes: {}", self.gpu_nodes.len());

        self.engine
            .as_mut()
            .unwrap()
            .render(
                &app_window.queue,
                &app_window.device,
                &view,
                self.ui.time_scale * 0.001,
                self.ui.show_velocity_vectors,
                &self.gpu_nodes,
            );

        let paint_jobs =
            self.ui_renderer
                .as_mut()
                .unwrap()
                .context
                .tessellate(
                    full_output.shapes,
                    full_output.pixels_per_point
                );

        let textures_delta = &full_output.textures_delta;

        for (id, image_delta) in &textures_delta.set {
            self.ui_renderer
                .as_mut()
                .unwrap()
                .renderer
                .update_texture(
                    &app_window.device,
                    &app_window.queue,
                    *id,
                    image_delta,
                );
        }

        for id in &textures_delta.free {
            self.ui_renderer
                .as_mut()
                .unwrap()
                .renderer
                .free_texture(id);
        }

        // println!("SHAPES: {}", shapes_len);
        // println!("PAINT JOBS: {}", paint_jobs.len());

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                self.window_size.0 as u32,
                self.window_size.1 as u32,
            ],
            pixels_per_point:
                app_window.window.scale_factor() as f32,
        };

        let mut encoder =
            app_window.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("egui encoder"),
                }
            );

        self.ui_renderer
            .as_mut()
            .unwrap()
            .renderer
            .update_buffers(
                &app_window.device,
                &app_window.queue,
                &mut encoder,
                &paint_jobs,
                &screen_descriptor,
            );


        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("egui render pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        }
                    )],
                    depth_stencil_attachment: None,
                    ..Default::default()
                }
            ).forget_lifetime();

            self.ui_renderer
                .as_mut()
                .unwrap()
                .renderer
                .render(
                    &mut render_pass,
                    &paint_jobs,
                    &screen_descriptor,
                );
        }
        app_window.queue.submit(Some(encoder.finish()));

        surface_texture.present();
    }

    fn update_ui(&mut self) -> egui::FullOutput {
        let ui_renderer = self.ui_renderer.as_mut().unwrap();

        let raw_input = ui_renderer
            .state
            .take_egui_input(
                &self.environment.as_ref().unwrap().window
            );

        // println!("EGUI FRAME");

        ui_renderer.context.run_ui(raw_input, |ctx| {

           egui::Window::new("Space Simulator")
                .show(ctx, |ui| {
                    ui.label(format!("FPS: {}", self.fps));
                    ui.label(format!(
                        "Bodies: {}",
                        self.bodies.len()
                    ));
                    ui.separator();
                    ui.add(
                        egui::Slider::new(
                            &mut self.ui.time_scale,
                            0.0..=5.0
                        )
                        .text("Simulation Speed")
                    );
                    ui.add(
                        egui::Slider::new(
                            &mut self.ui.theta,
                            0.1..=100.0
                        )
                        .text("Barnes-Hut Accuracy")
                    );
                    ui.checkbox(
                        &mut self.ui.show_velocity_vectors,
                        "Velocity Vectors"
                    );
                    ui.checkbox(
                        &mut self.ui.show_octree,
                        "Show Octree"
                    );
                });
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.environment = Some(AppEnvironment::new(&event_loop, self.window_name.clone(), self.window_size));

        let device = &self.environment
            .as_ref()
            .unwrap()
            .device;

        let app_window = self.environment.as_ref().unwrap();

        self.ui_renderer = Some(UiRenderer::new(
            egui::Context::default(),
            &app_window.window,
            &app_window.device,
            app_window.surface_desc.format,
        ));

        self.engine = Some(AppGraphicsEngine::new(&self.environment.as_ref().unwrap().device, &self.environment.as_ref().unwrap().surface_desc, &self.example_program, &self.bodies, &self.camera));
        
        // add queue if using write_buffer() example
        // self.engine = Some(AppGraphicsEngine::new(&self.environment.as_ref().unwrap().device, &self.environment.as_ref().unwrap().surface_desc, &self.environment.as_ref().unwrap().queue));
    }

    

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: winit::event::DeviceEvent) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                if !self.camera_rotating {
                    return;
                }

                let sensitivity = 0.002;

                self.camera.yaw += delta.0 as f32 * sensitivity;
                self.camera.pitch -= delta.1 as f32 * sensitivity;

                self.camera.pitch = self.camera.pitch.clamp(
                    -1.5,
                    1.5
                );
            }
            _ => {}
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent)
    {
        if let Some(ui) = self.ui_renderer.as_mut() {
            let response = ui.state.on_window_event(
                &self.environment.as_ref().unwrap().window,
                &event,
            );

            if response.consumed {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(key) = key_event.physical_key {
                    if key_event.state.is_pressed() {
                        self.keys.insert(key);
                    } else {
                        self.keys.remove(&key);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Right {
                    self.camera_rotating = state.is_pressed();
                }
            }
            WindowEvent::MouseWheel {device_id: _,  delta, phase: _ } => {
                let zoom_speed = 0.05;

                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.camera.fovy -= y * zoom_speed;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        self.camera.fovy -= pos.y as f32 * zoom_speed * 0.01;
                    }
                }

                //prevent camera from tweaking out
                self.camera.fovy = self.camera.fovy.clamp(10.0_f32.to_radians(), 90.0_f32.to_radians());
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                self.environment
                    .as_mut()
                    .unwrap()
                    .window
                    .request_redraw();

                self.update(dt);
                self.render();
            },
            _ => (),
        }
    }
}
