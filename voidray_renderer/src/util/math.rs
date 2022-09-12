use std::ops::{Add, Mul};

use crate::vector::*;

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
    a * (1.0 - t) + b * t
}

/// Returns a coordinate transformation which maps vectors in the normal coordinate system to the
/// world coordinate system.
pub fn local_to_world(normal: Vec3) -> Mat3 {
    let ns = if normal.x.is_normal() {
        vec3!(normal.y, -normal.x, 0.0).normalize()
    } else {
        vec3!(0.0, -normal.z, normal.y).normalize()
    };
    let nss = normal.cross(ns);
    Mat3::new(
        ns.x, nss.x, normal.x, ns.y, nss.y, normal.y, ns.z, nss.z, normal.z,
    )
}
