use std::{f32::consts::PI, ops::{Add, Mul}};

use cgmath::InnerSpace;
use rand::{thread_rng, Rng};

use crate::core::{Vec3, Vec2};

/// Returns true if a vector is smaller than 1.0e-8 in each coordinate
pub fn near_zero(vector: Vec3) -> bool {
    const EPS: f32 = 1.0e-8;
    vector.x.abs() < EPS && vector.y.abs() < EPS && vector.z.abs() < EPS
}

/// Reflect `a` about the normal `n`
pub fn reflect(vector: Vec3, normal: Vec3) -> Vec3 {
    vector - 2.0 * vector.dot(normal) * normal
}

pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = f32::min(n.dot(-uv), 1.0);
    let out_perp = etai_over_etat * (uv + cos_theta * n);
    let out_parallel = -(f32::abs(1.0 - out_perp.magnitude2())).sqrt() * n;
    out_perp + out_parallel
}

#[inline]
pub fn to_spherical_coords(vector: Vec3) -> Vec2 {
    Vec2 {
        x: (-vector.y).acos(),
        y: f32::atan2(-vector.z, vector.x) + PI,
    }
}

#[inline]
pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.0
}

#[inline]
pub fn radians_to_degrees(radians: f32) -> f32 {
    radians * 180.0 / PI
}

pub fn lerp<T>(a: T, b: T, t: f32) -> T where T: Add<Output = T> + Mul<f32, Output = T> {
    a * (1.0 - t) + b
}

pub fn random_vector(min: f32, max: f32) -> Vec3 {
    let rng = &mut thread_rng();
    Vec3::new(rng.gen_range(min..=max), rng.gen_range(min..=max), rng.gen_range(min..=max))
}

pub fn sample_unit_sphere_surface() -> Vec3 {
    random_vector(-1.0, 1.0).normalize()
    // loop {
    //     let vec = random_vector(-1.0, 1.0);
    //     if vec.magnitude2() <= 1.0 {
    //         return vec.normalize();
    //     }
    // }
}
