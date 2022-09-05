use egui::{InnerResponse, Ui};

use crate::core::{Vec3, camera::Camera};

pub trait Editable {
    fn display_ui(&mut self, ui: &mut Ui, modified: &mut bool) -> InnerResponse<()>;
}

impl Editable for Vec3 {
    fn display_ui(&mut self, ui: &mut Ui, modified: &mut bool) -> InnerResponse<()> {
        ui.horizontal(|ui| {
            *modified |= ui
                .add(
                    egui::widgets::DragValue::new(&mut self.x)
                        .speed(0.02)
                        .prefix("x: "),
                )
                .changed();
            *modified |= ui
                .add(
                    egui::widgets::DragValue::new(&mut self.y)
                        .speed(0.02)
                        .prefix("y: "),
                )
                .changed();
            *modified |= ui
                .add(
                    egui::widgets::DragValue::new(&mut self.z)
                        .speed(0.02)
                        .prefix("z: "),
                )
                .changed();
        })
    }
}

impl Editable for Camera {
    fn display_ui(&mut self, ui: &mut Ui, modified: &mut bool) -> InnerResponse<()> {
        ui.group(|ui| {
            ui.label("Look from:");
            self.eye.display_ui(ui, modified);
            ui.label("Look at:");
            self.direction.display_ui(ui, modified);
            ui.label("Vertical:");
            self.up.display_ui(ui, modified);
            ui.label("Field of view:");
            *modified |= ui
                .add(egui::Slider::new(&mut self.fov, 0.60..=120.0))
                .changed();
        })
    }
}
