use rand_distr::UnitDisc;

use crate::rand::*;
use crate::ray::*;
use crate::vector::*;

pub struct Camera {
    /// Location of the camera
    pub eye: Vec3,

    /// Direction that the camera is facing
    pub direction: Vec3,

    /// Direction of "up" for screen, must be orthogonal to `direction`
    pub up: Vec3,

    /// Field of view in the longer direction as an angle in radians, in (0, pi)
    pub fov: Float,

    /// Aperture radius and focal point for depth-of-field effects
    pub dof: Option<(Float, Vec3)>,
}

impl Camera {
    /// Perspective camera looking at a point, with a given field of view
    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3, fov: Float) -> Self {
        let direction = (center - eye).normalize();
        let up = (up - up.dot(direction) * direction).normalize();
        Self {
            eye,
            direction,
            up,
            fov,
            dof: None,
        }
    }

    pub fn build_acceleration(&self) -> CameraAcceleration {
        let d = (self.fov / 2.0).tan().recip();
        let right = self.direction.cross(self.up).normalize();
        let dof = self.dof.map(|(aperture, focal_point)| {
            (aperture, (focal_point - self.eye).dot(self.direction))
        });

        CameraAcceleration {
            origin: self.eye,
            direction: self.direction,
            right,
            up: self.up,
            d,
            dof,
        }
    }
}

/// Acceleration structure for a camera
pub struct CameraAcceleration {
    origin: Vec3,
    direction: Vec3,
    right: Vec3,
    up: Vec3,
    d: Float,
    // Aperture and focal length
    dof: Option<(Float, Float)>,
}

impl CameraAcceleration {
    /// Cast a ray, parametrized by x and y, each in the range [-1, 1]
    pub fn cast_ray(&self, x: Float, y: Float, rng: &mut ThreadRng) -> Ray {
        let mut origin = self.origin;
        let mut new_dir = self.d * self.direction + x * self.right + y * self.up;

        if let Some((aperture, focal_length)) = self.dof {
            // Depth of field
            let focal_point = origin + new_dir.normalize() * focal_length;
            let [x, y]: [Float; 2] = rng.sample(UnitDisc);
            origin += (x * self.right + y * self.up) * aperture;
            new_dir = focal_point - origin;
        }

        Ray::new(origin, new_dir.normalize())
    }
}
