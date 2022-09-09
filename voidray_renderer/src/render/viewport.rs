use std::sync::{Arc, RwLock};

use vulkano::pipeline::graphics::input_assembly::{InputAssemblyState, PrimitiveTopology};
use crate::graphics::quad::TexturedQuad;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::{device::Queue, render_pass::Subpass, pipeline::GraphicsPipeline};
use super::target::CpuRenderTarget;

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
    view: Arc<GraphicsPipeline>,
    target: Arc<RwLock<CpuRenderTarget>>,
}

impl Viewport {
    pub fn new(
        queue: Arc<Queue>, 
        subpass: Subpass,
        target: Arc<RwLock<CpuRenderTarget>>,
    ) -> Self {
        let view = {
            let vs = vs::load(queue.device().clone())
                .expect("failed to create shader module");
            let fs = fs::load(queue.device().clone())
                .expect("failed to create shader module");

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

        Self {
            queue,
            subpass,
            view,
            target,
        }
    }
}
