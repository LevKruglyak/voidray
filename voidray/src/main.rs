use hatchery::{
    engine::{Engine, EngineApi, EngineOptions, Hatchery, WindowOptions},
    gui::{egui_implementation::EguiImplementation, imgui_implementation::ImguiImplementation},
};
use imgui::Context;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer},
    pipeline::graphics::viewport::Viewport,
};
use winit::dpi::LogicalSize;

mod graphics;
// mod test_pipeline;

#[derive(Default)]
struct VoidrayEngine {
    test: f32,
}

impl Engine<EguiImplementation> for VoidrayEngine {
    fn immediate(&mut self, context: &mut egui::Context, api: &mut EngineApi) {
        egui::SidePanel::left("left_panel").show(context, |ui| {
            ui.heading("Side panel");
        });

        egui::Window::new("Testing").show(context, |ui| {
            ui.label("Hello, World!");
            ui.label(format!("Device: {}", api.device_name()));
            ui.add(egui::Slider::new(&mut self.test, 0.0..=1.0));
        });
    }

    fn render(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        viewport: Viewport,
    ) {

    }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions {
            title: "Voidray Engine",
            dimensions: LogicalSize::new(800, 800),
        },
        ..EngineOptions::default()
    };

    let engine = VoidrayEngine::default();

    Hatchery::run(options, engine);
}
