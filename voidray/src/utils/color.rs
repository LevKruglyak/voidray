use crate::core::{Float, Vec3, Vec4};

pub type Color = Vec3;
pub type ColorAlpha = Vec4;

pub struct Colors {}

impl Colors {
    pub fn red() -> Color {
        Color::new(1.0, 0.0, 0.0)
    }

    pub fn green() -> Color {
        Color::new(0.0, 1.0, 0.0)
    }

    pub fn blue() -> Color {
        Color::new(0.0, 0.0, 1.0)
    }

    pub fn white() -> Color {
        Color::new(1.0, 1.0, 1.0)
    }

    pub fn gray(value: Float) -> Color {
        Color::new(value, value, value)
    }

    pub fn black() -> Color {
        Color::new(0.0, 0.0, 0.0)
    }
}

pub fn into_alpha(color: Color, alpha: Float) -> ColorAlpha {
    ColorAlpha::new(color.x, color.y, color.z, alpha)
}

pub fn alpha_mul(a: ColorAlpha, b: ColorAlpha) -> ColorAlpha {
    ColorAlpha::new(a.x * b.x, a.y * b.y, a.z * b.z, Float::max(a.w, b.w))
}

// #[derive(Clone, Copy, Debug)]
// pub struct Color {
//     pub r: f32,
//     pub g: f32,
//     pub b: f32,
// }
//
// impl Color {
//     #[inline]
//     pub fn new(r: f32, g: f32, b: f32) -> Self {
//         Self { r, g, b }
//     }
//
//     #[inline]
//     pub fn from(data: [f32; 4]) -> Self {
//         Self {
//             r: data[0],
//             g: data[1],
//             b: data[2],
//         }
//     }
//
//     #[inline]
//     pub fn data(&self) -> [f32; 4] {
//         [self.r, self.g, self.b, 1.0]
//     }
//
//     pub fn to_vec(&self) -> Vec3 {
//         Vec3::new(self.r, self.g, self.b)
//     }
//
//     pub fn from_vec(vec: Vec3) -> Color {
//         Color::new(vec.x, vec.y, vec.z)
//     }
// }
//
// impl Mul<f32> for Color {
//     type Output = Self;
//     fn mul(self, rhs: f32) -> Self::Output {
//         Self {
//             r: self.r * rhs,
//             b: self.b * rhs,
//             g: self.g * rhs,
//         }
//     }
// }
//
// impl MulAssign<f32> for Color {
//     fn mul_assign(&mut self, rhs: f32) {
//         self.r *= rhs;
//         self.g *= rhs;
//         self.b *= rhs;
//     }
// }
//
// impl Add for Color {
//     type Output = Self;
//     fn add(self, rhs: Self) -> Self::Output {
//         Self {
//             r: self.r + rhs.r,
//             g: self.g + rhs.g,
//             b: self.b + rhs.b,
//         }
//     }
// }
//
// impl AddAssign for Color {
//     fn add_assign(&mut self, rhs: Self) {
//         self.r += rhs.r;
//         self.g += rhs.g;
//         self.b += rhs.b;
//     }
// }
//
// impl Mul for Color {
//     type Output = Self;
//     fn mul(self, rhs: Self) -> Self::Output {
//         Self {
//             r: self.r * rhs.r,
//             g: self.g * rhs.g,
//             b: self.b * rhs.b,
//         }
//     }
// }
