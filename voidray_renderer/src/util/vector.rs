pub use cgmath::{ElementWise, InnerSpace};
use cgmath::{Matrix3, Matrix4, Vector2, Vector3, Vector4};

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

pub type Vec4 = Vector4<Float>;
pub type Vec3 = Vector3<Float>;
pub type Vec2 = Vector2<Float>;

pub type Mat3 = Matrix3<Float>;
pub type Mat4 = Matrix4<Float>;

/// Simple Vec initialization macro
#[macro_export]
macro_rules! vec3 {
    ($x:expr) => {{
        Vec3::new($x, $x, $x)
    }};
    ($x:expr,$y:expr,$z:expr) => {{
        Vec3::new($x, $y, $z)
    }};
}

#[test]
fn vec_macro_test() {
    let a = Vec3::new(1.0, 1.0, 1.0);
    assert_eq!(a, vec3!(1.0));

    let b = Vec3::new(0.5, 0.6, 0.7);
    assert_eq!(b, vec3!(0.5, 0.6, 0.7));
}
