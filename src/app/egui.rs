use egui_wgpu::Renderer;
use egui_winit::State;

pub struct EguiImpl {
    pub context: egui::Context,
    pub state: State,
    pub renderer: Renderer,
}

impl EguiImpl {
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        window: &winit::window::Window,
    ) -> Self {
        let context = egui::Context::default();
        // Match the winit 0.30 initialization parameters
        let state = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            None,
            None,
            None
        );

        let renderer = Renderer::new(
            device,
            output_format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: true,
                predictable_texture_filtering: true
            }
        );

        Self { context, state, renderer }
    }
}