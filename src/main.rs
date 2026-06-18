
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowId};

mod app_environment;
use crate::app_environment::AppEnvironment;

mod app;
use crate::app::AppData;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}



#[derive(Default)]
struct App {
    environment: Option<AppEnvironment>,
    data: Option<AppData>,
}


impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.environment = Some(AppEnvironment::new(&event_loop));
        self.data = Some(AppData::new(&self.environment.as_ref().unwrap().device, &self.environment.as_ref().unwrap().surface_desc));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                self.environment.as_mut().unwrap().window.request_redraw();
                let environment = self.environment.as_ref().unwrap();

                let frame = match environment.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {e:?}");
                        return;
                    }
                }; 

                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                self.data.as_mut().unwrap().render(&environment.device, &environment.queue, &view);
                frame.present();
            },
            _ => (),
        }

    }
}
