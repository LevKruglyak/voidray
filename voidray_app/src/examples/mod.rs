pub mod example_import {
    pub use voidray_common::simple::Materials;
    pub use voidray_common::Environments;
    pub use voidray_common::Surfaces;
    pub use voidray_renderer::camera::Camera;
    pub use voidray_renderer::mesh::*;
    pub use voidray_renderer::color::*;
    pub use voidray_renderer::preamble::*;
    pub use voidray_renderer::scene::Scene;
    pub use voidray_renderer::settings::Settings;
    pub use voidray_renderer::settings::Tonemap;
    pub use voidray_renderer::texture::SampleType;
}

pub mod spheres;
pub mod cornell;
pub mod mushroom;
