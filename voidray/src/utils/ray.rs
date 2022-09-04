use derive_new::new;

use crate::core::Vec3;

#[derive(new, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

// #[derive(Clone, Copy)]
// pub struct HitRecord<M> {
//     pub point: Vec3,
//     pub normal: Vec3,
//     pub t: Float,
//     pub front_face: bool,
//     pub material: M,
// }
//
// impl<M> HitRecord<M> {
//     pub fn new(point: Vec3, outward_normal: Vec3, t: Float, ray: &Ray, material: M) -> Self {
//         let front_face = ray.direction.dot(outward_normal) < 0.0;
//         let normal = if front_face {
//             outward_normal
//         } else {
//             -outward_normal
//         };
//
//         Self {
//             point,
//             normal,
//             t,
//             front_face,
//             material,
//         }
//     }
// }
//
// impl<T> PartialEq for HitRecord<T> {
//     fn eq(&self, other: &Self) -> bool {
//         self.t == other.t
//     }
// }
//
// impl<T> PartialOrd for HitRecord<T> {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.t.partial_cmp(&other.t)
//     }
// }
