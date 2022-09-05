use crate::common::{objects::Sphere, materials::{Lambertian, Dielectric}};

use super::{camera::{Camera, RayOrigin}, object::Object, Vec3, environment::{Environment, HDRIEnvironment}};

pub struct Scene {
    pub camera: Camera,
    objects: Vec<Box<dyn Object>>,
    pub environment: Box<dyn Environment>,
}

type MaterialHandle = usize;

impl Default for Scene {
    fn default() -> Self {
        let mut scene = Self {
            camera: Camera::default(),
            objects: Vec::new(),
            environment: Box::new(HDRIEnvironment::new("voidray/assets/studio.exr"))
        };

        scene.objects.push(Box::new(Sphere {
            center: Vec3::new(0.0, 2.0, 0.0),
            radius: 3.0,
            material: Box::new(Dielectric::new(1.33)),
        }));

        // scene.objects.push(Box::new(Sphere {
        //     center: Vec3::new(0.0, 2.0, 0.0),
        //     radius: -2.9,
        //     material: Box::new(Dielectric::new(1.33)),
        // }));
        
        scene.objects.push(Box::new(Sphere {
            center: Vec3::new(0.0, 2.0, 0.0),
            radius: -2.99,
            material: Box::new(Dielectric::new(1.33)),
        }));

        scene.objects.push(Box::new(Sphere {
            center: Vec3::new(-1.0, 2.0, 0.0),
            radius: 1.0,
            material: Box::new(Lambertian::new(Vec3::new(1.0, 0.0, 0.0))),
        }));

        scene.objects.push(Box::new(Sphere {
            center: Vec3::new(1.0, 2.0, 0.0),
            radius: 1.0,
            material: Box::new(Lambertian::new(Vec3::new(0.0, 0.0, 1.0))),
        }));

        scene
    }
}

impl Scene {
    pub fn to_acceleration(&self) -> SceneAcceleration {
        SceneAcceleration {
            ray_origin: self.camera.to_ray_origin(),
            objects: &self.objects,
            environment: self.environment.as_ref(),
        }
    }
}

pub struct SceneAcceleration<'a> {
    pub ray_origin: RayOrigin,
    // TODO: Replace
    pub objects: &'a Vec<Box<dyn Object>>,
    pub environment: &'a dyn Environment,
}
