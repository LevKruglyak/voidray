use crate::aabb::Bounded;
use crate::color::*;
use crate::preamble::*;
use crate::rand::*;
use crate::ray::*;

/// BSDF material trait
pub trait Material: Send + Sync {
    /// Bidirectional scattering distribution function
    fn bsdf(&self, normal: Vec3, to_viewer: Vec3, to_incident: Vec3) -> Color;

    /// Importance sample function for the light hemisphere, returns (to_incident, pdf)
    fn sample(&self, normal: Vec3, to_viewer: Vec3, rng: &mut ThreadRng) -> Option<(Vec3, Float)>;
}

/// A surface defined mathematically, not through a mesh
pub trait AnalyticSurface: Bounded + Send + Sync {
    /// Function describing the intersection function for the surface
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;
}

#[derive(Clone)]
pub enum Surface {
    Analytic(Arc<dyn AnalyticSurface>),
}

/// A background environment for the scene, describing the behaviour of escaped rays
pub trait Environment: Send + Sync {
    /// Sample the enviroment along the ray direction
    fn sample(&self, ray: &Ray) -> Color;
}
