use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};

pub trait Mesh {
    fn bind(&self, builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>);
}

pub struct Vertex2f {

}
