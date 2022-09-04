#![allow(unused_variables, dead_code)]

use crate::render::RenderAction;
use crate::render::RenderTargetView;
use crate::render::Renderer;
use crate::core::tracer::RenderSettings;
use std::sync::Arc;
use std::sync::RwLock;

use crate::render::RenderTarget;
use crate::core::scene::Scene;

use hatchery::{
    engine::{Engine, EngineApi, EngineOptions, Hatchery, WindowOptions},
    gui::egui_implementation::EguiImplementation,
};
pub use log::*;
use simplelog::*;
use pipeline::ViewportPipeline;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer},
    pipeline::graphics::viewport::Viewport,
};
use winit::dpi::LogicalSize;

mod core;
mod common;

mod render;
mod pipeline;
mod utils;

struct VoidrayEngine {
    pipeline: ViewportPipeline,
    settings: Arc<RwLock<RenderSettings>>,
    scene: Arc<RwLock<Scene>>,
    target: Arc<RwLock<RenderTarget>>,
    renderer: Renderer,
}

impl Engine<EguiImplementation> for VoidrayEngine {
    fn init(api: &mut EngineApi) -> Self {
        // Initialize logging
        CombinedLogger::init(vec![TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )])
        .expect("failed to initialize logging");

        let dimensions = [100, 100];

        let scene = Arc::new(RwLock::new(Scene::default()));
        let target = Arc::new(RwLock::new(RenderTarget::new(&api.context, dimensions)));
        let target_view = Arc::new(RenderTargetView::new(&api.context, dimensions));
        let settings = Arc::new(RwLock::new(RenderSettings::default()));

        Self {
            pipeline: ViewportPipeline::new(
                api.graphics_queue(),
                api.viewport_subpass(),
                target.clone(),
                target_view,
            ),
            settings: settings.clone(),
            scene: scene.clone(),
            target: target.clone(),
            renderer: Renderer::new(api.device(), api.compute_queue(), scene, target, settings),
        }
    }

    fn start(&mut self, api: &mut EngineApi) {
        info!(
            "Using {:?} card: {}",
            api.context.device_type(),
            api.context.device_name()
        );
    }

    fn immediate(&mut self, context: &mut egui::Context, api: &mut EngineApi) {
        egui::TopBottomPanel::top("top_panel").show(context, |ui| {
            ui.label("top panel");
        });

        egui::SidePanel::left("left_panel")
            .min_width(200.0)
            .show(context, |ui| {
                ui.heading("Render Settings");
                ui.separator();

                let currently_rendering = self.renderer.currently_rendering();
                let mut settings = self.settings.write().unwrap();

                ui.add_enabled_ui(!currently_rendering, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Samples:");
                        ui.add(egui::Slider::new(&mut settings.samples_per_pixel, 1..=1000));
                    });
                    let samples_per_pixel = settings.samples_per_pixel;
                    ui.horizontal(|ui| {
                        ui.label("Samples per run:");
                        ui.add(egui::Slider::new(&mut settings.samples_per_run, 1..=samples_per_pixel));
                    });
                });

                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!currently_rendering, |ui| {
                        if ui.button("Render").clicked() {
                            self.renderer.execute(RenderAction::Start);
                        }
                    });
                    ui.add_enabled_ui(currently_rendering, |ui| {
                        if ui.button("Cancel").clicked() {
                            self.renderer.execute(RenderAction::Cancel);
                        }
                    });
                });

                let samples = self.renderer.sample_count();
                if currently_rendering {
                    ui.label(format!("Samples: {}/{}", samples.0, samples.1));
                }
            });

        egui::SidePanel::right("right_panel")
            .min_width(200.0)
            .max_width(200.0)
            .show(context, |ui| {

            });
    }

    fn render(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        viewport: Viewport,
    ) {
        let samples = self.renderer.sample_count();
        self.pipeline.draw(builder, viewport);
    }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions {
            title: "Voidray Engine",
            dimensions: LogicalSize::new(1400, 1000),
        },
        ..EngineOptions::default()
    };

    Hatchery::<VoidrayEngine, _>::run(options);
}
