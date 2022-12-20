use voidray_launcher::EngineApi;
use voidray_renderer::camera::Camera;
use voidray_renderer::render::renderer::RenderAction;
use voidray_renderer::scene::Scene;
use voidray_renderer::settings::ColorManagementSettings;
use voidray_renderer::settings::RenderMode;
use voidray_renderer::settings::RenderSettings;
use voidray_renderer::settings::Settings;
use voidray_renderer::settings::Tonemap;
use voidray_renderer::vec3;
use voidray_renderer::vector::Vec3;
use voidray_renderer::vector::PI;

use crate::egui::*;
use crate::examples::cornell;
use crate::examples::material;
use crate::examples::mushroom;
use crate::examples::spheres;
use crate::utils::human_duration;
use crate::widgets::FatButton;
use crate::VoidrayEngine;

pub struct GuiState {
    pub startup: bool,
    pub demo: DemoScene,
}

impl Default for GuiState {
    fn default() -> Self {
        Self { startup: true, demo: DemoScene::None, }
    }
}

pub trait Editable {
    fn display_ui(&mut self, ui: &mut Ui, modified: &mut bool, enabled: bool);
}

#[derive(PartialEq)]
pub enum DemoScene {
    None,
    Spheres,
    Cornell,
    Mushroom,
    Material,
}

pub fn engine_ui(engine: &mut VoidrayEngine, context: &mut Context, api: &mut EngineApi) {
    TopBottomPanel::top("top_panel").show(context, |ui| {
        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Start page").clicked() {
                    engine.target.force_clear();
                    engine.state.startup = true;
                }
            });

            ui.menu_button("About", |ui| {});

            ui.with_layout(Layout::right_to_left(), |ui| {
                ui.label(format!(
                    "Voidray Engine - v{}",
                    option_env!("CARGO_PKG_VERSION").unwrap_or(" unknown")
                ));
            });
        });
    });

    if !engine.state.startup {
        SidePanel::left("left_panel")
            .min_width(250.0)
            .resizable(false)
            .show(context, |ui| {
                let currently_rendering = engine.renderer.currently_rendering();
                let mut modified = false;

                {
                    let mut settings = engine.settings.write().unwrap();
                    settings
                        .render
                        .display_ui(ui, &mut modified, !currently_rendering);
                    settings
                        .color_management
                        .display_ui(ui, &mut modified, true);
                }
                render_actions(engine, ui, currently_rendering);

                let samples = engine.renderer.samples();
                let time = engine.renderer.elapsed_time();
                let remaining = engine.renderer.remaining_time();

                ui.add_space(5.0);
                if let Some(remaining) = remaining {
                    ui.add(ProgressBar::new(samples.0 as f32 / samples.1 as f32).show_percentage());
                    ui.add_space(5.0);
                }
                ui.label(format!("Samples: {}/{}", samples.0, samples.1));
                ui.label(format!("Elapsed time: {}", human_duration(&time)));
                if let Some(remaining) = remaining {
                    ui.label(format!("Remaining time: {}", human_duration(&remaining)));
                }
            });

        SidePanel::right("right_panel")
            .min_width(250.0)
            .resizable(false)
            .show(context, |ui| {
                let mut modified = false;
                engine
                    .scene
                    .write()
                    .unwrap()
                    .camera
                    .display_ui(ui, &mut modified, true);
            });
    }

    let startup = &mut engine.state.startup;
    if *startup {
        // CentralPanel::default().show(context, |ui| {});
        Window::new("")
            .title_bar(false)
            .min_width(400.0)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(context, |ui| {
                Grid::new("center_panel").num_columns(1).show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.heading("Voidray Engine");
                    });
                    ui.end_row();
                    ui.centered_and_justified(|ui| {
                        ui.label(format!(
                            "Version {}",
                            option_env!("CARGO_PKG_VERSION").unwrap_or(" unknown")
                        ));
                    });
                    ui.end_row();

                    ui.end_row();
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(
                            &mut engine.state.demo,
                            DemoScene::None,
                            "None",
                        );
                        ui.selectable_value(
                            &mut engine.state.demo,
                            DemoScene::Spheres,
                            "Spheres",
                        );
                        ui.selectable_value(
                            &mut engine.state.demo,
                            DemoScene::Cornell,
                            "Cornell",
                        );
                        ui.selectable_value(
                            &mut engine.state.demo,
                            DemoScene::Mushroom,
                            "Mushroom",
                        );
                        ui.selectable_value(
                            &mut engine.state.demo,
                            DemoScene::Material,
                            "Material",
                        );
                    });
                    ui.end_row();
                    ui.end_row();

                    let width = 200.0;
                    Grid::new("render_actions")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .max_col_width(width)
                        .min_col_width(width)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.horizontal_centered(|ui| {
                                if ui.add(FatButton::new("Load").width(width)).clicked() {
                                    let (scene, settings, dimensions) = match engine.state.demo {
                                        DemoScene::None => (Scene::empty(), Settings::default(), [1000, 1000]),
                                        DemoScene::Spheres => spheres::scene(),
                                        DemoScene::Cornell => cornell::scene(),
                                        DemoScene::Mushroom => mushroom::scene(),
                                        DemoScene::Material => material::scene(),
                                    };

                                    *engine.scene.write().unwrap() = scene;
                                    *engine.settings.write().unwrap() = settings;
                                    engine.target.resize(dimensions);
                
                                    *startup = false;
                                }
                            });
                            ui.horizontal_centered(|ui| {
                                if ui.add(FatButton::new("Close").width(width)).clicked() {
                                    *startup = false;
                                }
                            });
                            ui.end_row();
                        });
                });
            });
    }
}

