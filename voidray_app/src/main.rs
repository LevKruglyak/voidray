#![allow(dead_code, unused_variables)]

use voidray_launcher::gui_implementation::*;
use voidray_launcher::*;
use voidray_renderer::preamble::*;
use voidray_renderer::render::target::CpuRenderTarget;
use voidray_renderer::render::viewport::Viewport;

struct VoidrayEngine {
    target: Arc<RwLock<CpuRenderTarget>>,
    viewport: Viewport,
}

impl Engine for VoidrayEngine {
    type Gui = gui_implementation::EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        let api = context.api();
        let target = CpuRenderTarget::new(api.compute_queue(), [1000, 1000]);

        Self {
            target: target.clone(),
            viewport: Viewport::new(api.graphics_queue(), context.viewport_subpass(), target),
        }
    }

    fn immediate(
        &mut self,
        context: &mut <<Self as Engine>::Gui as GuiImplementation>::Context,
        api: &mut EngineApi,
    ) {
        egui::Window::new("Hello, World! ").show(context, |ui| {
            ui.label("This is a test of the window!");
        });
    }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions {
            title: "Voidray Engine",
            dimensions: LogicalSize::new(1200, 1000),
        },
        ..EngineOptions::default()
    };

    EngineLauncher::<VoidrayEngine>::run(options);
}
