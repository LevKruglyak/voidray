use bytemuck::{Pod, Zeroable};
use imgui::{Context, DrawVert, ImString, TextureId, Textures, Ui};
use std::{fmt, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuBufferPool},
    command_buffer::SecondaryAutoCommandBuffer,
    device::{Device, Queue},
    format::Format,
    image::{
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        ImageDimensions, ImageViewAbstract, ImmutableImage,
    },
    impl_vertex,
    pipeline::graphics::{
        color_blend::ColorBlendState,
        input_assembly::{InputAssemblyState, PrimitiveTopology},
        vertex_input::{BuffersDefinition, Vertex},
        viewport::{Viewport, ViewportState},
        GraphicsPipeline,
    },
    render_pass::{RenderPass, Subpass},
    sampler::{Sampler, SamplerCreateInfo},
    swapchain::Surface,
    sync::GpuFuture,
};
use winit::{event::WindowEvent, window::Window};

use super::GuiImplementation;

#[derive(Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
struct ImguiVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: u32,
}

impl_vertex!(ImguiVertex, pos, uv, col);

impl From<DrawVert> for ImguiVertex {
    fn from(vertex: DrawVert) -> ImguiVertex {
        unsafe { std::mem::transmute(vertex) }
    }
}

// #[derive(Debug)]
// pub enum RendererError {
//     BadTexture(TextureId),
//     BadImageDimensions(ImageDimensions),
// }
//
// impl fmt::Display for RendererError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::BadTexture(ref t) => {
//                 write!(f, "The Texture ID could not be found: {:?}", t)
//             }
//             Self::BadImageDimensions(d) => {
//                 write!(f, "Image Dimensions not supported (must be Dim2d): {:?}", d)
//             }
//         }
//     }
// }
//
// impl std::error::Error for RendererError {}

pub struct ImguiImplementation {
    context: Context,
    renderer: Renderer,
}

impl GuiImplementation for ImguiImplementation {
    fn new(surface: Arc<Surface<Window>>, graphics_queue: Arc<Queue>, subpass: Subpass) -> Self {
        let mut context = Context::create();
        let renderer = Renderer::init(
            &mut context,
            graphics_queue.device().clone(),
            graphics_queue,
            subpass,
        );

        Self { context, renderer }
    }

    fn render(&mut self, dimensions: [u32; 2]) -> SecondaryAutoCommandBuffer {
        // self.context.suspend
        unimplemented!();
    }

    fn update(&mut self, event: &WindowEvent) -> bool {
        unimplemented!();
    }

    fn viewport(&self, scale_factor: f32) -> Viewport {
        Viewport {
            origin: [0.0, 0.0],
            dimensions: [1000.0, 1000.0],
            depth_range: 0.0..1.0,
        }
    }

    type Context = Context;

    fn immediate(&mut self, ui: impl FnOnce(&mut Self::Context)) {
        ui(&mut self.context);
    }
}

pub struct Renderer {
    pipeline: Arc<GraphicsPipeline>,
    vrt_buffer_pool: CpuBufferPool<ImguiVertex>,
    idx_buffer_pool: CpuBufferPool<u16>,
}

impl Renderer {
    pub fn init(
        ctx: &mut imgui::Context,
        device: Arc<Device>,
        queue: Arc<Queue>,
        subpass: Subpass,
    ) -> Renderer {
        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();

        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<ImguiVertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .input_assembly_state(
                InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
            )
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .color_blend_state(ColorBlendState::default().blend_alpha())
            .render_pass(subpass)
            .build(device.clone())
            .expect("failed to make pipeline");

        ctx.set_renderer_name(Some("imgui-vulkano".to_string()));

        let vrt_buffer_pool =
            CpuBufferPool::new(device.clone(), BufferUsage::vertex_buffer_transfer_dst());
        let idx_buffer_pool = CpuBufferPool::new(device, BufferUsage::vertex_buffer_transfer_dst());

        Renderer {
            pipeline,
            vrt_buffer_pool,
            idx_buffer_pool,
        }
    }
}

mod vs {
    vulkano_shaders::shader! {
    ty: "vertex",
    src: "
#version 450

layout(push_constant) uniform VertPC {
    mat4 matrix;
};

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in uint col;

layout(location = 0) out vec2 f_uv;
layout(location = 1) out vec4 f_color;

// Built-in:
// vec4 gl_Position

void main() {
    f_uv = uv;
    f_color = unpackUnorm4x8(col);
    gl_Position = matrix * vec4(pos.xy, 0, 1);
}
        " }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(binding = 0) uniform sampler2D tex;

layout(location = 0) in vec2 f_uv;
layout(location = 1) in vec4 f_color;

layout(location = 0) out vec4 Target0;

void main() {
    Target0 = f_color * texture(tex, f_uv.st);
}
            "
    }
}
