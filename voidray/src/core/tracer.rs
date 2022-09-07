use cgmath::InnerSpace;

use crate::{
    pipeline::Tonemap,
    utils::color::{alpha_mul, into_alpha, ColorAlpha},
};
// use log::*;
use super::{
    object::Shape,
    ray::{Hittable, Ray},
    scene::SceneAcceleration,
    Float, Vec3,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RenderMode {
    Normal,
    Full,
}

#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub samples_per_pixel: u32,
    pub samples_per_run: u32,
    pub max_ray_depth: u8,
    pub tonemap: Tonemap,
    pub gamma: f32,
    pub exposure: f32,
    pub transparent: bool,
    pub render_mode: RenderMode,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            samples_per_pixel: 100,
            samples_per_run: 1,
            max_ray_depth: 10,
            render_mode: RenderMode::Full,
            tonemap: Tonemap::None,
            transparent: false,
            gamma: 2.2,
            exposure: 1.0,
        }
    }
}

pub fn trace_ray(
    scene: &SceneAcceleration,
    settings: &RenderSettings,
    u: Float,
    v: Float,
) -> ColorAlpha {
    let ray = scene.ray_origin.cast_ray(u, v);
    trace_ray_internal(scene, settings, &ray, 0)
}

fn trace_ray_internal(
    scene: &SceneAcceleration,
    settings: &RenderSettings,
    ray: &Ray,
    depth: u8,
) -> ColorAlpha {
    // Base condition
    if depth >= settings.max_ray_depth {
        return ColorAlpha::new(0.0, 0.0, 0.0, 1.0);
    }

    let mut result = None;
    let mut closest_so_far = Float::INFINITY;

    for object in &scene.objects {
        let shape = scene.shape_ref(object.shape);
        if let Some(hit) = match shape {
            Shape::Analytic(hittable) => hittable.hit(ray, 0.000001, closest_so_far),
            Shape::Mesh(handle) => scene.mesh_ref(*handle).hit(ray, 0.000001, closest_so_far),
        } {
            closest_so_far = hit.t;
            result = Some((hit, object));
        }
    }

    if let Some((hit, object)) = result {
        let material = scene.material_ref(object.material);
        let (attenuation, refracted, scattered) = match settings.render_mode {
            RenderMode::Full => material.scatter(ray, &hit),
            RenderMode::Normal => (
                0.5 * (hit.normal.normalize() + Vec3::new(1.0, 1.0, 1.0)),
                false,
                None,
            ),
        };

        let alpha = if refracted { 0.0 } else { 1.0 };

        if let Some(scattered) = scattered {
            return alpha_mul(
                into_alpha(attenuation, alpha),
                trace_ray_internal(scene, settings, &scattered, depth + 1),
            );
        } else {
            return into_alpha(attenuation, alpha);
        };
    }

    into_alpha(scene.environment.sample(ray), 0.0)
}
