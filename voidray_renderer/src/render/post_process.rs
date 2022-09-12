use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::Queue,
    image::{view::ImageView, AttachmentImage},
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};

#[allow(clippy::needless_question_mark)]
mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/post_process.glsl"
    }
}

pub use cs::ty::PostProcessingData;

pub struct PostProcessingPass {
    queue: Arc<Queue>,
    pipeline: Arc<ComputePipeline>,
}

impl PostProcessingPass {
    pub fn new(queue: Arc<Queue>) -> Self {
        let cs = cs::load(queue.device().clone()).expect("failed to create shader module");

        let pipeline = ComputePipeline::new(
            queue.device().clone(),
            cs.entry_point("main").unwrap(),
            &(),
            None,
            |_| {},
        )
        .expect("failed to create compute pipeline");

        Self { queue, pipeline }
    }

    pub fn render(
        &self,
        src: Arc<ImageView<AttachmentImage>>,
        dst: Arc<ImageView<AttachmentImage>>,
        data: PostProcessingData,
    ) {
        let mut builder = AutoCommandBufferBuilder::primary(
            self.queue.device().clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            layout.clone(),
            [
                WriteDescriptorSet::image_view(0, src),
                WriteDescriptorSet::image_view(1, dst),
            ],
        )
        .unwrap();
        builder
            .bind_pipeline_compute(self.pipeline.clone())
            .push_constants(self.pipeline.layout().clone(), 0, data)
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pipeline.layout().clone(),
                0,
                set,
            )
            .dispatch([1024 / 8, 1024 / 8, 1])
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.queue.device().clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }
}
