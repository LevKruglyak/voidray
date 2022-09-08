use crate::{
    core::{environment::Environment, ray::Ray, Float, PI},
    utils::{color::Color, math::to_spherical_coords},
};
use cgmath::InnerSpace;
use log::*;
use std::sync::Arc;

pub struct Environments {}

impl Environments {
    pub fn uniform(background: Color) -> Arc<dyn Environment> {
        Arc::new(UniformEnvironment::new(background))
    }

    pub fn hdri(path: &str) -> Arc<dyn Environment> {
        Arc::new(HDRIEnvironment::new(path))
    }
}

struct UniformEnvironment {
    pub color: Color,
}

impl UniformEnvironment {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Environment for UniformEnvironment {
    fn sample(&self, _: &Ray) -> Color {
        self.color
    }
}

struct HDRIEnvironment {
    image: Vec<Color>,
    width: usize,
    height: usize,
}

impl HDRIEnvironment {
    fn new(path: &str) -> Self {
        let image = exr::prelude::read_first_rgba_layer_from_file(
            path,
            |resolution, _| Self {
                image: vec![Color::new(0.0, 0.0, 0.0); resolution.width() * resolution.height()],
                width: resolution.width(),
                height: resolution.height(),
            },
            |skymap: &mut Self, position, (r, g, b, _): (f32, f32, f32, f32)| {
                skymap.image[position.x() + position.y() * skymap.width] =
                    Color::new(r as Float, g as Float, b as Float);
            },
        )
        .expect("could not read image!");
        info!(
            "loaded '{}', dimensions: {},{}",
            path,
            image.layer_data.channel_data.pixels.width,
            image.layer_data.channel_data.pixels.height
        );
        image.layer_data.channel_data.pixels
    }
}

impl Environment for HDRIEnvironment {
    fn sample(&self, ray: &Ray) -> Color {
        let spherical_coords = to_spherical_coords(ray.direction.normalize());
        let u = spherical_coords.x / PI;
        let v = spherical_coords.y / (2.0 * PI);
        let x = (v * self.width as Float) as usize % self.width;
        let y = self.height - 1 - (u * self.height as Float) as usize % self.height;
        self.image[x + y * self.width]
    }
}
