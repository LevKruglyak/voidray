use crate::aabb::Bounded;
use crate::color::*;
use crate::preamble::*;
use crate::rand::*;
use crate::ray::*;
use crate::scene::MeshHandle;
use crate::scene::SceneAcceleration;

/// BSDF material trait
pub trait BSDFMaterial: Send + Sync {
    /// Bidirectional scattering distribution function
    fn bsdf(&self, normal: &Vec3, to_viewer: &Vec3, to_incident: &Vec3) -> Color;

    /// Importance sample function for the light hemisphere, returns (to_incident, pdf)
    fn sample(&self, normal: &Vec3, to_viewer: &Vec3, rng: &mut ThreadRng)
        -> Option<(Vec3, Float)>;
}

impl<M> Material for M
where
    M: BSDFMaterial,
{
    fn scatter(
        &self,
        scene: &SceneAcceleration,
        ray: &Ray,
        hit: &HitRecord,
        rng: &mut ThreadRng,
    ) -> (Color, Option<Ray>) {
        let wo = ray.direction.normalize();
        if let Some((wi, pdf)) = self.sample(&hit.normal, &wo, rng) {
            let f = self.bsdf(&hit.normal, &wo, &wi);
            let ray = Ray::new(ray.at(hit.t), wi);
            let indirect = f * wi.dot(hit.normal).abs() * (1.0 / pdf);

            (indirect, Some(ray))
        } else {
            (hex_color(0x000000), None)
        }
    }
}

pub trait Material: Send + Sync {
    fn scatter(
        &self,
        scene: &SceneAcceleration,
        ray: &Ray,
        hit: &HitRecord,
        rng: &mut ThreadRng,
    ) -> (Color, Option<Ray>);
}

/// A surface defined mathematically, not through a mesh
pub trait AnalyticSurface: Bounded + Send + Sync {
    /// Function describing the intersection function for the surface
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;
}

#[derive(Clone)]
pub enum Surface {
    Analytic(Arc<dyn AnalyticSurface>),
    Mesh(MeshHandle),
}

/// A background environment for the scene, describing the behaviour of escaped rays
pub trait Environment: Send + Sync {
    /// Sample the enviroment along the ray direction
    fn sample(&self, ray: &Ray) -> Color;
}
