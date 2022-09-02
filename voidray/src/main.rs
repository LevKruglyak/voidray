use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crate::core::render::render;
use crate::core::render::RenderTarget;
use crate::core::scene::Scene;
use crate::core::render::RenderSettings;

use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use crossbeam::channel::bounded;
use hatchery::{
    engine::{Engine, EngineApi, EngineOptions, Hatchery, WindowOptions},
    gui::egui_implementation::EguiImplementation,
};
pub use log::*;
use simplelog::*;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer},
    pipeline::graphics::viewport::Viewport,
};
use winit::dpi::LogicalSize;
use viewport_pipeline::ViewportPipeline;

mod core;
mod viewport_pipeline;

enum RenderAction {
    Start,
    Cancel,
}

struct Renderer {
    currently_rendering: Arc<RwLock<bool>>,
    sample_count: Arc<RwLock<(u32, u32)>>,
    sender: Sender<RenderAction>,
}

impl Renderer {
    fn new(scene: Arc<RwLock<Scene>>, target: Arc<RwLock<RenderTarget>>, settings: Arc<RwLock<RenderSettings>>) -> Self {
        let (sender, receiver) = bounded(0);

        let sample_count = Arc::new(RwLock::new((0, 0)));
        let currently_rendering = Arc::new(RwLock::new(false));

        let thread_sample_count = sample_count.clone();
        let thread_currently_rendering = currently_rendering.clone();

        thread::spawn(move || {
            loop {
                if let Ok(RenderAction::Start) = receiver.try_recv() {
                    *thread_currently_rendering.write().unwrap() = true;

                    let settings = {
                        let settings = settings.read().unwrap();
                        settings.clone()
                    };

                    // Perform the render
                    if settings.poll_for_canel {
                        for sample in 0..=settings.samples_per_pixel {
                            if let Ok(RenderAction::Cancel) = receiver.try_recv() {
                                break;
                            }

                            thread::sleep(Duration::from_millis(1));
                            *thread_sample_count.write().unwrap() = (sample, settings.samples_per_pixel);
                        }
                    } else {
                        thread::sleep(Duration::from_millis(settings.samples_per_pixel as u64));
                    }

                    *thread_currently_rendering.write().unwrap() = false;
                }

                // Prevent polling all the time
                thread::sleep(Duration::from_millis(25));
            }
        });

        Self {
            currently_rendering,
            sample_count,
            sender,
        }
    }

    fn execute(&self, action: RenderAction) {
        self.sender.send(action).unwrap();
    }

    fn currently_rendering(&self) -> bool {
        *self.currently_rendering.read().unwrap()
    }

    fn sample_count(&self) -> (u32, u32) {
        *self.sample_count.read().unwrap()
    }
}

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

        let scene = Arc::new(RwLock::new(Scene::default()));
        let target = Arc::new(RwLock::new(RenderTarget::new(&api.context, [100, 100])));
        let settings = Arc::new(RwLock::new(RenderSettings::default()));

        Self {
            pipeline: ViewportPipeline::new(api.graphics_queue(), api.viewport_subpass()),
            settings: settings.clone(),
            scene: scene.clone(),
            target: target.clone(),
            renderer: Renderer::new(scene, target, settings),
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
        egui::SidePanel::left("left_panel").min_width(200.0).show(context, |ui| {
            ui.heading("Render Settings");
            ui.separator();

            let currently_rendering = self.renderer.currently_rendering();
            let mut settings = self.settings.write().unwrap();

            ui.add_enabled_ui(!currently_rendering, |ui| {

                ui.horizontal(|ui| {
                    ui.label("Samples:");
                    ui.add(egui::Slider::new(&mut settings.samples_per_pixel, 1..=1000));
                });
                ui.horizontal(|ui| {
                    ui.label("Sleep duration:");
                    ui.add(egui::Slider::new(&mut settings.sleep_duration, 1..=10000));
                });
                ui.checkbox(&mut settings.poll_for_canel, "Poll for cancel")
            });

            ui.horizontal(|ui| {
                ui.add_enabled_ui(!currently_rendering, |ui| {
                    if ui.button("Render").clicked() {
                        self.renderer.execute(RenderAction::Start);
                    }
                });
                ui.add_enabled_ui(currently_rendering && settings.poll_for_canel, |ui| {
                    if ui.button("Cancel").clicked() {
                        self.renderer.execute(RenderAction::Cancel);
                    }
                });
            });

            let samples = self.renderer.sample_count();
            ui.label(format!("Samples: {}/{}", samples.0, samples.1));
        });
    }

    fn render(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        viewport: Viewport,
    ) {
        self.pipeline.draw(builder, viewport);
    }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions {
            title: "Voidray Engine",
            dimensions: LogicalSize::new(1200, 800),
        },
        ..EngineOptions::default()
    };

    Hatchery::<VoidrayEngine, _>::run(options);
}
