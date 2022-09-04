use super::camera::{Camera, PerspectiveCamera, RayOrigin};

pub struct Scene {
    camera: Box<dyn Camera>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            camera: Box::new(PerspectiveCamera::default()),
        }
    }
}

impl Scene {
    pub fn into_acceleration(&self) -> SceneAcceleration {
        SceneAcceleration {
            ray_origin: self.camera.into_ray_origin(),
        }
    }
}

pub struct SceneAcceleration {
    pub ray_origin: RayOrigin,
}
