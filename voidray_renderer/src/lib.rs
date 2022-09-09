#![allow(dead_code, unused, unused_variables)]

mod core;
mod render;
mod util;

pub use crate::core::settings;
pub use crate::core::traits;

pub use util::ray;
#[macro_use]
pub use util::vector;
pub use util::aabb;
pub use util::color;
pub use util::math;

/// Std rand imports
pub mod rand {
    pub use rand::prelude::*;
    pub use rand_distr;
}

pub mod preamble {
    pub use crate::vec3;
    pub use crate::vector::*;
    pub use std::sync::Arc;
}
