
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowId};
use winit::keyboard::{Key, KeyCode};
use winit::window::CursorGrabMode;
use winit::event::DeviceEvent;

use std::time::Instant;
use std::collections::HashSet;

mod app_environment;
use crate::app_environment::AppEnvironment;

mod app;
use crate::app::*;
use crate::app::camera::Camera;


mod physics;
use physics::Body;

enum SpacePrograms {
    CreateBodies,
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let example_program = SpacePrograms::CreateBodies;

    let mut app = App::new("Vertex Buffer Example".to_string(), (600, 600), example_program);
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
    last_mouse_position: Option<(f64, f64)>,

    last_fps_update: Instant,
    frame_count: u32,
    fps: u32,
    last_frame: Instant,
}

impl App {
    pub fn new(window_name: String, window_size: (i32, i32), example_program: SpacePrograms) -> Self {

        let mut bodies = Vec::new();

        bodies.push(Body {
            position: [0.0, 0.0, 5.0],
            velocity: [0.0, 0., 0.0],
            acceleration: [0.0, 0.0, 0.0],
            mass: 2.0,
            radius: 0.5,
        });

        for i in 0..2 {
            let angle = i as f32 * 0.01;
            let radius = 10.0 + i as f32 * 0.01;

            bodies.push(Body {
                position: [
                    radius * angle.cos(),
                    radius * angle.sin(),
                    0.0,
                ],
                velocity: [
                    -angle.sin(),
                    angle.cos(),
                    0.0,
                ],
                acceleration: [0.0, 0.0, 0.0],
                mass: 1.0,
                radius: 0.02,
            });
        }

        let camera = Camera {
            position: glam::vec3(0.0, 0.0, 10.0),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,

            aspect: window_size.0 as f32 / window_size.1 as f32,
            fovy: 45.0_f32.to_radians(),
            znear: 0.1,
            zfar: 1000.0,

            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
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
            last_mouse_position: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.environment = Some(AppEnvironment::new(&event_loop, self.window_name.clone(), self.window_size));
        self.engine = Some(AppGraphicsEngine::new(&self.environment.as_ref().unwrap().device, &self.environment.as_ref().unwrap().surface_desc, &self.example_program, &self.bodies, &self.camera));
        // add queue if using write_buffer() example
        // self.engine = Some(AppGraphicsEngine::new(&self.environment.as_ref().unwrap().device, &self.environment.as_ref().unwrap().surface_desc, &self.environment.as_ref().unwrap().queue));
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: winit::event::DeviceEvent) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {

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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(key) = event.physical_key {
                    if event.state.is_pressed() {
                        self.keys.insert(key);
                    } else {
                        self.keys.remove(&key);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {

                if let Some((last_x, last_y)) = self.last_mouse_position {

                    let dx = position.x - last_x;
                    let dy = position.y - last_y;

                    let sensitivity = 0.002;

                    self.camera.yaw += dx as f32 * sensitivity;
                    self.camera.pitch -= dy as f32 * sensitivity;

                    // prevent flipping upside down
                    self.camera.pitch = self.camera.pitch.clamp(
                        -1.5,
                        1.5
                    );
                }

                self.last_mouse_position = Some((position.x, position.y));
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                self.environment.as_mut().unwrap().window.request_redraw();
                
                self.frame_count += 1;

                if self.last_fps_update.elapsed().as_secs_f32() >= 1.0 {
                    self.fps = self.frame_count;
                    self.frame_count = 0;
                    self.last_fps_update = Instant::now();

                    println!(
                        "FPS: {} | Bodies: {}",
                        self.fps,
                        self.bodies.len()
                    );
                }
                

                let physics_start = Instant::now();

                physics::update_bodies(&mut self.bodies, 0.001);

                let physics_ms = physics_start.elapsed().as_secs_f64() * 1000.0;
                
                println!(
                    "FPS: {} | Bodies: {} | Physics: {:.3} ms",
                    self.fps,
                    self.bodies.len(),
                    physics_ms
                );

                let app_window = self.environment.as_ref().unwrap();

                let speed = 2.5;
                let right = self.camera
                    .direction()
                    .cross(self.camera.up)
                    .normalize();
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

                if self.keys.contains(&KeyCode::ShiftLeft) || self.keys.contains(&KeyCode::ShiftRight){
                    self.camera.position -= up * speed * dt;
                }

                self.engine
                    .as_ref()
                    .unwrap()
                    .update_camera(
                        &app_window.queue,
                        &self.camera,
                    );

                // Send new body positions to GPU
                self.engine
                    .as_ref()
                    .unwrap()
                    .update_instances(
                        &app_window.queue,
                        &self.bodies
                    );

                let frame = match app_window.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {e:?}");
                        return;
                    }
                }; 

                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                self.engine.as_mut().unwrap().render(&app_window.queue, &app_window.device, &view);
                frame.present();
                
            },
            _ => (),
        }

    }
}
