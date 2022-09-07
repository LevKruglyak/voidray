use std::sync::Arc;

use crate::{
    common::{materials::Materials, objects::Shapes},
    utils::color::{Color, Colors},
};

use super::{
    camera::{Camera, RayOrigin},
    environment::{Environment, HDRIEnvironment},
    material::Material,
    mesh::Mesh,
    object::{Object, Shape},
    ray::Hittable,
    Vec3,
};

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
    pub shapes: Vec<Shape>,
    pub meshes: Vec<Arc<Mesh>>,
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

    pub fn add_mesh(&mut self, mesh: Arc<Mesh>) -> ShapeHandle {
        self.meshes.push(mesh);
        self.shapes.push(Shape::Mesh(self.meshes.len() - 1));
        self.shapes.len() - 1
    }

    pub fn add_object(&mut self, material: MaterialHandle, shape: ShapeHandle) -> ObjectHandle {
        self.objects.push(Object { material, shape });
        self.objects.len() - 1
    }

    pub fn material_ref(&self, handle: MaterialHandle) -> &dyn Material {
        self.materials[handle].as_ref()
    }

    pub fn shape_ref(&self, handle: ShapeHandle) -> &Shape {
        &self.shapes[handle]
    }

    pub fn mesh_ref(&self, handle: MeshHandle) -> &Mesh {
        self.meshes[handle].as_ref()
    }
}

impl Default for Scene {
    fn default() -> Self {
        let mut scene = Self {
            camera: Camera::default(),
            materials: Vec::new(),
            objects: Vec::new(),
            shapes: Vec::new(),
            meshes: Vec::new(),
            environment: Arc::new(HDRIEnvironment::new("voidray/assets/studio.exr")),
        };

        scene.camera = Camera {
            eye: Vec3::new(0.4, 0.6, 3.0),
            direction: Vec3::new(0.0, -0.1, 1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov: 70.0,
            aspect_ratio: 1.0,
        };

        let ball1_mat = Materials::dielectric(&mut scene, 1.33);
        let gold = Materials::metal(&mut scene, Color::new(0.83, 0.68, 0.07), 0.03);
        let stand_mat = Materials::lambertian(&mut scene, Color::new(0.053, 0.053, 0.053));
        let ground_mat = Materials::lambertian(&mut scene, Color::new(0.1, 0.01, 0.01));

        let red = Materials::colored_emissive(&mut scene, Colors::red(), 1000.0);
        let blue = Materials::colored_emissive(&mut scene, Colors::green(), 100.0);
        let green = Materials::colored_emissive(&mut scene, Colors::blue(), 100.0);

        let red_shape = Shapes::sphere(&mut scene, Vec3::new(1.3, 0.5, 0.0), 0.25);
        let gold_shape = Shapes::sphere(&mut scene, Vec3::new(-0.3, 0.20, 1.5), 0.20);
        let green_shape = Shapes::sphere(&mut scene, Vec3::new(-2.0, 3.0, 0.0), 0.5);
        //
        let material_main = scene.add_mesh(Arc::new(Mesh::from_file(
            "voidray/assets/material_testing_main.obj",
        )));
        let material_stand = scene.add_mesh(Arc::new(Mesh::from_file(
            "voidray/assets/material_testing_stand.obj",
        )));
        let ground_shape = Shapes::ground_plane(&mut scene, 0.0);
        // let cube_shape = scene.add_mesh(Arc::new(Mesh::from_file("voidray/assets/fancy_monkey.obj")));

        scene.add_object(ground_mat, ground_shape);
        scene.add_object(red, red_shape);
        // scene.add_object(gold, gold_shape);
        // scene.add_object(green, green_shape);
        // scene.add_object(ball1_mat, cube_shape);
        scene.add_object(ball1_mat, material_main);
        scene.add_object(stand_mat, material_stand);

        // let ground_mat = Materials::diffuse_glossy(&mut scene, Color::new(0.01, 0.01, 0.01), 0.1, 0.1);
        //
        // let red = Materials::colored_emissive(&mut scene, Colors::red(), 100.0);
        // let blue = Materials::colored_emissive(&mut scene, Colors::green(), 100.0);
        // let green = Materials::colored_emissive(&mut scene, Colors::blue(), 100.0);
        //
        // let red_shape = Shapes::sphere(&mut scene, Vec3::new(1.2, 0.5, 0.0), 0.5);
        // let blue_shape = Shapes::sphere(&mut scene, Vec3::new(0.0, 0.5, 0.0), 0.5);
        // let green_shape = Shapes::sphere(&mut scene, Vec3::new(-1.2, 0.5, 0.0), 0.5);
        //
        // let ground_shape = Shapes::ground_plane(&mut scene, 0.0);
        //
        // scene.add_object(ground_mat, ground_shape);
        // scene.add_object(red, red_shape);
        // scene.add_object(blue, blue_shape);
        // scene.add_object(green, green_shape);

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
            meshes: self.meshes.clone(),
        }
    }
}

pub struct SceneAcceleration {
    pub ray_origin: RayOrigin,
    pub objects: Vec<Object>,
    pub shapes: Vec<Shape>,
    pub materials: Vec<Arc<dyn Material>>,
    pub environment: Arc<dyn Environment>,
    pub meshes: Vec<Arc<Mesh>>,
}

impl SceneAcceleration {
    pub fn material_ref(&self, handle: MaterialHandle) -> &dyn Material {
        self.materials[handle].as_ref()
    }

    pub fn shape_ref(&self, handle: ShapeHandle) -> &Shape {
        &self.shapes[handle]
    }

    pub fn mesh_ref(&self, handle: MeshHandle) -> &Mesh {
        self.meshes[handle].as_ref()
    }
}
