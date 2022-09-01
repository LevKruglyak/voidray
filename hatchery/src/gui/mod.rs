use std::sync::Arc;

use vulkano::{
    command_buffer::SecondaryAutoCommandBuffer, device::Queue, format::Format,
    pipeline::graphics::viewport::Viewport, render_pass::Subpass, swapchain::Surface,
};
use winit::{event::WindowEvent, window::Window};

#[cfg(feature = "egui")]
pub mod egui_implementation;

// #[cfg(feature = "imgui")]
pub mod imgui_implementation;

// An abstraction layer over the gui library to allow for easier switching between two libraries

/// Represents an arbitrary immediate mode gui implementation such as imgui-rs or egui
pub trait GuiImplementation {
    fn new(surface: Arc<Surface<Window>>, graphics_queue: Arc<Queue>, subpass: Subpass) -> Self;

    fn render(&mut self, dimensions: [u32; 2]) -> SecondaryAutoCommandBuffer;

    type Context;

    /// Rebuild the ui
    fn immediate(&mut self, ui: impl FnOnce(&mut Self::Context));

    // Pass winit event to gui
    fn update(&mut self, event: &WindowEvent) -> bool;

    // Return the leftover area
    fn viewport(&self, scale_factor: f32) -> Viewport;

    fn requested_format() -> Option<Format> {
        Some(Format::B8G8R8A8_SRGB)
    }
}
