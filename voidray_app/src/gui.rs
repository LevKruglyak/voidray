use voidray_launcher::Engine;
use voidray_launcher::EngineApi;
use voidray_renderer::render::renderer::RenderAction;
use voidray_renderer::settings::RenderMode;
use voidray_renderer::settings::RenderSettings;

use crate::VoidrayEngine;
use crate::egui::*;
use crate::widgets::FatButton;

pub trait Editable {
    fn display_ui(&mut self, ui: &mut Ui, modified: &mut bool, enabled: bool); 
}


pub fn engine_ui(
    engine: &mut VoidrayEngine,
    context: &mut Context,
    api: &mut EngineApi,
) {
    TopBottomPanel::top("top_panel")
        .show(context, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {

                });

                ui.menu_button("About", |ui| {

                });

                ui.with_layout(Layout::right_to_left(), |ui| {
                    ui.label(format!("Voidray Engine - v{}", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")));
                });
            });
        });

    SidePanel::left("left_panel")
        .min_width(250.0)
        .resizable(false)
        .show(context, |ui| {
            let currently_rendering = engine.renderer.read().unwrap().currently_rendering();
            let mut modified = false;

            {
                let mut settings = engine.settings.write().unwrap();
               settings.render.display_ui(ui, &mut modified, !currently_rendering);
            }
            render_actions(engine, ui, currently_rendering);
        
            let samples = engine.renderer.read().unwrap().samples();

            ui.add_space(5.0);
            ui.label(format!("Samples: {}/{}", samples.0, samples.1));
            // ui.label(format!("Elapsed time: {:.4}s", time.as_secs_f32()));
        });
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
                                DragValue::new(&mut self.total_samples).speed(1),
                            );
                            ui.end_row();
                            ui.label("Update frequency:");
                            ui.add(DragValue::new(&mut self.update_frequency).speed(1));
                            ui.end_row();
                            ui.label("Max ray bounces:");
                            ui.add(DragValue::new(&mut self.max_bounces).speed(1));
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
                        engine.renderer.write().unwrap().execute(RenderAction::Render);
                    }
                });
            });
            ui.add_enabled_ui(rendering, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.add(FatButton::new("Cancel").width(width)).clicked() {
                        engine.renderer.write().unwrap().execute(RenderAction::Cancel);
                    }
                });
            });
            ui.end_row();
        });
}