impl Editable for Camera {
    fn display_ui(&mut self, ui: &mut Ui, modified: &mut bool, enabled: bool) {
        CollapsingHeader::new("Camera")
            .default_open(true)
            .show(ui, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    Grid::new("render_settings")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .max_col_width(125.0)
                        .min_col_width(125.0)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Position:");
                            self.eye.display_ui(ui, modified, true);
                            ui.end_row();
                            ui.label("Direction:");
                            self.direction.display_ui(ui, modified, true);
                            ui.end_row();
                            ui.label("Up:");
                            self.up.display_ui(ui, modified, true);
                            ui.end_row();

                            ui.label("Fov:");
                            ui.add(
                                DragValue::new(&mut self.fov)
                                    .fixed_decimals(2)
                                    .clamp_range(0.0..=PI)
                                    .speed(0.01),
                            );
                            ui.end_row();

                            let mut dof_enabled = self.dof.is_some();
                            ui.label("Depth of field:");
                            ui.checkbox(&mut dof_enabled, "");
                            ui.end_row();
    
                            if dof_enabled && self.dof.is_none() {
                                self.dof = Some((0.0, vec3!(0.0)));
                            }

                            if !dof_enabled && self.dof.is_some() {
                                self.dof = None;
                            }

                            if let Some(mut dof) = self.dof {
                                ui.label("Focal length:");
                                ui.add(DragValue::new(&mut dof.0).fixed_decimals(2).speed(0.01).clamp_range(0.0..=10.0));
                                ui.end_row();
                                ui.label("Focal point:");
                                dof.1.display_ui(ui, modified, true);
                                ui.end_row();
                                self.dof = Some(dof);
                            }
                        });
                });
            });
        ui.add_space(15.0);
    }
}

impl Editable for Vec3 {
    fn display_ui(&mut self, ui: &mut Ui, modified: &mut bool, enabled: bool) {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.x).max_decimals(2).speed(0.1));
            ui.add(DragValue::new(&mut self.y).max_decimals(2).speed(0.1));
            ui.add(DragValue::new(&mut self.z).max_decimals(2).speed(0.1));
        });
    }
}

