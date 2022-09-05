use cgmath::{ElementWise, InnerSpace};

use crate::utils::color::Color;
// use log::*;
use super::{ray::Ray, scene::SceneAcceleration, Float, Vec3};

#[derive(Debug, Clone)]
pub enum RenderMode {
    Normal,
    Full,
}

#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub samples_per_pixel: u32,
    pub samples_per_run: u32,
    pub max_ray_depth: u8,
    pub render_mode: RenderMode,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            samples_per_pixel: 100,
            samples_per_run: 10,
            max_ray_depth: 10,
            render_mode: RenderMode::Full,
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

    for object in scene.objects {
        if let Some(hit) = object.hit(ray, 0.00001, closest_so_far) {
            closest_so_far = hit.t;
            result = Some((hit, object));
        }
    }

    if let Some((hit, object)) = result {
        let (attenuation, scattered) = match settings.render_mode {
            RenderMode::Full => object.scatter(ray, &hit),
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
