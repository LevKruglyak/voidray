use std::sync::Arc;

use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo,
        SecondaryAutoCommandBuffer, SubpassContents,
    },
    device::{Device, Queue},
    format::Format,
    image::ImageAccess,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
};
use vulkano_util::{context::VulkanoContext, renderer::SwapchainImageView};

use crate::gui::GuiImplementation;

pub struct FinalRenderPass {
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
}

impl FinalRenderPass {
    pub fn new(context: &VulkanoContext, format: Format) -> Self {
        let render_pass = Self::create_render_pass(context.device(), format);

        Self {
            device: context.device(),
            graphics_queue: context.graphics_queue(),
            render_pass,
        }
    }

    pub fn viewport_subpass(&self) -> Subpass {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }

    pub fn ui_subpass(&self) -> Subpass {
        Subpass::from(self.render_pass.clone(), 1).unwrap()
    }

    fn create_render_pass(device: Arc<Device>, format: Format) -> Arc<RenderPass> {
        vulkano::ordered_passes_renderpass!(
            device,
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: 1,
                }
            },
            passes: [
                { color: [color], depth_stencil: {}, input: [] }, // Draw render result
                { color: [color], depth_stencil: {}, input: [] } // Gui render pass
            ]
        )
        .expect("error creating render pass")
    }

    pub fn render<F, G>(
        &self,
        before_future: F,
        gui: &mut G,
        target: SwapchainImageView,
        viewport_command_buffer: SecondaryAutoCommandBuffer,
    ) -> Box<dyn GpuFuture>
    where
        F: GpuFuture + 'static,
        G: GuiImplementation + 'static,
    {
        // Get dimensions
        let image_dimensions = target.image().dimensions();

        // Create framebuffer (must be in same order as render pass description in `new`
        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![target],
                ..Default::default()
            },
        )
        .unwrap();

        // Create primary command buffer builder
        let mut primary_builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.graphics_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Begin render pass
        primary_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassContents::SecondaryCommandBuffers,
            )
            .unwrap();

        // Render viewport
        primary_builder
            .execute_commands(viewport_command_buffer)
            .unwrap();

        // Render gui
        primary_builder
            .next_subpass(SubpassContents::SecondaryCommandBuffers)
            .unwrap();

        let cb = gui.render(image_dimensions.width_height());
        primary_builder.execute_commands(cb).unwrap();

        // End render pass
        primary_builder.end_render_pass().unwrap();

        // Build command buffer
        let command_buffer = primary_builder.build().unwrap();

        // Execute primary command buffer
        let after_future = before_future
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .unwrap();

        after_future.boxed()
    }
}
