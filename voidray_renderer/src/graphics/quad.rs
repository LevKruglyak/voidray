use crate::preamble::*;
use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer},
    device::Queue,
    impl_vertex,
    pipeline::graphics::vertex_input::BuffersDefinition,
};

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
struct TexturedVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl_vertex!(TexturedVertex, position, uv);

pub struct TexturedQuad {
    vertex: Arc<CpuAccessibleBuffer<[TexturedVertex]>>,
    index: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl TexturedQuad {
    pub fn new(queue: Arc<Queue>, min: [f32; 2], max: [f32; 2]) -> Self {
        let vertex = CpuAccessibleBuffer::from_iter(
            queue.device().clone(),
            BufferUsage::vertex_buffer(),
            false,
            [
                TexturedVertex {
                    position: [min[0], min[1]],
                    uv: [0.0, 0.0],
                },
                TexturedVertex {
                    position: [min[0], max[1]],
                    uv: [0.0, 1.0],
                },
                TexturedVertex {
                    position: [max[0], max[1]],
                    uv: [1.0, 1.0],
                },
                TexturedVertex {
                    position: [max[0], min[1]],
                    uv: [1.0, 0.0],
                },
            ]
            .iter()
            .cloned(),
        )
        .expect("failed to create vertex buffer");

        let index = CpuAccessibleBuffer::from_iter(
            queue.device().clone(),
            BufferUsage::index_buffer(),
            false,
            [0, 2, 1, 0, 3, 2].iter().cloned(),
        )
        .expect("failed to create buffer");

        Self { vertex, index }
    }

    pub fn buffers_definition() -> BuffersDefinition {
        BuffersDefinition::new().vertex::<TexturedVertex>()
    }

    pub fn draw(&self, builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>) {
        builder
            .bind_vertex_buffers(0, self.vertex.clone())
            .bind_index_buffer(self.index.clone())
            .draw_indexed(self.index.len() as u32, 1, 0, 0, 0)
            .expect("failed to draw quad");
    }
}
