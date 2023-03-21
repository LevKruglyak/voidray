use super::CoefficientSpectrum;
use std::ops::{Deref, DerefMut};
use voidray_math::types::*;

const SPECTRAL_SAMPLES: usize = 60;

pub struct SampledSpectrum {
    coefficients: [Float; SPECTRAL_SAMPLES],
}

impl SampledSpectrum {
    // The range of the visual spectrum where the human
    // visual system is most sensitive in nanometers
    const SAMPLED_LAMBDA_START: usize = 400;
    const SAMPLED_LAMBDA_END: usize = 700;
}

impl CoefficientSpectrum for SampledSpectrum {
    const NUM_SAMPLES: usize = SPECTRAL_SAMPLES;

    fn constant(value: Float) -> Self {
        Self {
            coefficients: [value; Self::NUM_SAMPLES],
        }
    }
}

impl Deref for SampledSpectrum {
    type Target = [Float];

    fn deref(&self) -> &Self::Target {
        &self.coefficients
    }
}

impl DerefMut for SampledSpectrum {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.coefficients
    }
}
