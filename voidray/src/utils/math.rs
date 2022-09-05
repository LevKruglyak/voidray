use std::ops::{Add, Mul};

use cgmath::InnerSpace;
use rand::{thread_rng, Rng};

use crate::core::{Float, Vec2, Vec3, PI};

/// Returns true if a vector is smaller than 1.0e-8 in each coordinate
pub fn near_zero(vector: Vec3) -> bool {
    const EPS: Float = 1.0e-8;
    vector.x.abs() < EPS && vector.y.abs() < EPS && vector.z.abs() < EPS
}

/// Reflect `a` about the normal `n`
pub fn reflect(vector: Vec3, normal: Vec3) -> Vec3 {
    vector - 2.0 * vector.dot(normal) * normal
}

pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: Float) -> Vec3 {
    let cos_theta = Float::min(n.dot(-uv), 1.0);
    let out_perp = etai_over_etat * (uv + cos_theta * n);
    let out_parallel = -(Float::abs(1.0 - out_perp.magnitude2())).sqrt() * n;
    out_perp + out_parallel
}

#[inline]
pub fn to_spherical_coords(vector: Vec3) -> Vec2 {
    Vec2 {
        x: (-vector.y).acos(),
        y: Float::atan2(-vector.z, vector.x) + PI,
    }
}

#[inline]
pub fn degrees_to_radians(degrees: Float) -> Float {
    degrees * PI / 180.0
}

#[inline]
pub fn radians_to_degrees(radians: Float) -> Float {
    radians * 180.0 / PI
}

pub fn lerp<T>(a: T, b: T, t: Float) -> T
where
    T: Add<Output = T> + Mul<Float, Output = T>,
{
    a * (1.0 - t) + b
}

pub fn random_vector(min: Float, max: Float) -> Vec3 {
    let rng = &mut thread_rng();
    Vec3::new(
        rng.gen_range(min..=max),
        rng.gen_range(min..=max),
        rng.gen_range(min..=max),
    )
}

pub fn sample_unit_sphere_surface() -> Vec3 {
    random_vector(-1.0, 1.0).normalize()
}
