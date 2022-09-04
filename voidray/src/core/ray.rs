use derive_new::new;

use super::Vec3;

#[derive(new, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}
