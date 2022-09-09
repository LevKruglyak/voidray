use super::scene::SceneAcceleration;
use crate::color::*;
use crate::preamble::*;
use crate::rand::*;
use crate::ray::*;
use crate::settings::RenderSettings;

pub fn trace_ray(
    scene: &SceneAcceleration,
    settings: &RenderSettings,
    x: Float,
    y: Float,
    rng: &mut ThreadRng,
) -> Color {
    trace_ray_internal(scene, settings, &scene.camera.cast_ray(x, y, rng), 0, rng)
}

fn trace_ray_internal(
    scene: &SceneAcceleration,
    settings: &RenderSettings,
    ray: &Ray,
    depth: u32,
    rng: &mut ThreadRng,
) -> Color {
    let mut color = BLACK; // emittance component
                               //
    if depth < settings.max_bounces {
        match scene.hit(ray) {
            None => match scene.environment.as_ref() {
                None => return BLACK,
                Some(environment) => return environment.sample(ray),
            },
            Some((hit, object)) => {
                let world_pos = ray.at(hit.t);
                let material = scene.material_ref(object.material);
                let to_viewer = -ray.direction.normalize();

                if let Some((to_incident, pdf)) = material.sample(hit.normal, to_viewer, rng) {
                    if pdf.is_normal() {
                        let bsdf = material.bsdf(hit.normal, to_viewer, to_incident);

                        let ray = Ray::new(world_pos, to_incident);
                        let next_indirect = trace_ray_internal(scene, settings, &ray, depth + 1, rng);
                        let indirect = (bsdf * next_indirect * to_incident.dot(hit.normal).abs()) * (1.0 / pdf);

                        color += indirect.clamp(settings.firefly_clamp);
                    }
                }
            }
        }
    }

    color
}
