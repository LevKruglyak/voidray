use crate::utils::color::Color;

use super::ray::{HitRecord, Ray};

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> (Color, bool, Option<Ray>);
}
