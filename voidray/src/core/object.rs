use std::sync::Arc;

use super::{
    ray::Hittable,
    scene::{MaterialHandle, MeshHandle, ShapeHandle},
};

#[derive(Clone, Copy)]
pub struct Object {
    pub shape: ShapeHandle,
    pub material: MaterialHandle,
}

#[derive(Clone)]
pub enum Shape {
    Analytic(Arc<dyn Hittable>),
    Mesh(MeshHandle),
}
