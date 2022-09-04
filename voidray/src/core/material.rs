use crate::utils::color::Color;

use super::{Vec3, ray::{Ray, HitRecord}};

pub trait Material: Send + Sync {
    fn bsdf(&self, normal: Vec3, to_viewer: Vec3, to_ray: Vec3) -> Color;
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> (Color, Option<Ray>);
} 
