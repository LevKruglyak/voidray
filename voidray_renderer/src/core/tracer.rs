use super::scene::SceneAcceleration;
use crate::color::*;
use crate::preamble::*;
use crate::rand::*;
use crate::ray::*;
use crate::settings::RenderMode;
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

    if depth < settings.max_bounces {
        match scene.hit(ray) {
            None => match scene.environment.as_ref() {
                None => return BLACK,
                Some(environment) => return environment.sample(ray),
            },
            Some((hit, object)) => {
                let material = scene.material_ref(object.material);

                // Different render modes
                let (attenuation, scattered) = match settings.render_mode {
                    RenderMode::Full => material.scatter(ray, &hit, rng),
                    RenderMode::Normal => (Color(0.5 * hit.normal.normalize()) + WHITE, None),
                };

                // Whether or not light was scattered and should be recursively traced
                let delta_color = if let Some(scattered) = scattered {
                    attenuation * trace_ray_internal(scene, settings, &scattered, depth + 1, rng)
                } else {
                    attenuation
                };

                color += delta_color.clamp(settings.firefly_clamp);
            }
        }
    }

    color
}
