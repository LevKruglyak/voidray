use crate::common::{objects::Sphere, materials::PrincipledBSDF};

use super::{camera::{Camera, PerspectiveCamera, RayOrigin}, object::Object, Vec3};

pub struct Scene {
    camera: Box<dyn Camera>,
    objects: Vec<Box<dyn Object>>,
}

type MaterialHandle = usize;

impl Default for Scene {
    fn default() -> Self {
        let mut scene = Self {
            camera: Box::new(PerspectiveCamera::default()),
            objects: Vec::new(),
        };

        scene.objects.push(Box::new(Sphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 0.01,
            material: PrincipledBSDF::diffuse(Vec3::new(1.0, 1.0, 1.0)),
        }));

        scene
    }
}

impl Scene {
    pub fn to_acceleration(&self) -> SceneAcceleration {
        SceneAcceleration {
            ray_origin: self.camera.to_ray_origin(),
            objects: &self.objects,
        }
    }
}

pub struct SceneAcceleration<'a> {
    pub ray_origin: RayOrigin,
    // TODO: Replace
    pub objects: &'a Vec<Box<dyn Object>>,
}
