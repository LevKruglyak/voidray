use std::ops::{
    Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Range, Sub, SubAssign,
};

use voidray_math::types::Float;

use self::rgb::RgbSpectrum;

pub mod rgb;
pub mod sampled;

// Note: Note that the presence of this sample accessor imposes the
// implicit assumption that the spectral representation is a set of
// coefficients that linearly scale a fixed set of basis functions.

/// Represents a spectral power distribution
pub trait CoefficientSpectrum: Sized + Deref<Target = [Float]> + DerefMut {
    /// Initialize a spectrum with constant value across all wavelengths
    fn constant(value: Float) -> Self;

    const NUM_SAMPLES: usize;

    fn sqrt(mut self) -> Self {
        for x in &mut self[..] {
            *x = x.sqrt();
        }
        self
    }

    fn exp(mut self) -> Self {
        for x in &mut self[..] {
            *x = x.exp();
        }
        self
    }

    fn powf(mut self, pow: Float) -> Self {
        for x in &mut self[..] {
            *x = x.powf(pow);
        }
        self
    }

    fn powi(mut self, pow: i32) -> Self {
        for x in &mut self[..] {
            *x = x.powi(pow);
        }
        self
    }

    fn clamp(mut self, range: Range<Float>) -> Self {
        for x in &mut self[..] {
            *x = x.clamp(range.start, range.end);
        }
        self
    }
}

/// Macro for automatically generatic arithmetic operations for SPDs
macro_rules! impl_spectrum_arithmetic {
    ($spectrum:ty) => {
        impl Add for $spectrum {
            type Output = Self;

            fn add(mut self, rhs: Self) -> Self {
                self += rhs;
                self
            }
        }

        impl AddAssign for $spectrum {
            fn add_assign(&mut self, rhs: Self) {
                for i in 0..<$spectrum as CoefficientSpectrum>::NUM_SAMPLES {
                    self[i] += rhs[i];
                }
            }
        }

        impl Mul for $spectrum {
            type Output = Self;

            fn mul(mut self, rhs: Self) -> Self {
                self *= rhs;
                self
            }
        }

        impl MulAssign for $spectrum {
            fn mul_assign(&mut self, rhs: Self) {
                for i in 0..<$spectrum as CoefficientSpectrum>::NUM_SAMPLES {
                    self[i] *= rhs[i];
                }
            }
        }

        impl Mul<Float> for $spectrum {
            type Output = Self;

            fn mul(mut self, rhs: Float) -> Self {
                for i in 0..<$spectrum as CoefficientSpectrum>::NUM_SAMPLES {
                    self[i] *= rhs;
                }
                self
            }
        }

        impl Sub for $spectrum {
            type Output = Self;

            fn sub(mut self, rhs: Self) -> Self {
                self -= rhs;
                self
            }
        }

        impl SubAssign for $spectrum {
            fn sub_assign(&mut self, rhs: Self) {
                for i in 0..<$spectrum as CoefficientSpectrum>::NUM_SAMPLES {
                    self[i] -= rhs[i];
                }
            }
        }

        impl Div for $spectrum {
            type Output = Self;

            fn div(mut self, rhs: Self) -> Self {
                self /= rhs;
                self
            }
        }

        impl Div<Float> for $spectrum {
            type Output = Self;

            fn div(mut self, rhs: Float) -> Self {
                for i in 0..<$spectrum as CoefficientSpectrum>::NUM_SAMPLES {
                    self[i] /= rhs;
                }
                self
            }
        }

        impl DivAssign for $spectrum {
            fn div_assign(&mut self, rhs: Self) {
                for i in 0..<$spectrum as CoefficientSpectrum>::NUM_SAMPLES {
                    self[i] /= rhs[i];
                }
            }
        }
    };
}

impl_spectrum_arithmetic!(RgbSpectrum);

#[cfg(test)]
mod tests {}
