use std::f32::consts::PI;

use cgmath::{InnerSpace, ElementWise};

use crate::{utils::{color::Color, math::{lerp, near_zero, sample_unit_sphere_surface}}, core::{material::Material, Vec3, ray::{Ray, HitRecord}}};

#[derive(Copy, Clone)]
pub struct PrincipledBSDF {
    pub color: Color,

    /// Index of refraction
    pub index: f32,

    /// Roughness parameter for Beckmann microfacet distribution
    pub roughness: f32,

    /// Metallic versus dielectric
    pub metallic: f32,

    /// Self-emittance of light
    pub emittance: f32,

    /// Transmittance (e.g., glass)
    pub transparent: bool,
}

impl PrincipledBSDF {
    /// Perfect diffuse (Lambertian) material with a given color
    pub fn diffuse(color: Color) -> PrincipledBSDF {
        PrincipledBSDF {
            color,
            index: 1.5,
            roughness: 1.0,
            metallic: 0.0,
            emittance: 0.0,
            transparent: false,
        }
    }

    /// Specular material with a given color and roughness
    pub fn specular(color: Color, roughness: f32) -> PrincipledBSDF {
        PrincipledBSDF {
            color,
            index: 1.5,
            roughness,
            metallic: 0.0,
            emittance: 0.0,
            transparent: false,
        }
    }

    /// Clear material with a specified index of refraction and roughness (such as glass)
    pub fn clear(index: f32, roughness: f32) -> PrincipledBSDF {
        PrincipledBSDF {
            color: Color::new(1.0, 1.0, 1.0),
            index,
            roughness,
            metallic: 0.0,
            emittance: 0.0,
            transparent: true,
        }
    }

    /// Colored transparent material
    pub fn transparent(color: Color, index: f32, roughness: f32) -> PrincipledBSDF {
        PrincipledBSDF {
            color,
            index,
            roughness,
            metallic: 0.0,
            emittance: 0.0,
            transparent: true,
        }
    }

    /// Metallic material (has extra tinted specular reflections)
    pub fn metallic(color: Color, roughness: f32) -> PrincipledBSDF {
        PrincipledBSDF {
            color,
            index: 1.5,
            roughness,
            metallic: 1.0,
            emittance: 0.0,
            transparent: false,
        }
    }

    /// Perfect emissive material, useful for modeling area lights
    pub fn light(color: Color, emittance: f32) -> PrincipledBSDF {
        PrincipledBSDF {
            color,
            index: 1.0,
            roughness: 1.0,
            metallic: 0.0,
            emittance,
            transparent: false,
        }
    }
}

