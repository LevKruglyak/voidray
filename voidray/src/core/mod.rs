use cgmath::{Vector2, Vector3};

pub mod camera;
pub mod environment;
pub mod material;
pub mod object;
pub mod ray;
pub mod scene;
pub mod tracer;

#[cfg(feature = "high_precision")]
pub type Float = f64;
#[cfg(feature = "high_precision")]
pub static PI: Float = std::f64::consts::PI;
#[cfg(feature = "high_precision")]
pub static INF: Float = std::f64::INFINITY;

#[cfg(not(feature = "high_precision"))]
pub type Float = f32;
#[cfg(not(feature = "high_precision"))]
pub static PI: Float = std::f32::consts::PI;
#[cfg(not(feature = "high_precision"))]
pub static INF: Float = std::f32::INFINITY;

pub type Vec3 = Vector3<Float>;
pub type Vec2 = Vector2<Float>;
