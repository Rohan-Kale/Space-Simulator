
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowId};

mod app_environment;
use crate::app_environment::AppEnvironment;

mod app;
use crate::app::*;


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

    example_program: SpacePrograms,
}

impl App {
    pub fn new(window_name: String, window_size: (i32, i32), example_program: SpacePrograms) -> Self {

        let mut bodies = Vec::new();

        bodies.push(Body {
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
            acceleration: [0.0, 0.0],
            mass: 1.0,
            radius: 0.05,
        });

        for i in 0..200 {
            let angle = i as f32 * 0.5;
            let radius = 0.2 + i as f32 * 0.005;

            let velocity = (1.0 / radius).sqrt();

            bodies.push(Body {
                position: [
                    radius * angle.cos(),
                    radius * angle.sin(),
                ],

                velocity: [
                    -velocity * angle.sin(),
                    velocity * angle.cos(),
                ],

                acceleration: [0.0, 0.0],
                mass: 0.01,
                radius: 0.01,
            });
        }
        Self {
            window_name,
            window_size,
            environment: None,
            engine: None,
            bodies,
            example_program,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.environment = Some(AppEnvironment::new(&event_loop, self.window_name.clone(), self.window_size));
        self.engine = Some(AppGraphicsEngine::new(&self.environment.as_ref().unwrap().device, &self.environment.as_ref().unwrap().surface_desc, &self.example_program, &self.bodies));
        
        // add queue if using write_buffer() example
        // self.engine = Some(AppGraphicsEngine::new(&self.environment.as_ref().unwrap().device, &self.environment.as_ref().unwrap().surface_desc, &self.environment.as_ref().unwrap().queue));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                self.environment.as_mut().unwrap().window.request_redraw();

                physics::update_bodies(&mut self.bodies, 0.001);

                let app_window = self.environment.as_ref().unwrap();

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
