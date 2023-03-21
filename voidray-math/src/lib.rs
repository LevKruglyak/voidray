mod common;
mod vector;

pub mod types {
    #[cfg(feature = "high_precision")]
    pub type Float = f64;

    #[cfg(not(feature = "high_precision"))]
    pub type Float = f32;

    pub use crate::vector::*;
}

pub mod operations {
    pub use crate::common::Lerp;
}
