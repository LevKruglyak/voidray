use super::CoefficientSpectrum;
use std::ops::{Deref, DerefMut};
use voidray_math::types::Float;

const SPECTRAL_SAMPLES: usize = 60;

pub struct RgbSpectrum {
    coefficients: [Float; SPECTRAL_SAMPLES],
}

impl CoefficientSpectrum for RgbSpectrum {
    const NUM_SAMPLES: usize = SPECTRAL_SAMPLES;

    fn constant(value: Float) -> Self {
        Self {
            coefficients: [value; Self::NUM_SAMPLES],
        }
    }
}

impl Deref for RgbSpectrum {
    type Target = [Float];

    fn deref(&self) -> &Self::Target {
        &self.coefficients
    }
}

impl DerefMut for RgbSpectrum {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.coefficients
    }
}
