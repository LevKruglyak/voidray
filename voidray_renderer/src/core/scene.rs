use super::camera::{Camera, CameraAcceleration};
use crate::core::traits::*;
use crate::preamble::*;
use crate::ray::*;

/// Represents an (object, name) pair
pub struct Named<T> {
    object: T,
    name: String,
}

/// Represents a structure which can be turned into an acceleration structure
pub trait Accelerable<A> {
    fn build_acceleration(&self) -> A;
}

impl<T> Accelerable<Vec<T>> for Vec<Named<T>>
where
    T: Clone,
{
    fn build_acceleration(&self) -> Vec<T> {
        self.iter().map(|named| named.object.clone()).collect()
    }
}

#[derive(Clone)]
pub struct Object {
    pub surface: SurfaceHandle,
    pub material: MaterialHandle,
}

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Named<Object>>,
    pub surfaces: Vec<Named<Surface>>,
    pub materials: Vec<Named<Arc<dyn Material>>>,
    pub environment: Option<Arc<dyn Environment>>,
}

#[derive(Clone, Copy)]
pub struct MaterialHandle(usize);

#[derive(Clone, Copy)]
pub struct ObjectHandle(usize);

#[derive(Clone, Copy)]
pub struct SurfaceHandle(usize);

pub struct SceneAcceleration {
    pub camera: CameraAcceleration,
    pub environment: Option<Arc<dyn Environment>>,
    objects: Vec<Object>,
    surfaces: Vec<Surface>,
    materials: Vec<Arc<dyn Material>>,
}

// let mut result = None;
// let mut closest_so_far = Float::INFINITY;
//
// for object in &scene.objects {
//     let shape = scene.shape_ref(object.shape);
//     if let Some(hit) = match shape {
//         Shape::Analytic(hittable) => hittable.hit(ray, T_MIN, closest_so_far),
//         Shape::Mesh(handle) => scene.mesh_ref(*handle).hit(ray, T_MIN, closest_so_far),
//     } {
//         closest_so_far = hit.t;
//         result = Some((hit, object));
//     }
// }
impl Scene {
    pub fn empty() -> Self {
        Self {
            camera: Camera::look_at(
                vec3!(1.0, 0.0, 10.0),
                vec3!(0.0),
                vec3!(0.0, 1.0, 0.0),
                PI / 6.0,
            ),
            materials: Vec::new(),
            objects: Vec::new(),
            surfaces: Vec::new(),
            environment: None,
        }
    }

    pub fn add_material(&mut self, material: Arc<dyn Material>) -> MaterialHandle {
        self.materials.push(Named { object: material, name: format!("material_{}", self.materials.len())});
        MaterialHandle(self.materials.len() - 1)
    }

    pub fn add_analytic_surface(&mut self, analytic: Arc<dyn AnalyticSurface>) -> SurfaceHandle {
        self.surfaces.push(Named { object: Surface::Analytic(analytic), name: format!("surface_{}", self.surfaces.len()) });
        SurfaceHandle(self.surfaces.len() - 1)
    }

    // pub fn add_mesh(&mut self, mesh: Arc<Mesh>) -> ShapeHandle {
    //     self.meshes.push(mesh);
    //     self.shapes.push(Shape::Mesh(self.meshes.len() - 1));
    //     self.shapes.len() - 1
    // }

    pub fn add_object(&mut self, material: MaterialHandle, surface: SurfaceHandle) -> ObjectHandle {
        self.objects.push(Named { object: Object { surface, material, }, name: format!("object_{}", self.objects.len())});
        ObjectHandle(self.objects.len() - 1)
    }
}

impl Accelerable<SceneAcceleration> for Scene {
    fn build_acceleration(&self) -> SceneAcceleration {
        SceneAcceleration {
            camera: self.camera.build_acceleration(),
            objects: self.objects.build_acceleration(),
            surfaces: self.surfaces.build_acceleration(),
            materials: self.materials.build_acceleration(),
            environment: self.environment.clone(),
        }
    }
}

impl SceneAcceleration {
    pub fn hit(&self, ray: &Ray) -> Option<(HitRecord, &Object)> {
        let mut result = None;

        let t_min = 0.00001;
        let mut closest = INF;

        // Simple implementation
        for object in &self.objects {
            let surface = self.surface_ref(object.surface);
            if let Some(hit) = match surface {
                Surface::Analytic(analytic) => analytic.hit(ray, t_min, closest),
            } {
                closest = hit.t;
                result = Some((hit, object));
            }
        }

        result
    }

    pub fn surface_ref(&self, surface_handle: SurfaceHandle) -> &Surface {
        &self.surfaces[surface_handle.0]
    }

    pub fn material_ref(&self, material_handle: MaterialHandle) -> &dyn Material {
        self.materials[material_handle.0].as_ref()
    }
}
