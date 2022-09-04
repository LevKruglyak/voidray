use crate::utils::color::Color;

use super::scene::SceneAcceleration;

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
    let trace_ray_internal = |scene: &SceneAcceleration, settings: &RenderSettings, ray, depth: u8| -> Color {
        // Base condition
        if depth >= settings.max_ray_depth {
            return Color::new(0.0, 0.0, 0.0);
        }

        Color::new(1.0, 0.0, 0.0)
    };

    let ray = scene.ray_origin.cast_ray(u, v);
    trace_ray_internal(scene, settings, ray, 0)
}
