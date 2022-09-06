use cgmath::{ElementWise, InnerSpace};

use crate::utils::color::Color;
// use log::*;
use super::{ray::{Ray, Hittable}, scene::SceneAcceleration, Float, Vec3, object::Shape};

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
    pub enable_aces: bool,
    pub gamma: f32,
    pub render_mode: RenderMode,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            samples_per_pixel: 100,
            samples_per_run: 10,
            max_ray_depth: 10,
            render_mode: RenderMode::Full,
            enable_aces: false,
            gamma: 2.2,
        }
    }
}

pub fn trace_ray(
    scene: &SceneAcceleration,
    settings: &RenderSettings,
    u: Float,
    v: Float,
) -> Color {
    let ray = scene.ray_origin.cast_ray(u, v);
    trace_ray_internal(scene, settings, &ray, 0)
}

fn trace_ray_internal(
    scene: &SceneAcceleration,
    settings: &RenderSettings,
    ray: &Ray,
    depth: u8,
) -> Color {
    // Base condition
    if depth >= settings.max_ray_depth {
        return Color::new(0.0, 0.0, 0.0);
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
        let (attenuation, scattered) = match settings.render_mode {
            RenderMode::Full => material.scatter(ray, &hit),
            RenderMode::Normal => (
                0.5 * (hit.normal.normalize() + Vec3::new(1.0, 1.0, 1.0)),
                None,
            ),
        };

        if let Some(scattered) = scattered {
            return attenuation.mul_element_wise(trace_ray_internal(
                scene,
                settings,
                &scattered,
                depth + 1,
            ));
        } else {
            return attenuation;
        };
    }

    scene.environment.sample(ray)
}
