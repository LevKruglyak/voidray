use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use vulkano::{
    buffer::BufferUsage,
    buffer::{CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::AutoCommandBufferBuilder,
    command_buffer::{
        CommandBufferInheritanceInfo, CommandBufferUsage, SecondaryAutoCommandBuffer,
    },
    descriptor_set::PersistentDescriptorSet,
    device::Queue,
    pipeline::{
        graphics::{
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::Subpass,
    sampler::{Sampler, SamplerCreateInfo},
};

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450
layout(location = 0) in vec3 position;

// layout(set = 0, binding = 0) uniform Data {
//     mat4 proj;
// } uniforms;
//
// void main() {
//     gl_Position = uniforms.proj * vec4(position, 1.0);
// }
void main() {
    gl_Position = vec4(position, 1.0);
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(0.0, 0.0, 0.0, 1.0);
}"
    }
}

// #[repr(C)]
// #[derive(Clone, Copy, Zeroable, Pod)]
// pub struct UniformData {
//      projection: Mat4f,
// }

pub struct TestPipeline {
    graphics_queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
    subpass: Subpass,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex3f]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl TestPipeline {
    pub fn new(graphics_queue: Arc<Queue>, subpass: Subpass) -> Self {
        let pipeline = {
            let vs =
                vs::load(graphics_queue.device().clone()).expect("failed to create shader module");
            let fs =
                fs::load(graphics_queue.device().clone()).expect("failed to create shader module");

            GraphicsPipeline::start()
                .vertex_input_state(BuffersDefinition::new().vertex::<Vertex3f>())
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .input_assembly_state(
                    InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
                )
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .render_pass(subpass.clone())
                .build(graphics_queue.device().clone())
                .expect("failed to make pipeline")
        };

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            graphics_queue.device().clone(),
            BufferUsage::all(),
            false,
            [
                Vertex3f {
                    position: [1.0, 0.0, 0.0],
                },
                Vertex3f {
                    position: [0.0, 1.0, 0.0],
                },
                Vertex3f {
                    position: [0.0, 0.0, 1.0],
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
            [0, 1, 2].iter().cloned(),
        )
        .expect("failed to create buffer");

        Self {
            graphics_queue,
            pipeline,
            subpass,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn draw(&mut self) -> SecondaryAutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::secondary(
            self.graphics_queue.device().clone(),
            self.graphics_queue.family(),
            CommandBufferUsage::MultipleSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(self.subpass.clone().into()),
                ..Default::default()
            },
        )
        .expect("failed to make secondary command buffer");

        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            // .bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline_layout, first_set, descriptor_sets)
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .bind_index_buffer(self.index_buffer.clone())
            .set_viewport(
                0,
                vec![Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [1000.0, 1000.0],
                    depth_range: 0.0..1.0,
                }],
            )
            .draw_indexed(self.index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap();

        builder.build().expect("failed to build command buffer")
    }
}
