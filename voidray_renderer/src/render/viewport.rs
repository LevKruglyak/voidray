use super::post_process::{PostProcessingData, PostProcessingPass};
use super::target::CpuRenderTarget;
use crate::graphics::quad::TexturedQuad;
use std::sync::Arc;
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
    graphics_queue: Arc<Queue>,
    compute_queue: Arc<Queue>,
    subpass: Subpass,
    pipeline: Arc<GraphicsPipeline>,
    target: Arc<CpuRenderTarget>,
    view_quad: TexturedQuad,
    post_process: PostProcessingPass,
}

impl Viewport {
    pub fn new(
        graphics_queue: Arc<Queue>,
        compute_queue: Arc<Queue>,
        subpass: Subpass,
        target: Arc<CpuRenderTarget>,
    ) -> Self {
        let pipeline = {
            let vs =
                vs::load(graphics_queue.device().clone()).expect("failed to create shader module");
            let fs =
                fs::load(compute_queue.device().clone()).expect("failed to create shader module");

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
                .build(graphics_queue.device().clone())
                .expect("failed to make pipeline")
        };

        let view_quad = TexturedQuad::new(graphics_queue.clone(), [-1.0, -1.0], [1.0, 1.0]);
        let post_process = PostProcessingPass::new(compute_queue.clone());

        Self {
            graphics_queue,
            compute_queue,
            subpass,
            pipeline,
            target,
            view_quad,
            post_process,
        }
    }

    pub fn draw(
        &mut self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        viewport: graphics::viewport::Viewport,
        data: PostProcessingData,
    ) {
        let mut secondary_builder = AutoCommandBufferBuilder::secondary(
            self.graphics_queue.device().clone(),
            self.graphics_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(self.subpass.clone().into()),
                ..Default::default()
            },
        )
        .unwrap();

        let view = {
            let pre_process = self.target.pull_view();
            let post_process = self.target.get_view(1);

            // Run post processing pass
            self.post_process
                .render(pre_process, post_process.clone(), data);
            post_process
        };

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
            self.graphics_queue.device().clone(),
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
