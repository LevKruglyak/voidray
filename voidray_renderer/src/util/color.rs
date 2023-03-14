use crate::vector::*;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign};

pub const BLACK: Color = Color(vec3!(0.0));

#[allow(non_snake_case)]
pub fn GRAY(v: Float) -> Color {
    Color(vec3!(v))
}

pub const WHITE: Color = Color(vec3!(1.0));

/// Simple linear rgb color structure
#[derive(Clone, Copy)]
pub struct Color(pub Vec3);

/// Creates a color for a hex
pub fn hex_color(x: u32) -> Color {
    let r = ((x >> 16) & 0xff) as Float / 255.0;
    let g = ((x >> 8) & 0xff) as Float / 255.0;
    let b = (x & 0xff) as Float / 255.0;
    Color(vec3!(r, g, b))
}

impl Color {
    pub fn new(r: Float, g: Float, b: Float) -> Color {
        Color(vec3!(r, g, b))
    }

    pub fn clamp(&self, max: Float) -> Color {
        Color(vec3!(
            Float::min(self.0.x, max),
            Float::min(self.0.y, max),
            Float::min(self.0.z, max)
        ))
    }

    pub fn mean(&self) -> Float {
        (self.0.x + self.0.y + self.0.z) / 3.0
    }

    pub fn r(&self) -> Float {
        self.0.x
    }

    pub fn g(&self) -> Float {
        self.0.y
    }

    pub fn b(&self) -> Float {
        self.0.z
    }

    pub fn a(&self) -> Float {
        1.0
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Self) -> Self::Output {
        Color(self.0.add(rhs.0))
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.0.add_assign(rhs.0);
    }
}

impl Mul for Color {
    type Output = Color;

    fn mul(self, rhs: Self) -> Self::Output {
        Color(self.0.mul_element_wise(rhs.0))
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Color(self.0 * rhs)
    }
}

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        self.0.mul_assign(rhs);
    }
}

impl Div<f32> for Color {
    type Output = Color;

    fn div(self, rhs: f32) -> Self::Output {
        Color(self.0 / rhs)
    }
}
