use cgmath::InnerSpace;
use derive_new::new;

use crate::utils::math::degrees_to_radians;

use super::{ray::Ray, Float, Vec3};

#[derive(Debug, new)]
pub struct RayOrigin {
    pub origin: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub lower_left_corner: Vec3,
}

impl RayOrigin {
    pub fn cast_ray(&self, s: Float, t: Float) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + self.horizontal * s - self.vertical * t - self.origin,
        )
    }
}

pub struct Camera {
    pub eye: Vec3,
    pub direction: Vec3,
    pub up: Vec3,
    pub fov: Float,
    pub aspect_ratio: Float,
}

impl Camera {
    pub fn to_ray_origin(&self) -> RayOrigin {
        let theta = degrees_to_radians(self.fov);
        let h = Float::tan(theta / 2.0);
        let viewport_height = 2.0 * h;
        let viewport_width = self.aspect_ratio * viewport_height;

        let w = self.direction.normalize();
        let u = (self.up.cross(w)).normalize();
        let v = w.cross(u);

        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;

        RayOrigin {
            origin: self.eye,
            horizontal,
            vertical,
            lower_left_corner: self.eye - horizontal / 2.0 + vertical / 2.0 - w,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: Vec3::new(0.0, 1.0, 4.0),
            direction: Vec3::new(0.0, -0.1, 1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov: 70.0,
            aspect_ratio: 1.0,
        }
    }
}
