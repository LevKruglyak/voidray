use super::target::CpuRenderTarget;
use crate::{
    color::BLACK,
    core::{scene::SceneAcceleration, tracer::trace_ray},
    preamble::*,
    settings::RenderSettings,
};
use rand::{thread_rng, Rng};
use rayon::prelude::*;

pub fn iterative_render(
    target: Arc<CpuRenderTarget>,
    scene: &SceneAcceleration,
    settings: &RenderSettings,
    samples: u32,
    total_samples: u32,
) {
    let dimensions = target.dimensions();

    target
        .buffer() 
        .as_slice_mut()
        .par_chunks_exact_mut(4)
        .enumerate()
        .for_each(|(index, pixel)| {
            let x = index as u32 % dimensions[0];
            let y = index as u32 / dimensions[1];

            // Rng
            let mut rng = thread_rng();

            let d = std::cmp::max(dimensions[0], dimensions[1]) as Float;
            let x = ((2 * x + 1) as Float - dimensions[0] as Float) / d;
            let y = ((2 * (dimensions[1] - y) - 1) as Float - dimensions[1] as Float) / d;

            let mut color = BLACK;

            for _ in 0..samples {
                // Camera UV, normalized to [-1, 1]
                let dx = rng.gen_range((-1.0 / d)..(1.0 / d));
                let dy = rng.gen_range((-1.0 / d)..(1.0 / d));

                color += trace_ray(scene, settings, x + dx, y + dy, &mut rng);
            }

            color *= 1.0 / total_samples as Float;

            // Copy into the target
            pixel[0] += color.r() as f32;
            pixel[1] += color.g() as f32;
            pixel[2] += color.b() as f32;
            pixel[3] += color.a() as f32;
        });

    target.try_push();
}
