#![allow(unused_variables, dead_code)]

use crate::core::tracer::RenderMode;
use crate::core::tracer::RenderSettings;
use crate::render::RenderAction;
use crate::render::RenderTargetView;
use crate::render::Renderer;
use std::sync::Arc;
use std::sync::RwLock;

use crate::core::scene::Scene;
use crate::render::RenderTarget;

use egui::CollapsingHeader;
use egui::Color32;
use egui::ComboBox;
use egui::DragValue;
use egui::ScrollArea;
use gui::Editable;
use hatchery::engine::EngineContext;
use hatchery::{
    engine::{Engine, EngineApi, EngineOptions, Hatchery, WindowOptions},
    gui::egui_implementation::EguiImplementation,
};
pub use log::*;
use pipeline::Tonemap;
use pipeline::ViewportPipeline;
use simplelog::*;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::render_pass::Subpass;
use vulkano::{command_buffer::AutoCommandBufferBuilder, pipeline::graphics::viewport::Viewport};
use widgets::FatButton;
use winit::dpi::LogicalSize;

mod common;
mod core;

mod gui;
mod pipeline;
mod render;
mod utils;
mod widgets;

struct VoidrayEngine {
    pipeline: ViewportPipeline,
    settings: Arc<RwLock<RenderSettings>>,
    scene: Arc<RwLock<Scene>>,
    target: Arc<RwLock<RenderTarget>>,
    renderer: Renderer,
}

