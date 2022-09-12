use std::sync::Arc;

use voidray_renderer::color::*;
use voidray_renderer::math::{near_zero, reflect, refract};
use voidray_renderer::rand::rand_distr::UnitSphere;
use voidray_renderer::rand::{Rng, ThreadRng};
use voidray_renderer::ray::*;
use voidray_renderer::scene::{SceneAcceleration, TextureHandle};
use voidray_renderer::texture::AbstractTexture;
use voidray_renderer::traits::Material;
use voidray_renderer::vector::*;

pub struct Materials {}

impl Materials {
    pub fn lambertian(albedo: Color) -> Arc<dyn Material> {
        Arc::new(Lambertian {
            albedo: ColorType::Color(albedo),
            normal: None,
        })
    }

    pub fn lambertian_texture(albedo: TextureHandle, normal: TextureHandle) -> Arc<dyn Material> {
        Arc::new(Lambertian {
            albedo: ColorType::Texture(albedo),
            normal: Some(normal),
        })
    }

    pub fn metal(albedo: Color, fuzz: Float) -> Arc<dyn Material> {
        Arc::new(Metal { albedo, fuzz })
    }

    pub fn dielectric(ir: Float) -> Arc<dyn Material> {
        Arc::new(Dielectric { ir })
    }
    //
    // pub fn colored_dielectric(color: Color, ir: Float, transparency: Float) -> Arc<dyn Material> {
    //     Arc::new(MixMaterial::new(
    //         Box::new(Lambertian::new(color)),
    //         Box::new(Dielectric::new(ir)),
    //         transparency,
    //     ))
    // }
    //
    // pub fn emissive(strength: Float) -> Arc<dyn Material> {
    //     Arc::new(Emission::new(Color::new(1.0, 1.0, 1.0), strength))
    // }
    //
    pub fn colored_emissive(color: Color, strength: Float) -> Arc<dyn Material> {
        Arc::new(Emission::new(color, strength))
    }
    //
    // pub fn diffuse_glossy(
    //     color: Color,
    //     roughness: Float,
    //     reflectiveness: Float,
    // ) -> Arc<dyn Material> {
    //     Arc::new(MixMaterial::new(
    //         Box::new(Lambertian::new(color)),
    //         Box::new(Metal::new(Color::new(1.0, 1.0, 1.0), roughness)),
    //         reflectiveness,
    //     ))
    // }
}

pub enum ColorType {
    Color(Color),
    Texture(TextureHandle),
}

pub struct Lambertian {
    albedo: ColorType,
    normal: Option<TextureHandle>,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self {
            albedo: ColorType::Color(albedo),
            normal: None,
        }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        scene: &SceneAcceleration,
        ray: &Ray,
        hit: &HitRecord,
        rng: &mut ThreadRng,
    ) -> (Color, Option<Ray>) {
        let normal = if let Some(normal_map) = self.normal {
            scene.texture_ref(normal_map).sample(hit.uv.x, hit.uv.y).0
        } else {
            hit.normal
        };

        let mut scatter_direction = normal + Vec3::from(rng.sample(UnitSphere));

        if near_zero(scatter_direction) {
            // Catch degenerate scatter direction
            scatter_direction = normal;
        }

        let scattered = Ray::new(hit.point, scatter_direction);
        match self.albedo {
            ColorType::Color(albedo) => (albedo, Some(scattered)),
            ColorType::Texture(texture) => (
                scene.texture_ref(texture).sample(hit.uv.x, hit.uv.y),
                // Color::new(u, v, 0.0),
                Some(scattered),
            ),
        }
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: Float,
}

impl Material for Metal {
    fn scatter(
        &self,
        scene: &SceneAcceleration,
        ray: &Ray,
        hit: &HitRecord,
        rng: &mut ThreadRng,
    ) -> (Color, Option<Ray>) {
        let reflected = reflect(ray.direction, hit.normal).normalize();
        let scattered = Ray::new(
            hit.point,
            reflected + self.fuzz * Vec3::from(rng.sample(UnitSphere)),
        );

        if scattered.direction.dot(hit.normal) > 0.0 {
            (self.albedo, Some(scattered))
        } else {
            (BLACK, None)
        }
    }
}
//
pub struct Emission {
    color: Color,
}

impl Emission {
    pub fn new(color: Color, strength: Float) -> Self {
        Self {
            color: color * strength,
        }
    }
}

impl Material for Emission {
    fn scatter(
        &self,
        scene: &SceneAcceleration,
        ray: &Ray,
        hit: &HitRecord,
        rng: &mut ThreadRng,
    ) -> (Color, Option<Ray>) {
        (self.color, None)
    }
}
//
pub struct Dielectric {
    ir: Float,
}

impl Dielectric {
    fn reflectance(cosine: Float, idx: Float) -> Float {
        // Schlick's approximation for reflectance
        let mut r0 = (1.0 - idx) / (1.0 + idx);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * Float::powi(1.0 - cosine, 5)
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        scene: &SceneAcceleration,
        ray: &Ray,
        hit: &HitRecord,
        rng: &mut ThreadRng,
    ) -> (Color, Option<Ray>) {
        let refraction_ratio = if hit.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction = ray.direction.normalize();
        let cos_theta = Float::min(hit.normal.dot(-unit_direction), 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract: bool = (refraction_ratio * sin_theta) > 1.0;
        let direction = if cannot_refract
            || Dielectric::reflectance(cos_theta, refraction_ratio) > rng.gen_range(0.0..1.0)
        {
            reflect(unit_direction, hit.normal)
        } else {
            refract(unit_direction, hit.normal, refraction_ratio)
        };

        (
            Color::new(1.0, 1.0, 1.0),
            Some(Ray::new(hit.point, direction)),
        )
    }
}
//
// pub struct MixMaterial {
//     first: Box<dyn Material>,
//     second: Box<dyn Material>,
//     factor: Float,
// }
//
// impl Material for MixMaterial {
//     fn scatter(&self, ray: &Ray, hit: &HitRecord) -> (Color, bool, Option<Ray>) {
//         if thread_rng().gen_range(0.0..1.0) >= self.factor {
//             self.first.scatter(ray, hit)
//         } else {
//             self.second.scatter(ray, hit)
//         }
//     }
// }
