use crate::core::tracer::RenderSettings;
use crate::render::{RenderTarget, RenderTargetView};
use bytemuck::{Pod, Zeroable};
use std::fmt::Debug;
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
        path: "src/shaders/viewport_vertex.glsl"
    }
}

#[allow(clippy::needless_question_mark)]
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        include: [ "src/shaders" ],
        path: "src/shaders/viewport_fragment.glsl"
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tonemap {
    None,
    SimpleACES,
    SimpleReinhard,
    LumaReinhard,
    LumaWhitePreservingReinhard,
    RomBinDaHouse,
    Filmic,
    Uncharted2,
}

impl Debug for Tonemap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tonemap::None => f.write_str("None"),
            Tonemap::SimpleACES => f.write_str("ACES"),
            Tonemap::SimpleReinhard => f.write_str("Reinhard"),
            Tonemap::LumaReinhard => f.write_str("L Reinhard"),
            Tonemap::LumaWhitePreservingReinhard => f.write_str("LWP Reinhard"),
            Tonemap::RomBinDaHouse => f.write_str("RBDH"),
            Tonemap::Filmic => f.write_str("Filmic"),
            Tonemap::Uncharted2 => f.write_str("Uncharted 2"),
        }
    }
}

#[allow(clippy::wrong_self_convention)]
impl Tonemap {
    fn to_int(&self) -> i32 {
        match self {
            Tonemap::None => 0,
            Tonemap::SimpleACES => 1,
            Tonemap::SimpleReinhard => 2,
            Tonemap::LumaReinhard => 3,
            Tonemap::LumaWhitePreservingReinhard => 4,
            Tonemap::RomBinDaHouse => 5,
            Tonemap::Filmic => 6,
            Tonemap::Uncharted2 => 7,
        }
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
            post_processing_data: PostProcessingData {
                scale: 0.0,
                gamma: 2.2,
                exposure: 1.0,
                tonemap: 0,
                transparent: false as u32,
            },
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

        self.post_processing_data.tonemap = settings.tonemap.to_int();
        self.post_processing_data.gamma = settings.gamma;
        self.post_processing_data.exposure = settings.exposure;
        self.post_processing_data.transparent = settings.transparent as u32;

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
