use std::ops::{Add, AddAssign, Mul};

use num::Float;

/// Supports linear interpolation between two values by a float
pub trait Lerp<F: Float> {
    fn lerp(t: F, a: Self, b: Self) -> Self;
}

/// Very generic `lerp` auto-implementation
impl<F: Float, G: Add<Output = G> + AddAssign + Mul<F, Output = G>> Lerp<F> for G {
    fn lerp(t: F, a: Self, b: Self) -> Self {
        // To avoid copying for large `G`, use AddAssign
        let mut x = a * t;
        x += b * (F::one() - t);
        x
    }
}