impl Editable for RenderSettings {
    fn display_ui(&mut self, ui: &mut Ui, modified: &mut bool, enabled: bool) {
        CollapsingHeader::new("Render Settings")
            .default_open(true)
            .show(ui, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    Grid::new("render_settings")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .max_col_width(110.0)
                        .min_col_width(110.0)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Total samples:");
                            ui.add(
                                DragValue::new(&mut self.total_samples)
                                    .speed(1)
                                    .clamp_range(1..=u32::max_value()),
                            );
                            ui.end_row();
                            ui.label("Update frequency:");
                            ui.add(
                                DragValue::new(&mut self.update_frequency)
                                    .speed(0.05)
                                    .clamp_range(0.05..=10.0),
                            );
                            ui.end_row();
                            ui.label("Max ray bounces:");
                            ui.add(
                                DragValue::new(&mut self.max_bounces)
                                    .speed(1)
                                    .clamp_range(0..=255),
                            );
                            ui.end_row();
                            ui.label("Render mode:");
                            ComboBox::from_id_source("render_mode")
                                .selected_text(format!("{:?}", self.render_mode))
                                .width(110.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.render_mode,
                                        RenderMode::Full,
                                        format!("{:?}", RenderMode::Full),
                                    );
                                    ui.selectable_value(
                                        &mut self.render_mode,
                                        RenderMode::Normal,
                                        format!("{:?}", RenderMode::Normal),
                                    );
                                });
                            ui.end_row();
                        });
                });
            });
        ui.add_space(15.0);
    }
}

impl Editable for ColorManagementSettings {
    fn display_ui(&mut self, ui: &mut Ui, modified: &mut bool, enabled: bool) {
        CollapsingHeader::new("Color Management")
            .default_open(true)
            .show(ui, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    Grid::new("render_settings")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .max_col_width(110.0)
                        .min_col_width(110.0)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Tonemap:");
                            ComboBox::from_id_source("tonemap")
                                .selected_text(format!("{:?}", self.tonemap))
                                .width(110.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.tonemap,
                                        Tonemap::None,
                                        format!("{:?}", Tonemap::None),
                                    );
                                    ui.selectable_value(
                                        &mut self.tonemap,
                                        Tonemap::ACES,
                                        format!("{:?}", Tonemap::ACES),
                                    );
                                    ui.selectable_value(
                                        &mut self.tonemap,
                                        Tonemap::Reinhard,
                                        format!("{:?}", Tonemap::Reinhard),
                                    );
                                    ui.selectable_value(
                                        &mut self.tonemap,
                                        Tonemap::Filmic,
                                        format!("{:?}", Tonemap::Filmic),
                                    );
                                    ui.selectable_value(
                                        &mut self.tonemap,
                                        Tonemap::Uncharted2,
                                        format!("{:?}", Tonemap::Uncharted2),
                                    );
                                });
                            ui.end_row();
                            ui.label("Gamma");
                            ui.add(
                                DragValue::new(&mut self.gamma)
                                    .speed(0.02)
                                    .clamp_range(0.1..=5.0),
                            );
                            ui.end_row();
                            ui.label("Exposure:");
                            ui.add(
                                DragValue::new(&mut self.exposure)
                                    .speed(0.02)
                                    .clamp_range(-16.0..=16.0),
                            );
                            ui.end_row();
                            ui.label("Transparent:");
                            ui.checkbox(&mut self.transparent, "");
                            ui.end_row();
                        });
                });
            });
        ui.add_space(15.0);
    }
}

pub fn render_actions(engine: &mut VoidrayEngine, ui: &mut Ui, rendering: bool) {
    let width = 125.0;
    Grid::new("render_actions")
        .num_columns(2)
        .spacing([10.0, 4.0])
        .max_col_width(width)
        .min_col_width(width)
        .striped(true)
        .show(ui, |ui| {
            ui.add_enabled_ui(!rendering, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.add(FatButton::new("Render").width(width)).clicked() {
                        engine.renderer.execute(RenderAction::Render);
                    }
                });
            });
            ui.add_enabled_ui(rendering, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.add(FatButton::new("Cancel").width(width)).clicked() {
                        engine.renderer.execute(RenderAction::Cancel);
                    }
                });
            });
            ui.end_row();
        });
}
