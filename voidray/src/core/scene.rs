use std::sync::Arc;

use crate::{utils::color::Color, common::{materials::Materials, objects::Shapes}};

use super::{
    camera::{Camera, RayOrigin},
    environment::{Environment, HDRIEnvironment, UniformEnvironment},
    object::{Object, Shape},
    Vec3, material::Material, ray::Hittable,
};

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
    pub shapes: Vec<Shape>,
    pub materials: Vec<Arc<dyn Material>>,
    pub environment: Arc<dyn Environment>,
}

pub type MaterialHandle = usize;
pub type ObjectHandle = usize;
pub type ShapeHandle = usize;
pub type MeshHandle = usize;

impl Scene {
    pub fn add_material(&mut self, material: Arc<dyn Material>) -> MaterialHandle {
        self.materials.push(material);
        self.materials.len() - 1
    }

    pub fn add_analytic(&mut self, analytic: Arc<dyn Hittable>) -> ShapeHandle {
        self.shapes.push(Shape::Analytic(analytic));
        self.shapes.len() - 1
    }

    pub fn add_object(&mut self, material: MaterialHandle, shape: ShapeHandle) -> ObjectHandle {
        self.objects.push(Object { material, shape, });
        self.objects.len() - 1
    }

    pub fn material_ref(&self, handle: MaterialHandle) -> &dyn Material {
        self.materials[handle].as_ref()
    }

    pub fn shape_ref(&self, handle: ShapeHandle) -> &Shape {
        &self.shapes[handle]
    }
}

impl Default for Scene {
    fn default() -> Self {
        let mut scene = Self {
            camera: Camera::default(),
            materials: Vec::new(),
            objects: Vec::new(),
            shapes: Vec::new(),
            environment: Arc::new(HDRIEnvironment::new("voidray/assets/studio.exr")),
        };

        let ball1_mat = Materials::dielectric(&mut scene, 1.33);
        let ball2_mat = Materials::lambertian(&mut scene, Color::new(1.0, 0.1, 0.05));
        let emissive = Materials::emissive(&mut scene, 10.0);
        let ground_mat = Materials::lambertian(&mut scene, Color::new(0.2, 0.2, 0.2));

        let ball1_shape = Shapes::sphere(&mut scene, Vec3::new(2.0, 2.0, 0.0), 3.0);
        let ball2_shape = Shapes::sphere(&mut scene, Vec3::new(-2.0, 2.0, 0.0), 3.0);
        let emissive_shape = Shapes::sphere(&mut scene, Vec3::new(0.0, 20.0, 0.0), 3.0);
        let ground_shape = Shapes::ground_plane(&mut scene, -1.0);

        scene.add_object(ground_mat, ground_shape);
        scene.add_object(ball1_mat, ball1_shape);
        scene.add_object(ball2_mat, ball2_shape);
        // scene.add_object(emissive, emissive_shape);

        scene
    }
}

impl Scene {
    pub fn to_acceleration(&self) -> SceneAcceleration {
        SceneAcceleration {
            ray_origin: self.camera.to_ray_origin(),
            objects: self.objects.clone(),
            shapes: self.shapes.clone(),
            materials: self.materials.clone(),
            environment: self.environment.clone(),
        }
    }
}

pub struct SceneAcceleration {
    pub ray_origin: RayOrigin,
    pub objects: Vec<Object>,
    pub shapes: Vec<Shape>,
    pub materials: Vec<Arc<dyn Material>>,
    pub environment: Arc<dyn Environment>,
}

impl SceneAcceleration {
    pub fn material_ref(&self, handle: MaterialHandle) -> &dyn Material {
        self.materials[handle].as_ref()
    }

    pub fn shape_ref(&self, handle: ShapeHandle) -> &Shape {
        &self.shapes[handle]
    }
}
