use egui::Context;
use egui_wgpu::Renderer;
use egui_winit::State;

pub struct UiRenderer {
    pub context: Context,
    pub state: State,
    pub renderer: Renderer,
}

impl UiRenderer {
    pub fn new(
        context: egui::Context,
        window: &winit::window::Window,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Self {

        let state = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            None,
            None,
            None,
        );


        let renderer = egui_wgpu::Renderer::new(
            device,
            format,
            egui_wgpu::RendererOptions {
                ..Default::default()
            },
        );


        Self {
            context,
            state,
            renderer,
        }
    }

}
