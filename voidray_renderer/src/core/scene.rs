use super::camera::{Camera, CameraAcceleration};
use super::texture::{ImageTexture, SampleType, Texture};
use crate::aabb::{AABB, Bounded};
use crate::bvh::{BvhNode, BoundsCollection};
use crate::core::traits::*;
use crate::mesh::Mesh;
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
    pub textures: Vec<Named<Arc<Texture>>>,
    pub objects: Vec<Named<Object>>,
    pub surfaces: Vec<Named<Surface>>,
    pub meshes: Vec<Arc<Mesh>>,
    pub materials: Vec<Named<Arc<dyn Material>>>,
    pub environment: Option<Arc<dyn Environment>>,
}

#[derive(Clone, Copy)]
pub struct MaterialHandle(usize);

#[derive(Clone, Copy)]
pub struct ObjectHandle(usize);

#[derive(Clone, Copy)]
pub struct SurfaceHandle(usize);

#[derive(Clone, Copy)]
pub struct TextureHandle(usize);

#[derive(Clone, Copy)]
pub struct MeshHandle(usize);

pub struct SceneAcceleration {
    pub camera: CameraAcceleration,
    pub environment: Option<Arc<dyn Environment>>,
    bvh: BvhNode,
    objects: Vec<Object>,
    surfaces: Vec<Surface>,
    textures: Vec<Arc<Texture>>,
    meshes: Vec<Arc<Mesh>>,
    materials: Vec<Arc<dyn Material>>,
}

impl BoundsCollection for SceneAcceleration {
    fn bounds_ref(&self, handle: usize) -> AABB {
        let surface = self.surface_ref(SurfaceHandle(handle));

        match surface {
            Surface::Mesh(handle) => self.mesh_ref(*handle).bounds(),
            Surface::Analytic(surface) => surface.bounds(),
        }
    }

    fn hit(&self, handle: usize, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        match self.surface_ref(SurfaceHandle(handle)) {
            &Surface::Mesh(handle) => self.mesh_ref(handle).hit(ray, t_min, t_max),
            Surface::Analytic(surface) => surface.hit(ray, t_min, t_max),
        }
    }

    fn objects(&self) -> Vec<usize> {
        (0..self.surfaces.len()).collect()
    }
}

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
            textures: Vec::new(),
            meshes: Vec::new(),
            environment: None,
        }
    }

    pub fn add_material(&mut self, material: Arc<dyn Material>) -> MaterialHandle {
        self.materials.push(Named {
            object: material,
            name: format!("material_{}", self.materials.len()),
        });
        MaterialHandle(self.materials.len() - 1)
    }

    pub fn add_analytic_surface(&mut self, analytic: Arc<dyn AnalyticSurface>) -> SurfaceHandle {
        self.surfaces.push(Named {
            object: Surface::Analytic(analytic),
            name: format!("surface_{}", self.surfaces.len()),
        });
        SurfaceHandle(self.surfaces.len() - 1)
    }

    pub fn add_mesh(&mut self, mesh: Arc<Mesh>) -> SurfaceHandle {
        self.meshes.push(mesh);
        self.surfaces.push(Named {
            object: Surface::Mesh(MeshHandle(self.meshes.len() - 1)),
            name: format!("mesh_{}", self.surfaces.len()),
        });
        SurfaceHandle(self.surfaces.len() - 1)
    }

    pub fn add_mesh_from_file(&mut self, path: &str) -> SurfaceHandle {
        self.meshes.push(Arc::new(Mesh::from_file(path)));
        self.surfaces.push(Named {
            object: Surface::Mesh(MeshHandle(self.meshes.len() - 1)),
            name: format!("mesh_{}", self.surfaces.len()),
        });
        SurfaceHandle(self.surfaces.len() - 1)
    }

    pub fn add_object(&mut self, material: MaterialHandle, surface: SurfaceHandle) -> ObjectHandle {
        self.objects.push(Named {
            object: Object { surface, material },
            name: format!("object_{}", self.objects.len()),
        });
        ObjectHandle(self.objects.len() - 1)
    }

    pub fn add_image_texture(&mut self, path: &str, sample_type: SampleType) -> TextureHandle {
        self.textures.push(Named {
            object: Arc::new(Texture::Image(ImageTexture::new(path, sample_type))),
            name: format!("texture_{}", self.textures.len()),
        });
        TextureHandle(self.textures.len() - 1)
    }
}

impl Accelerable<SceneAcceleration> for Scene {
    fn build_acceleration(&self) -> SceneAcceleration {
        let mut scene_accel = SceneAcceleration {
            camera: self.camera.build_acceleration(),
            objects: self.objects.build_acceleration(),
            surfaces: self.surfaces.build_acceleration(),
            materials: self.materials.build_acceleration(),
            textures: self.textures.build_acceleration(),
            meshes: self.meshes.clone(),
            bvh: BvhNode::None,
            environment: self.environment.clone(),
        };

        scene_accel.bvh = BvhNode::from_list(&mut scene_accel.objects(), &scene_accel);
        scene_accel
    }
}

impl SceneAcceleration {
    pub fn hit(&self, ray: &Ray) -> Option<(HitRecord, &Object)> {
        self.bvh.hit(ray, 0.00001, INF, self).map(|(hit, handle)| {
            (hit, self.object_ref(ObjectHandle(handle)))
        })

        // let mut result = None;
        //
        // let t_min = 0.00001;
        // let mut closest = INF;
        //
        // // Simple implementation
        // for object in &self.objects {
        //     let surface = self.surface_ref(object.surface);
        //     if let Some(hit) = match surface {
        //         Surface::Analytic(analytic) => analytic.hit(ray, t_min, closest),
        //         &Surface::Mesh(handle) => self.mesh_ref(handle).hit(ray, t_min, closest), 
        //     } {
        //         closest = hit.t;
        //         result = Some((hit, object));
        //     }
        // }
        //
        // result
    }

    pub fn surface_ref(&self, surface_handle: SurfaceHandle) -> &Surface {
        &self.surfaces[surface_handle.0]
    }

    pub fn material_ref(&self, material_handle: MaterialHandle) -> &dyn Material {
        self.materials[material_handle.0].as_ref()
    }

    pub fn texture_ref(&self, texture_handle: TextureHandle) -> &Texture {
        self.textures[texture_handle.0].as_ref()
    }

    pub fn mesh_ref(&self, mesh_handle: MeshHandle) -> &Mesh {
        self.meshes[mesh_handle.0].as_ref()
    }

    pub fn object_ref(&self, object_handle: ObjectHandle) -> &Object {
        &self.objects[object_handle.0]
    }
}
