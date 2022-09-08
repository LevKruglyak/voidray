use crate::utils::color::Color;

use super::ray::Ray;

pub trait Environment: Send + Sync {
    fn sample(&self, ray: &Ray) -> Color;
}
