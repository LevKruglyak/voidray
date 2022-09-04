use cgmath::ElementWise;

use crate::utils::color::Color;
// use log::*;
use super::{scene::SceneAcceleration, ray::Ray};

#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub samples_per_pixel: u32,
    pub samples_per_run: u32,
    pub max_ray_depth: u8,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            samples_per_pixel: 100,
            samples_per_run: 10,
            max_ray_depth: 10,
        }
    }
}

pub fn trace_ray(scene: &SceneAcceleration, settings: &RenderSettings, u: f32, v: f32) -> Color {
    let ray = scene.ray_origin.cast_ray(u, v);
    trace_ray_internal(scene, settings, &ray, 0)
}


fn trace_ray_internal(scene: &SceneAcceleration, settings: &RenderSettings, ray: &Ray, depth: u8) -> Color {
    // Base condition
    if depth >= settings.max_ray_depth {
        return Color::new(0.0, 0.0, 0.0);
    }

    let object = &scene.objects[0];
    if let Some(hit) = object.hit(ray, 0.00001, f32::INFINITY) {
        let (attenuation, scattered) = object.scatter(ray, &hit);           

        if let Some(scattered) = scattered {
            return attenuation.mul_element_wise(trace_ray_internal(scene, settings, &scattered, depth + 1));
        } else {
            return attenuation;
        };
    }

    Color::new(0.1, 0.1, 0.1)
}
