use voidray_launcher::Engine;
use voidray_launcher::EngineApi;
use voidray_renderer::camera::Camera;
use voidray_renderer::render::renderer::RenderAction;
use voidray_renderer::settings::ColorManagementSettings;
use voidray_renderer::settings::RenderMode;
use voidray_renderer::settings::RenderSettings;
use voidray_renderer::settings::Tonemap;
use voidray_renderer::vector::PI;
use voidray_renderer::vector::Vec3;

use crate::egui::*;
use crate::utils::human_duration;
use crate::widgets::FatButton;
use crate::VoidrayEngine;

pub trait Editable {
    fn display_ui(&mut self, ui: &mut Ui, modified: &mut bool, enabled: bool);
}

pub fn engine_ui(engine: &mut VoidrayEngine, context: &mut Context, api: &mut EngineApi) {
    TopBottomPanel::top("top_panel").show(context, |ui| {
        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {});

            ui.menu_button("About", |ui| {});

            ui.with_layout(Layout::right_to_left(), |ui| {
                ui.label(format!(
                    "Voidray Engine - v{}",
                    option_env!("CARGO_PKG_VERSION").unwrap_or(" unknown")
                ));
            });
        });
    });

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
                ui.add(ProgressBar::new(samples.0 as f32 / samples.1 as f32)
                    .show_percentage());
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
            engine.scene.write().unwrap().camera.display_ui(ui, &mut modified, true);
        });
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
                        .max_col_width(80.0)
                        .min_col_width(140.0)
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
                                    .speed(0.01)
                            );
                            ui.end_row();
                            
                            if let Some(mut dof) = self.dof {
                                ui.label("Focal length:");
                                ui.add(
                                    DragValue::new(&mut dof.0)
                                        .fixed_decimals(2)
                                        .speed(0.1)
                                );
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
            ui.add(
                DragValue::new(&mut self.x)
                    .fixed_decimals(2)
                    .speed(0.1)
            );
            ui.add(
                DragValue::new(&mut self.y)
                    .fixed_decimals(2)
                    .speed(0.1)
            );
            ui.add(
                DragValue::new(&mut self.z)
                    .fixed_decimals(2)
                    .speed(0.1)
            );
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
