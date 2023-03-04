#![allow(dead_code, unused_variables)]

use gui::{engine_ui, GuiState};
use voidray_launcher::gui_implementation::*;
use voidray_launcher::*;
use voidray_renderer::preamble::*;
use voidray_renderer::render::post_process::PostProcessingData;
use voidray_renderer::render::renderer::Renderer;
use voidray_renderer::render::target::CpuRenderTarget;
use voidray_renderer::render::viewport::Viewport;
use voidray_renderer::scene::Scene;
use voidray_renderer::settings::Settings;
use voidray_renderer::vulkano::command_buffer::{
    AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
};
use voidray_renderer::vulkano::device::Features;
use voidray_renderer::vulkano::pipeline::graphics;
use voidray_renderer::vulkano::render_pass::Subpass;

mod examples;
mod gui;
mod utils;
mod widgets;

pub struct VoidrayEngine {
    pub target: Arc<CpuRenderTarget>,
    pub scene: Arc<RwLock<Scene>>,
    pub settings: Arc<RwLock<Settings>>,
    pub renderer: Renderer,
    pub state: GuiState,
    viewport: Viewport,
}

impl Engine for VoidrayEngine {
    type Gui = gui_implementation::EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        let api = context.api();
        let target = CpuRenderTarget::new(api.compute_queue(), 2, [500, 500]);
        let scene = Scene::empty();

        let scene = Arc::new(RwLock::new(scene));
        let settings = Arc::new(RwLock::new(Settings::default()));

        Self {
            target: target.clone(),
            scene: scene.clone(),
            settings: settings.clone(),
            viewport: Viewport::new(
                api.graphics_queue(),
                api.compute_queue(),
                context.viewport_subpass(),
                target.clone(),
            ),
            state: GuiState::default(),
            renderer: Renderer::new(api.compute_queue(), scene, settings, target),
        }
    }

    fn immediate(
        &mut self,
        context: &mut <<Self as Engine>::Gui as GuiImplementation>::Context,
        api: &mut EngineApi,
    ) {
        engine_ui(self, context, api);
    }

    fn render(
        &mut self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        subpass: Subpass,
        viewport: graphics::viewport::Viewport,
        api: &mut EngineApi,
    ) {
        let samples = self.renderer.samples();
        let mut scale = samples.1 as f32 / samples.0 as f32;
        if !scale.is_normal() {
            scale = 0.0;
        }

        let settings = self.settings.read().unwrap();
        let data = Some(PostProcessingData {
            scale,
            exposure: settings.color_management.exposure,
            gamma: settings.color_management.gamma,
            tonemap: settings.color_management.tonemap.as_i32(),
        });

        self.viewport.draw(command_buffer, viewport, data);
    }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions {
            title: "Voidray Engine",
            dimensions: LogicalSize::new(1500, 1000),
        },
        ..Default::default()
    };

    EngineLauncher::<VoidrayEngine>::run(options);
}
