#![allow(dead_code, unused_variables)]

mod core;
pub mod graphics;
pub mod render;
mod util;

pub use crate::core::*;
pub use util::*;

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
pub use vulkano_shaders;
pub use vulkano_shaders::shader;
pub use vulkano_util;

pub use rayon;
