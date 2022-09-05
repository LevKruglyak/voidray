use crate::utils::color::Color;

use super::ray::{Ray, HitRecord};

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> (Color, Option<Ray>);
} 