impl Material for PrincipledBSDF {
    fn bsdf(&self, normal: Vec3, to_viewer: Vec3, to_ray: Vec3) -> Color {
        let normal_dot_ray = normal.dot(to_ray);
        let normal_dot_viewer = normal.dot(to_viewer);
        let ray_outside = normal_dot_ray.is_sign_positive();
        let viewer_outside = normal_dot_viewer.is_sign_positive();

        if !self.transparent && (!ray_outside || !viewer_outside) {
            // Opaque materials do not transmit light
            return Color::new(0.0, 0.0, 0.0);
        }

        if ray_outside == viewer_outside {
            let h = (to_viewer + to_ray).normalize(); // halfway vector
            let viewer_dot_h= to_viewer.dot(h);
            let normal_dot_h = normal.dot(h);
            let nh2 = normal_dot_h.powi(2);

            // d: microfacet distribution function
            // D = exp(((n • h)^2 - 1) / (m^2 (n • h)^2)) / (π m^2 (n • h)^4)
            let m2 = self.roughness * self.roughness;
            let d = ((nh2 - 1.0) / (m2 * nh2)).exp() / (m2 * PI * nh2 * nh2);

            // f: fresnel, schlick's approximation
            // F = F0 + (1 - F0)(1 - wi • h)^5
            let f = if !ray_outside && (1.0 - viewer_dot_h * viewer_dot_h).sqrt() * self.index > 1.0 {
                // Total internal reflection
                Vec3::new(1.0, 1.0, 1.0)
            } else {
                let f0 = ((self.index - 1.0) / (self.index + 1.0)).powi(2);
                let f0 = lerp(Vec3::new(f0, f0, f0), self.color, self.metallic);
                f0 + (Vec3::new(1.0, 1.0, 1.0) - f0) * (1.0 - viewer_dot_h).powi(5)
            };

            // g: geometry function, microfacet shadowing
            // G = min(1, 2(n • h)(n • wo)/(wo • h), 2(n • h)(n • wi)/(wo • h))
            let g = f32::min(normal_dot_ray * normal_dot_h, normal_dot_viewer * normal_dot_h);
            let g = (2.0 * g) / viewer_dot_h;
            let g = g.min(1.0);

            // BRDF: putting it all together
            // Cook-Torrance = DFG / (4(n • wi)(n • wo))
            // Lambert = (1 - F) * c / π
            let specular = d * f * g / (4.0 * normal_dot_viewer * normal_dot_ray);
            if self.transparent {
                specular
            } else {
                let diffuse = Vec3::new(1.0, 1.0, 1.0) - f.mul_element_wise(self.color) / PI;
                specular + diffuse
            }
        } else {
            // Ratio of refractive indices, η_i / η_o
            let eta_t = if viewer_outside {
                self.index
            } else {
                1.0 / self.index
            };
            let h = (to_ray * eta_t + to_viewer).normalize(); // halfway vector
            let ray_dot_h = to_ray.dot(h);
            let viewer_dot_h = to_viewer.dot(h);
            let normal_dot_h = normal.dot(h);
            let nh2 = normal_dot_h.powi(2);

            // d: microfacet distribution function
            // D = exp(((n • h)^2 - 1) / (m^2 (n • h)^2)) / (π m^2 (n • h)^4)
            let m2 = self.roughness * self.roughness;
            let d = ((nh2 - 1.0) / (m2 * nh2)).exp() / (m2 * PI * nh2 * nh2);

            // f: fresnel, schlick's approximation
            // F = F0 + (1 - F0)(1 - wi • h)^5
            let f0 = ((self.index - 1.0) / (self.index + 1.0)).powi(2);
            let f0 = lerp(Vec3::new(f0, f0, f0), self.color, self.metallic);
            let f = f0 + (Vec3::new(1.0, 1.0, 1.0) - f0) * (1.0 - ray_dot_h.abs()).powi(5);

            // g: geometry function, microfacet shadowing
            // G = min(1, 2(n • h)(n • wo)/(wo • h), 2(n • h)(n • wi)/(wo • h))
            let g = f32::min((normal_dot_ray * normal_dot_h).abs(), (normal_dot_viewer * normal_dot_h).abs());
            let g = (2.0 * g) / viewer_dot_h.abs();
            let g = g.min(1.0);

            // BTDF: putting it all together
            // Cook-Torrance = |h • wi|/|n • wi| * |h • wo|/|n • wo|
            //                  * η_o^2 (1 - F)DG / (η_i (h • wi) + η_o (h • wo))^2
            let btdf = (ray_dot_h * viewer_dot_h / (normal_dot_h * normal_dot_viewer)).abs()
                * (d * (Vec3::new(1.0, 1.0, 1.0) - f) * g / (eta_t * ray_dot_h + viewer_dot_h).powi(2));

            btdf.mul_element_wise(self.color)
        }
    }

    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> (Color, Option<Ray>) {
        (Color::new(1.0, 0.0, 0.0), None)
        // let mut scatter_direction = hit.normal + sample_unit_sphere_surface();
        //
        // if near_zero(scatter_direction) {
        //     // Catch degenerate scatter direction
        //     scatter_direction = hit.normal;
        // }
        //
        // let scattered = Ray::new(hit.point, scatter_direction);
        // (self.color, Some(scattered))
    }
}
