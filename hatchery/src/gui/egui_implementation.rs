use std::sync::Arc;

use super::GuiImplementation;
use egui::Context;
use egui_winit_vulkano::Gui;
use vulkano::{
    command_buffer::SecondaryAutoCommandBuffer, device::Queue,
    pipeline::graphics::viewport::Viewport, render_pass::Subpass, swapchain::Surface,
};
use winit::{event::WindowEvent, window::Window};

pub struct EguiImplementation {
    gui: Gui,
}

impl EguiImplementation {
    pub fn context(&self) -> Context {
        self.gui.context()
    }
}

impl GuiImplementation for EguiImplementation {
    type Context = Context;

    fn new(surface: Arc<Surface<Window>>, graphics_queue: Arc<Queue>, subpass: Subpass) -> Self {
        Self {
            gui: Gui::new_with_subpass(surface, graphics_queue, subpass),
        }
    }

    fn viewport(&self, scale_factor: f32) -> Viewport {
        let context = self.context();

        let origin = context.available_rect().left_top();
        let dimensions = context.available_rect().right_bottom() - origin;

        Viewport {
            origin: [origin.x * scale_factor, origin.y * scale_factor],
            dimensions: [dimensions.x * scale_factor, dimensions.y * scale_factor],
            depth_range: 0.0..1.0,
        }
    }

    fn immediate(&mut self, ui: impl FnOnce(&mut Context)) {
        self.gui.immediate_ui(|context| {
            ui(&mut context.context());
        });
    }

    fn update(&mut self, event: &WindowEvent) -> bool {
        self.gui.update(event)
    }

    fn render(&mut self, dimensions: [u32; 2]) -> SecondaryAutoCommandBuffer {
        self.gui.draw_on_subpass_image(dimensions)
    }
}