impl Engine for VoidrayEngine {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        // Initialize logging
        CombinedLogger::init(vec![TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )])
        .expect("failed to initialize logging");

        let dimensions = [1000, 1000];

        let subpass = context.viewport_subpass();
        let api = context.api_mut();

        let scene = Arc::new(RwLock::new(Scene::default()));
        let target = Arc::new(RwLock::new(RenderTarget::new(&api.context, dimensions)));
        let target_view = Arc::new(RenderTargetView::new(&api.context, dimensions));
        let settings = Arc::new(RwLock::new(RenderSettings::default()));

        Self {
            pipeline: ViewportPipeline::new(
                api.graphics_queue(),
                subpass,
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

        #[cfg(feature = "high_precision")]
        info!("Compiled with high_precision");

        #[cfg(not(feature = "high_precision"))]
        info!("Compiled with low_precision");
    }

    fn immediate(&mut self, context: &mut egui::Context, api: &mut EngineApi) {
        egui::TopBottomPanel::top("top_panel").show(context, |ui| {
            ui.label("top panel");
        });

        egui::SidePanel::left("left_panel")
            .min_width(200.0)
            .resizable(false)
            .show(context, |ui| {
                let currently_rendering = self.renderer.currently_rendering();
                let mut settings = self.settings.write().unwrap();
                // let mut modified = false;

                let col_width = 100.0;

                CollapsingHeader::new("Render Settings")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add_enabled_ui(!currently_rendering, |ui| {
                            egui::Grid::new("render_settings")
                                .num_columns(2)
                                .spacing([10.0, 4.0])
                                .max_col_width(col_width)
                                .min_col_width(col_width)
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label("Total samples:");
                                    ui.add(
                                        DragValue::new(&mut settings.samples_per_pixel).speed(1),
                                    );
                                    ui.end_row();
                                    ui.label("Samples per run:");
                                    ui.add(DragValue::new(&mut settings.samples_per_run).speed(1));
                                    ui.end_row();
                                    ui.label("Max ray depth:");
                                    ui.add(DragValue::new(&mut settings.max_ray_depth).speed(1));
                                    ui.end_row();
                                    ui.label("Render mode:");
                                    ComboBox::from_id_source("render_mode")
                                        .selected_text(format!("{:?}", settings.render_mode))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut settings.render_mode,
                                                RenderMode::Full,
                                                format!("{:?}", RenderMode::Full),
                                            );
                                            ui.selectable_value(
                                                &mut settings.render_mode,
                                                RenderMode::Normal,
                                                format!("{:?}", RenderMode::Normal),
                                            );
                                        });
                                    ui.end_row();
                                });
                        });
                    });
                ui.add_space(15.0);

                CollapsingHeader::new("Color Management")
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::Grid::new("color_management")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .max_col_width(col_width)
                            .min_col_width(col_width)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Tone map:");
                                ComboBox::from_id_source("tonemap")
                                    .selected_text(format!("{:?}", settings.tonemap))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut settings.tonemap,
                                            Tonemap::None,
                                            format!("{:?}", Tonemap::None),
                                        );
                                        ui.selectable_value(
                                            &mut settings.tonemap,
                                            Tonemap::SimpleACES,
                                            format!("{:?}", Tonemap::SimpleACES),
                                        );
                                        ui.selectable_value(
                                            &mut settings.tonemap,
                                            Tonemap::SimpleReinhard,
                                            format!("{:?}", Tonemap::SimpleReinhard),
                                        );
                                        ui.selectable_value(
                                            &mut settings.tonemap,
                                            Tonemap::LumaReinhard,
                                            format!("{:?}", Tonemap::LumaReinhard),
                                        );
                                        ui.selectable_value(
                                            &mut settings.tonemap,
                                            Tonemap::LumaWhitePreservingReinhard,
                                            format!("{:?}", Tonemap::LumaWhitePreservingReinhard),
                                        );
                                        ui.selectable_value(
                                            &mut settings.tonemap,
                                            Tonemap::RomBinDaHouse,
                                            format!("{:?}", Tonemap::RomBinDaHouse),
                                        );
                                        ui.selectable_value(
                                            &mut settings.tonemap,
                                            Tonemap::Filmic,
                                            format!("{:?}", Tonemap::Filmic),
                                        );
                                        ui.selectable_value(
                                            &mut settings.tonemap,
                                            Tonemap::Uncharted2,
                                            format!("{:?}", Tonemap::Uncharted2),
                                        );
                                    });
                                ui.end_row();
                                ui.label("Gamma");
                                ui.add(DragValue::new(&mut settings.gamma).speed(0.02));
                                ui.end_row();
                                ui.label("Exposure:");
                                ui.add(DragValue::new(&mut settings.exposure).speed(0.02));
                                ui.end_row();
                                ui.label("Transparent:");
                                ui.checkbox(&mut settings.transparent, "");
                                ui.end_row();
                            });
                    });
                ui.add_space(15.0);

                // if let Ok(mut scene_write) = self.scene.try_write() {
                //     ui.heading("Camera");
                //     scene_write.camera.display_ui(ui, &mut false);
                // }

                egui::Grid::new("render_actions")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .max_col_width(col_width)
                    .min_col_width(col_width)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.add_enabled_ui(!currently_rendering, |ui| {
                            ui.horizontal_centered(|ui| {
                                if ui.add(FatButton::new("Render").width(115.0)).clicked() {
                                    self.renderer.execute(RenderAction::Start);
                                }
                            });
                        });
                        ui.add_enabled_ui(currently_rendering, |ui| {
                            ui.horizontal_centered(|ui| {
                                if ui.add(FatButton::new("Cancel").width(115.0)).clicked() {
                                    self.renderer.execute(RenderAction::Cancel);
                                }
                            });
                        });
                        ui.end_row();
                    });

                let samples = self.renderer.sample_count();
                if currently_rendering {
                    ui.label(format!("Samples: {}/{}", samples.0, samples.1));
                }
            });

        egui::SidePanel::right("right_panel")
            .min_width(200.0)
            .max_width(200.0)
            .show(context, |ui| {});

        // egui::TopBottomPanel::bottom("bottom_panel")
        //     .min_height(200.0)
        //     .show(context, |ui| {
        //         egui::ScrollArea::vertical()
        //             .auto_shrink([false, false])
        //             .stick_to_bottom()
        //             .show(ui, |ui| {
        //                 for _ in 0..10 {
        //                     ui.label("[---] console not yet supported...");
        //                 }
        //             });
        //     });
    }

    // fn render(
    //     &mut self,
    //     subpass: vulkano::render_pass::Subpass,
    //     viewport: Viewport,
    //     api: &mut EngineApi,
    // ) -> SecondaryAutoCommandBuffer {
    //     let secondary_builder = AutoCommandBufferBuilder::secondary(
    //         api.context.device(),
    //         api.context.graphics_queue().family(),
    //         CommandBufferUsage::MultipleSubmit,
    //         CommandBufferInheritanceInfo {
    //             render_pass: Some(subpass.into()),
    //             ..Default::default()
    //         },
    //     )
    //     .unwrap();
    //     secondary_builder.build().unwrap()
    // }

    fn render(
        &mut self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        subpass: Subpass,
        viewport: Viewport,
        api: &mut EngineApi,
    ) {
        self.pipeline
            .draw(command_buffer, &self.settings.read().unwrap(), viewport);
    }

    // fn render(
    //     &mut self,
    //     builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
    //     viewport: Viewport,
    // ) {
    //     let samples = self.renderer.sample_count();
    //     self.pipeline
    //         .draw(builder, &self.settings.read().unwrap(), viewport);
    // }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions {
            title: "Voidray Engine",
            dimensions: LogicalSize::new(1400, 1000),
        },
        ..EngineOptions::default()
    };

    Hatchery::<VoidrayEngine>::run(options);
}
