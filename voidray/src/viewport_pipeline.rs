use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use vulkano::{
    buffer::BufferUsage,
    buffer::{CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::AutoCommandBufferBuilder,
    command_buffer::SecondaryAutoCommandBuffer,
    device::Queue,
    pipeline::{
        graphics::{
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::Subpass,
};
use vulkano::impl_vertex;

#[allow(clippy::needless_question_mark)]
mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 f_uv;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    f_uv = uv;
}"
    }
}

#[allow(clippy::needless_question_mark)]
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(location = 0) in vec2 f_uv;
layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(sin(100.0 * f_uv), f_uv.x, 1.0);
}"
    }
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
struct QuadVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl_vertex!(QuadVertex, position, uv);

pub struct ViewportPipeline {
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[QuadVertex]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl ViewportPipeline {
    pub fn new(graphics_queue: Arc<Queue>, subpass: Subpass) -> Self {
        let pipeline = {
            let vs =
                vs::load(graphics_queue.device().clone()).expect("failed to create shader module");
            let fs =
                fs::load(graphics_queue.device().clone()).expect("failed to create shader module");

            GraphicsPipeline::start()
                .vertex_input_state(BuffersDefinition::new().vertex::<QuadVertex>())
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .input_assembly_state(
                    InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
                )
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .render_pass(subpass)
                .build(graphics_queue.device().clone())
                .expect("failed to make pipeline")
        };

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            graphics_queue.device().clone(),
            BufferUsage::all(),
            false,
            [
                QuadVertex {
                    position: [-1.0, -1.0],
                    uv: [0.0, 0.0],
                },
                QuadVertex {
                    position: [-1.0, 1.0],
                    uv: [0.0, 1.0],
                },
                QuadVertex {
                    position: [1.0, 1.0],
                    uv: [1.0, 1.0],
                },
                QuadVertex {
                    position: [1.0, -1.0],
                    uv: [1.0, 0.0],
                },
            ]
            .iter()
            .cloned(),
        )
        .expect("failed to create buffer");

        let index_buffer = CpuAccessibleBuffer::from_iter(
            graphics_queue.device().clone(),
            BufferUsage::all(),
            false,
            [0, 2, 1, 0, 3, 2].iter().cloned(),
        )
        .expect("failed to create buffer");

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn draw(&mut self, builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, viewport: Viewport) {
        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .bind_index_buffer(self.index_buffer.clone())
            .set_viewport(
                0,
                vec![viewport],
            )
            .draw_indexed(self.index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap();
    }
}
