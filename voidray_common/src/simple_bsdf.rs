use std::sync::Arc;

use voidray_renderer::math::{refract, reflect};
use voidray_renderer::{color::*, vec3};
use voidray_renderer::rand::{ThreadRng, Rng};
use voidray_renderer::rand::rand_distr::{self, UnitSphere};
use voidray_renderer::traits::Material;
use voidray_renderer::vector::*;

pub struct DisneyBSDF {
}

impl DisneyBSDF {
    /// Perfect diffuse (Lambertian) material with a given color
    pub fn diffuse(albedo: Color, roughness: Float) -> Arc<DisneyDiffuse> {
        Arc::new(DisneyDiffuse {
            albedo,
            roughness,
        })
    }
}

pub struct DisneyDiffuse {
    albedo: Color,
    roughness: Float,
}

impl DisneyDiffuse {
}

impl Material for DisneyDiffuse {
    fn bsdf(&self, normal: Vec3, to_viewer: Vec3, to_incident: Vec3) -> Color {
        let h = (to_viewer + to_incident).normalize(); 
        let f_d90 = 0.5 + 2.0 * self.roughness * (to_incident.dot(h)).powi(2);
        let f_d_in = (1.0 + (f_d90 - 1.0)) * (1.0 - (normal.dot(to_viewer)).powi(5));
        let f_d_out = (1.0 + (f_d90 - 1.0)) * (1.0 - (normal.dot(to_incident)).powi(5));
        self.albedo * (1.0 / PI) * f_d_in * f_d_out
    }

    fn sample(&self, normal: Vec3, to_viewer: Vec3, rng: &mut ThreadRng) -> Option<(Vec3, Float)> {
        Some((Vec3::from(rng.sample(UnitSphere)), 1.0))    
    }
}
