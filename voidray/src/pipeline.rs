use crate::core::tracer::RenderSettings;
use crate::render::{RenderTarget, RenderTargetView};
use bytemuck::{Pod, Zeroable};
use std::sync::{Arc, RwLock};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::ImageViewAbstract;
use vulkano::impl_vertex;
use vulkano::pipeline::{Pipeline, PipelineBindPoint};
use vulkano::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo};
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

use self::fs::ty::PostProcessingData;

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

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(push_constant) uniform PostProcessingData {
  float scale;
  float gamma;
  bool aces;
} ppd;

// float3 ACESFitted(float3 color)
// {
//     color = mul(ACESInputMat, color);
//
//     // Apply RRT and ODT
//     color = RRTAndODTFit(color);
//
//     color = mul(ACESOutputMat, color);
//
//     // Clamp to [0, 1]
//     color = saturate(color);
//
//     return color;
// }

const mat3 ACESInputMat = 
{
    {0.59719, 0.35458, 0.04823},
    {0.07600, 0.90834, 0.01566},
    {0.02840, 0.13383, 0.83777}
};

const mat3 ACESOutputMat = 
{
    { 1.60475, -0.53108, -0.07367},
    {-0.10208,  1.10813, -0.00605},
    {-0.00327, -0.07276,  1.07602}
};

vec3 RRTAndODTFit(vec3 v) {
    vec3 a = v * (v + 0.0245786) - 0.000090537;
    vec3 b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return a / b;
}

vec3 ACESFitted(vec3 color) {
    color = ACESInputMat * color;

    // Apply RRT and ODT
    color = RRTAndODTFit(color);

    color = ACESOutputMat * color;

    return color;
}

vec3 ACESFilm(vec3 x) {
    float a = 2.51f;
    float b = 0.03f;
    float c = 2.43f;
    float d = 0.59f;
    float e = 0.14f;
    return ((x*(a*x+b))/(x*(c*x+d)+e));
}

void main() {
    vec3 color = texture(tex, f_uv).xyz * ppd.scale;

    if (ppd.aces) {
        color = ACESFilm(color);
    }

    color = pow(color, vec3(1.0 / ppd.gamma));

    f_color = vec4(color, 1.0);
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
    target: Arc<RwLock<RenderTarget>>,
    target_view: Arc<RenderTargetView>,
    graphics_queue: Arc<Queue>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[QuadVertex]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    post_processing_data: PostProcessingData,
}

impl ViewportPipeline {
    pub fn new(
        graphics_queue: Arc<Queue>,
        subpass: Subpass,
        target: Arc<RwLock<RenderTarget>>,
        target_view: Arc<RenderTargetView>,
    ) -> Self {
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
            graphics_queue,
            vertex_buffer,
            index_buffer,
            target,
            target_view,
            post_processing_data: PostProcessingData { scale: 0.0, aces: false as u32, gamma: 2.2 },
        }
    }

    pub fn draw(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        settings: &RenderSettings,
        viewport: Viewport,
    ) {
        if let Ok(mut target) = self.target.try_write() {
            if target.needs_sync() {
                target.copy_to_view(self.target_view.clone());
                self.post_processing_data.scale = target.scale();
            }
        }

        self.post_processing_data.aces = settings.enable_aces as u32;
        self.post_processing_data.gamma = settings.gamma;

        let descriptor_set = self.create_descriptor_set(self.target_view.view());
        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .bind_index_buffer(self.index_buffer.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .push_constants(self.pipeline.layout().clone(), 0, self.post_processing_data)
            .set_viewport(0, vec![viewport])
            .draw_indexed(self.index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap();
    }

    fn create_descriptor_set(
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
