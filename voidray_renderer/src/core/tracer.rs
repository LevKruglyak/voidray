use super::scene::SceneAcceleration;
use crate::color::*;
use crate::preamble::*;
use crate::rand::*;
use crate::ray::*;

#[derive(Debug, Clone, PartialEq)]
pub enum RenderMode {
    Normal,
    Full,
}

#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub render_mode: RenderMode,
    pub firefly_clamp: Float,
    pub max_bounces: u32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            render_mode: RenderMode::Full,
            firefly_clamp: 3.0,
            max_bounces: 10,
        }
    }
}

fn trace_ray_internal(
    scene: &SceneAcceleration,
    settings: &RenderSettings,
    ray: &Ray,
    depth: u32,
    rng: &mut ThreadRng,
) -> Color {
    match scene.hit(ray) {
        None => match scene.environment.as_ref() {
            None => BLACK,
            Some(environment) => environment.sample(ray),
        },
        Some((hit, object)) => {
            let world_pos = ray.at(hit.t);
            let material = scene.material_ref(object.material);
            let to_viewer = -ray.direction.normalize();

            let mut color = BLACK; // emittance component
                                   // light sampling

            if depth < settings.max_bounces {
                if let Some((to_incident, pdf)) = material.sample(hit.normal, to_viewer, rng) {
                    let bsdf = material.bsdf(hit.normal, to_viewer, to_incident);

                    let ray = Ray::new(world_pos, to_incident);
                    let indirect = (bsdf
                        * trace_ray_internal(scene, settings, &ray, depth + 1, rng))
                        * to_incident.dot(hit.normal).abs()
                        * (1.0 / pdf);

                    color += indirect.clamp(settings.firefly_clamp);
                }
            }

            color
        }
    }
}

pub fn trace_ray(
    scene: &SceneAcceleration,
    settings: &RenderSettings,
    x: Float,
    y: Float,
    rng: &mut ThreadRng,
) -> Color {
    trace_ray_internal(scene, settings, &scene.camera.cast_ray(x, y, rng), 0, rng)
}
