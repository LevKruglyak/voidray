#![allow(dead_code, unused_variables)]

mod core;
pub mod render;
pub mod graphics;
mod util;

pub use crate::core::settings;
pub use crate::core::traits;
pub use crate::core::scene;
pub use crate::core::camera;

pub use util::aabb;
pub use util::color;
pub use util::math;
pub use util::ray;
pub use util::vector;

/// Std rand imports
pub mod rand {
    pub use rand::prelude::*;
    pub use rand_distr;
}

pub mod preamble {
    pub use crate::vec3;
    pub use crate::vector::*;
    pub use std::sync::Arc;
    pub use std::sync::RwLock;
}

pub use vulkano;
pub use vulkano_util;
pub use vulkano_shaders;
pub use vulkano_shaders::shader;

pub use rayon;
