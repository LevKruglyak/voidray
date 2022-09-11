use std::sync::Arc;
use super::target::CpuRenderTarget;
use crate::graphics::quad::TexturedQuad;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferInheritanceInfo, CommandBufferUsage,
    PrimaryAutoCommandBuffer,
};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::ImageViewAbstract;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::input_assembly::{InputAssemblyState, PrimitiveTopology};
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::{graphics, Pipeline, PipelineBindPoint};
use vulkano::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo};
use vulkano::{device::Queue, pipeline::GraphicsPipeline, render_pass::Subpass};

#[allow(clippy::needless_question_mark)]
mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/view_vert.glsl"
    }
}

#[allow(clippy::needless_question_mark)]
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/view_frag.glsl"
    }
}

pub struct Viewport {
    queue: Arc<Queue>,
    subpass: Subpass,
    pipeline: Arc<GraphicsPipeline>,
    target: Arc<CpuRenderTarget>,
    view_quad: TexturedQuad,
}

impl Viewport {
    pub fn new(queue: Arc<Queue>, subpass: Subpass, target: Arc<CpuRenderTarget>) -> Self {
        let pipeline = {
            let vs = vs::load(queue.device().clone()).expect("failed to create shader module");
            let fs = fs::load(queue.device().clone()).expect("failed to create shader module");

            GraphicsPipeline::start()
                .vertex_input_state(TexturedQuad::buffers_definition())
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .input_assembly_state(
                    InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
                )
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .render_pass(subpass.clone())
                .color_blend_state(ColorBlendState::new(1).blend_alpha())
                .build(queue.device().clone())
                .expect("failed to make pipeline")
        };

        let view_quad = TexturedQuad::new(queue.clone(), [-1.0, -1.0], [1.0, 1.0]);

        Self {
            queue,
            subpass,
            pipeline,
            target,
            view_quad,
        }
    }

    pub fn draw(
        &mut self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        viewport: graphics::viewport::Viewport,
    ) {
        let mut secondary_builder = AutoCommandBufferBuilder::secondary(
            self.queue.device().clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(self.subpass.clone().into()),
                ..Default::default()
            },
        )
        .unwrap();

        let view = self.target.view();

        let descriptor_set = self.create_view_descriptor_set(view);
        secondary_builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .set_viewport(0, vec![viewport]);
        self.view_quad.draw(&mut secondary_builder);
        command_buffer
            .execute_commands(secondary_builder.build().unwrap())
            .unwrap();
    }

    fn create_view_descriptor_set(
        &self,
        viewport_view: Arc<dyn ImageViewAbstract>,
    ) -> Arc<PersistentDescriptorSet> {
        let sampler = Sampler::new(
            self.queue.device().clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )
        .unwrap();

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        PersistentDescriptorSet::new(
            layout.clone(),
            [WriteDescriptorSet::image_view_sampler(
                0,
                viewport_view.clone(),
                sampler,
            )],
        )
        .unwrap()
    }
}
